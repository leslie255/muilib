use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

use crate::theme::{ButtonKind, Theme};

use uitest::{
    AppResources, Bounds, ButtonView, Canvas as _, CanvasRef, EventRouter, ImageView, RectSize,
    RectView, SpreadAxis, Srgb, Srgba, StackAlignment, StackPaddingType, StackView, TextView,
    UiContext, View, ViewExt as _, WindowCanvas, ZStackView, view_lists::*,
};

pub(crate) struct Application<'cx> {
    resources: &'cx AppResources,
    mouse_event_router: Arc<EventRouter<'cx, UiState<'cx>>>,
    window: Option<Arc<Window>>,
    ui: Option<UiState<'cx>>,
}

impl<'cx> Application<'cx> {
    pub fn new(resources: &'cx AppResources) -> Self {
        Self {
            resources,
            mouse_event_router: Arc::new(EventRouter::new(Bounds::default())),
            window: None,
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
                self.window = Some(Arc::clone(&window));
                *ui = Some(UiState::create(
                    self.resources,
                    window,
                    self.mouse_event_router.clone(),
                ));
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window_id != window.id() {
            return;
        }
        if let WindowEvent::Resized(size_physical) = event {
            let size_logical = size_physical.to_logical::<f32>(window.scale_factor());
            let bounds = Bounds::from_scalars(0., 0., size_logical.width, size_logical.height);
            self.mouse_event_router.set_bounds(bounds);
        }
        if let Some(ui) = self.ui.as_mut() {
            let should_redraw = self.mouse_event_router.window_event(&event, ui);
            if should_redraw {
                window.request_redraw();
            }
            ui.window_event(event_loop, window_id, event);
        }
    }
}

struct UiState<'cx> {
    window: Arc<Window>,
    window_canvas: WindowCanvas<'static>,
    ui_context: UiContext<'cx, Self>,
    root_view: Box<dyn View<'cx, Self>>,
}

impl<'cx> UiState<'cx> {
    pub fn create(
        resources: &'cx AppResources,
        window: Arc<Window>,
        event_router: Arc<EventRouter<'cx, Self>>,
    ) -> Self {
        let (ui_context, window_canvas) =
            UiContext::create_for_window(resources, Arc::clone(&window), event_router)
                .unwrap_or_else(|e| panic!("{e}"));

        let image = resources.load_image("images/pfp.png").unwrap();
        let texture = ui_context.create_texture(image);

        let mut self_ = Self {
            window,
            window_canvas,
            root_view: StackView::horizontal(ViewList1::new(
                StackView::horizontal(ViewList3::new(
                    StackView::vertical(ViewList2::new(
                        TextView::new(&ui_context).with_text("Button:"),
                        ButtonView::new(&ui_context)
                            .with_size(RectSize::new(128., 64.))
                            .with_style(
                                Theme::DEFAULT.button_style(ButtonKind::Primary).scaled(2.),
                            ),
                    ))
                    .with_alignment(StackAlignment::Leading)
                    .with_fixed_padding(4.)
                    .with_padding_type(StackPaddingType::Interpadded),
                    StackView::vertical(ViewList2::new(
                        TextView::new(&ui_context).with_text("ImageView:"),
                        ImageView::new(RectSize::new(100., 100.)).with_texture(texture.clone()),
                    ))
                    .with_alignment(StackAlignment::Leading)
                    .with_fixed_padding(4.)
                    .with_padding_type(StackPaddingType::Interpadded),
                    StackView::vertical(ViewList2::new(
                        TextView::new(&ui_context).with_text("ZStackView:"),
                        ZStackView::new(ViewList2::new(
                            ImageView::new(RectSize::new(100., 100.)).with_texture(texture.clone()),
                            RectView::new(RectSize::new(50., 50.))
                                .with_fill_color(Srgba::from_hex(0x80808080))
                                .with_line_color(Srgb::from_hex(0xFFFFFF))
                                .with_line_width(2.),
                        ))
                        .with_alignment_horizontal(StackAlignment::Ratio(0.2))
                        .with_alignment_vertical(StackAlignment::Ratio(0.2)),
                    ))
                    .with_alignment(StackAlignment::Leading)
                    .with_fixed_padding(4.)
                    .with_padding_type(StackPaddingType::Interpadded),
                ))
                .with_padding_type(StackPaddingType::Interpadded)
                .with_fixed_padding(10.),
            ))
            .with_padding_type(StackPaddingType::Omnipadded)
            .with_background_color(Theme::DEFAULT.tertiary_background())
            .into_ratio_container_view()
            .with_ratio_top(0.2)
            .with_ratio_left(0.2)
            .with_background_color(Theme::DEFAULT.secondary_background())
            .into_spread_view(SpreadAxis::Both)
            .into_container_view()
            .with_padding_top(20.)
            .with_padding_bottom(20.)
            .with_padding_left(80.)
            .with_padding_right(80.)
            .with_background_color(Theme::DEFAULT.primary_background())
            .into_box_dyn_view(),
            ui_context,
        };
        self_.window_resized();
        self_
    }

    fn frame(&mut self, canvas: CanvasRef) {
        let mut render_pass = self
            .ui_context
            .begin_render_pass(&canvas, Theme::DEFAULT.primary_background());

        self.ui_context
            .prepare_view_bounded(&canvas, canvas.bounds(), self.root_view.as_mut());
        self.ui_context
            .draw_view(&mut render_pass, self.root_view.as_ref());
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
                let canvas_view = self.window_canvas.create_ref().unwrap();
                self.frame(canvas_view);
                self.window.pre_present_notify();
                self.window_canvas.finish_drawing().unwrap();
                // self.window.request_redraw();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } if event.state.is_pressed() => {
                if event.logical_key == Key::Named(NamedKey::F5) {
                    self.window.request_redraw();
                }
            }
            _ => (),
        }
    }

    fn window_resized(&mut self) {
        self.window_canvas.reconfigure_for_size(
            self.ui_context.wgpu_device(),
            self.window.inner_size(),
            self.window.scale_factor(),
            None,
        );
    }
}
