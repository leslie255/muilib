use std::sync::{Arc, Mutex, Weak};

use pollster::FutureExt as _;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    button::{Button, ButtonRenderer},
    resources::AppResources,
    shapes::{BoundingBox, Font, Rect, RectRenderer, TextRenderer},
    theme::{ButtonKind, Theme},
    utils::*,
    wgpu_utils::{Canvas as _, CanvasView, ProjectionSpace, WindowCanvas},
};

pub(crate) struct Application<'cx> {
    resources: &'cx AppResources,
    ui: Option<Arc<Mutex<UiState<'cx>>>>,
}

impl<'cx> Application<'cx> {
    pub fn new(resources: &'cx AppResources) -> Self {
        Self {
            resources,
            ui: None,
        }
    }
}

impl<'cx> ApplicationHandler for Application<'cx> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match &mut self.ui {
            Some(_) => (),
            ui @ None => {
                let window = event_loop
                    .create_window(WindowAttributes::default().with_title("UI Test"))
                    .unwrap();
                let window = Arc::new(window);
                *ui = Some(UiState::create(self.resources, window));
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(ui) = self.ui.as_mut() {
            ui.lock()
                .unwrap()
                .window_event(event_loop, window_id, event)
        };
    }
}

fn init_wgpu() -> (wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&the_default());
    let adapter = instance.request_adapter(&the_default()).block_on().unwrap();
    let features = wgpu::FeaturesWGPU::POLYGON_MODE_LINE;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_features: features.into(),
            ..the_default()
        })
        .block_on()
        .unwrap();
    (instance, adapter, device, queue)
}

struct UiState<'cx> {
    resources: &'cx AppResources,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    window_canvas: WindowCanvas<'static>,
    text_renderer: TextRenderer<'cx>,
    rect_renderer: RectRenderer<'cx>,
    rect_background: Rect,
    button_renderer_mundane: ButtonRenderer<'cx, UiState<'cx>>,
    button_mundane: Button<UiState<'cx>>,
    button_renderer_primary: ButtonRenderer<'cx, UiState<'cx>>,
    button_primary: Button<UiState<'cx>>,
    button_renderer_toxic: ButtonRenderer<'cx, UiState<'cx>>,
    button_toxic: Button<UiState<'cx>>,
}

impl<'cx> UiState<'cx> {
    pub fn create(resources: &'cx AppResources, window: Arc<Window>) -> Arc<Mutex<Self>> {
        Arc::new_cyclic(|weak_self| Self::create_(weak_self, resources, window))
    }

    fn create_(
        weak_self: &Weak<Mutex<Self>>,
        resources: &'cx AppResources,
        window: Arc<Window>,
    ) -> Mutex<Self> {
        _ = weak_self;
        let (instance, adapter, device, queue) = init_wgpu();
        let window_canvas = WindowCanvas::create_for_window(
            &instance,
            &adapter,
            &device,
            window.clone(),
            |color_format| wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: color_format,
                view_formats: vec![color_format],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width: window.inner_size().width,
                height: window.inner_size().height,
                desired_maximum_frame_latency: 3,
                present_mode: wgpu::PresentMode::AutoVsync,
            },
        );
        let canvas_format = window_canvas.format();

        let rect_renderer = RectRenderer::create(&device, resources, canvas_format).unwrap();
        let rect_background = rect_renderer.create_rect(&device);
        rect_background.set_fill_color(&queue, Theme::DEFAULT.primary_background());
        rect_background.set_parameters(&queue, BoundingBox::new(-1., -1., 2., 2.), 0.);

        let font = Font::load_from_path(resources, "fonts/big_blue_terminal.json").unwrap();
        let text_renderer =
            TextRenderer::create(&device, &queue, font, resources, canvas_format).unwrap();

        let create_button = |button_renderer: &ButtonRenderer<_>, i: usize, title: &str| {
            let i = i as f32;
            let width = 64.;
            let height = 24.;
            let inter_padding = 10.;
            let bounds = BoundingBox::new(20. + i * (width + inter_padding), 20., width, height);
            button_renderer.create_button(&device, &queue, bounds, title, None)
        };
        let button_renderer_mundane = ButtonRenderer::new(
            weak_self.clone(),
            text_renderer.clone(),
            rect_renderer.clone(),
            Theme::DEFAULT.button_style_set(ButtonKind::Mundane),
        );
        let button_renderer_primary =
            button_renderer_mundane.fork(Theme::DEFAULT.button_style_set(ButtonKind::Primary));
        let button_renderer_toxic =
            button_renderer_mundane.fork(Theme::DEFAULT.button_style_set(ButtonKind::Toxic));
        let button_mundane = create_button(&button_renderer_mundane, 0, "Cancel");
        let button_primary = create_button(&button_renderer_primary, 1, "OK");
        let button_toxic = create_button(&button_renderer_toxic, 2, "Delete");

        let mut self_ = Self {
            resources,
            device,
            queue,
            window,
            window_canvas,
            text_renderer,
            rect_renderer,
            rect_background,
            button_renderer_mundane,
            button_mundane,
            button_renderer_primary,
            button_primary,
            button_renderer_toxic,
            button_toxic,
        };
        self_.window_resized();
        Mutex::new(self_)
    }

    fn frame(&mut self, canvas: CanvasView) {
        assert!(
            canvas.depth_stencil_texture_view.is_none(),
            "TODO: drawing with depth stencil buffer"
        );
        let mut encoder = self.device.create_command_encoder(&the_default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &canvas.color_texture_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
                resolve_target: None,
            })],
            ..the_default()
        });

        let projection = canvas.projection(ProjectionSpace::TopLeftDown, -1.0, 1.0);

        // Draw background rect.
        self.rect_renderer
            .draw_rect(&mut render_pass, &self.rect_background);

        // Draw button.
        self.button_mundane.set_projection(&self.queue, projection);
        self.button_renderer_mundane
            .prepare_button_for_drawing(&self.queue, &self.button_mundane);
        self.button_renderer_mundane
            .draw_button(&mut render_pass, &self.button_mundane);

        self.button_primary.set_projection(&self.queue, projection);
        self.button_renderer_primary
            .prepare_button_for_drawing(&self.queue, &self.button_primary);
        self.button_renderer_primary
            .draw_button(&mut render_pass, &self.button_primary);

        self.button_toxic.set_projection(&self.queue, projection);
        self.button_renderer_toxic
            .prepare_button_for_drawing(&self.queue, &self.button_toxic);
        self.button_renderer_toxic
            .draw_button(&mut render_pass, &self.button_toxic);

        drop(render_pass);

        self.queue.submit([encoder.finish()]);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(_) => self.window_resized(),
            WindowEvent::RedrawRequested => {
                let canvas_view = self.window_canvas.begin_drawing().unwrap();
                self.frame(canvas_view);
                self.window.pre_present_notify();
                self.window_canvas.finish_drawing().unwrap();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn window_resized(&mut self) {
        self.window_canvas.reconfigure_for_size(
            &self.device,
            self.window.inner_size(),
            self.window.scale_factor(),
            None,
        );
    }
}
