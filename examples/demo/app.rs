use std::sync::Arc;

use muilib::{Canvas as _, RectSize, Srgb};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

use crate::theme::{ButtonKind, Theme};

pub struct App<'cx> {
    window: Arc<Window>,
    window_canvas: muilib::WindowCanvas<'static>,
    ui_context: muilib::UiContext<'cx>,
    button_increase: muilib::ButtonView<'cx, Self>,
    button_decrease: muilib::ButtonView<'cx, Self>,
    button_reset: muilib::ButtonView<'cx, Self>,
    toolbar_rect: muilib::RectView,
    rects: Vec<muilib::RectView>,
    event_router: Arc<muilib::EventRouter<'cx, Self>>,
}

impl<'cx> muilib::LazyApplicationHandler<&'cx muilib::AppResources> for App<'cx> {
    fn new(resources: &'cx muilib::AppResources, event_loop: &ActiveEventLoop) -> Self {
        let window = event_loop
            .create_window(WindowAttributes::default().with_title("UI Test"))
            .unwrap();
        Self::create(resources, window)
    }
}

impl<'cx> App<'cx> {
    pub fn create(resources: &'cx muilib::AppResources, window: Window) -> Self {
        let window = Arc::new(window);
        let event_router = Arc::new(muilib::EventRouter::new());
        let (ui_context, window_canvas) =
            muilib::UiContext::create_for_window(resources, window.clone())
                .unwrap_or_else(|e| panic!("{e}"));

        // let image = resources.load_image("images/pfp.png").unwrap();
        // let texture = ui_context.create_texture(image);

        let theme = Theme::DEFAULT;

        let colors = [0xC04040, 0x40C040, 0x4040C0, 0x008080, 0x808000, 0x800080];

        let mut self_ = Self {
            window,
            window_canvas,
            button_increase: muilib::ButtonView::new(&ui_context, &event_router)
                .with_style(theme.button_style(ButtonKind::Mundane))
                .with_title("+")
                .with_callback(Self::button_increase),
            button_decrease: muilib::ButtonView::new(&ui_context, &event_router)
                .with_style(theme.button_style(ButtonKind::Mundane))
                .with_title("-")
                .with_callback(Self::button_decrease),
            button_reset: muilib::ButtonView::new(&ui_context, &event_router)
                .with_style(theme.button_style(ButtonKind::Toxic))
                .with_title("Reset")
                .with_callback(Self::button_reset),
            toolbar_rect: muilib::RectView::new(RectSize::new(f32::INFINITY, 56.))
                .with_fill_color(theme.secondary_background()),
            rects: colors
                .into_iter()
                .map(|color| {
                    muilib::RectView::new(RectSize::new(100., 100.))
                        .with_fill_color(Srgb::from_hex(color))
                        .with_line_color(Srgb::from_hex(0xFFFFFF))
                        .with_line_width(4.)
                })
                .collect(),
            ui_context,
            event_router,
        };
        self_.window_resized();
        self_
    }

    fn button_increase(&mut self, event: muilib::ButtonEvent) {
        if event.is_button_trigger() {
            for (i, rect) in self.rects.iter_mut().enumerate() {
                if i.is_multiple_of(2) {
                    continue;
                }
                let size = rect.size_mut();
                *size = size.scaled(1.1, 1.1);
            }
        }
    }

    fn button_decrease(&mut self, event: muilib::ButtonEvent) {
        if event.is_button_trigger() {
            for (i, rect) in self.rects.iter_mut().enumerate() {
                if i.is_multiple_of(2) {
                    continue;
                }
                let size = rect.size_mut();
                *size = size.scaled(1. / 1.1, 1. / 1.1);
            }
        }
    }

    fn button_reset(&mut self, event: muilib::ButtonEvent) {
        if event.is_button_trigger() {
            for (i, rect) in self.rects.iter_mut().enumerate() {
                if i.is_multiple_of(2) {
                    continue;
                }
                rect.set_size(RectSize::new(100., 100.));
            }
        }
    }

    fn frame(&mut self, canvas: muilib::CanvasRef) {
        let layout = self.ui_context.begin_layout_pass();

        let toolbar_hstack = layout.hstack(|hstack| {
            hstack.set_fixed_padding(4.);
            hstack.subview(&mut self.button_increase);
            hstack.subview(&mut self.button_decrease);
            hstack.subview(&mut self.button_reset);
            hstack.subview(layout.spacer(RectSize::new(f32::INFINITY, 0.)));
        });

        let main_body = layout.vstack(|vstack| {
            vstack.set_fixed_padding(4.);
            vstack.set_fixed_padding(4.);
            vstack.set_alignment_horizontal(muilib::StackAlignmentHorizontal::Center);
            let rows = self.rects.get_disjoint_mut([0..1, 1..3, 3..6]).unwrap();
            for row in rows {
                vstack.subview(layout.hstack(|hstack| {
                    hstack.set_fixed_padding(4.);
                    for rect in row {
                        hstack.subview(rect);
                    }
                }));
            }
        });

        let root_view = layout.vstack(|vstack| {
            vstack.set_fixed_padding(4.);
            vstack.set_alignment_vertical(muilib::StackAlignmentVertical::Top);
            vstack.set_alignment_horizontal(muilib::StackAlignmentHorizontal::Left);
            // Toolbar.
            vstack.subview(
                layout
                    .container(
                        layout
                            .container(toolbar_hstack)
                            .set_padding(muilib::ContainerPadding::Fixed(12.)),
                    )
                    .set_background_rect_view(&mut self.toolbar_rect),
            );
            // Main body.
            vstack.subview(
                layout
                    .container(
                        layout
                            .container(main_body)
                            .set_padding(muilib::ContainerPadding::Spread),
                    )
                    .set_padding(muilib::ContainerPadding::Fixed(12.))
                    .set_padding_top(muilib::ContainerPadding::Fixed(4.)),
            );
        });

        self.ui_context
            .prepare_view_bounded(&canvas, canvas.bounds(), root_view);

        let mut render_pass = self
            .ui_context
            .begin_render_pass(&canvas, Theme::DEFAULT.primary_background());

        self.ui_context.draw_view(&mut render_pass, root_view);
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

impl<'cx> ApplicationHandler for App<'cx> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match &event {
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
        let should_redraw = self.event_router.clone().window_event(&event, self);
        if should_redraw {
            self.window.request_redraw();
        }
    }
}
