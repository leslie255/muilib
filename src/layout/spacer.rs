use crate::{RectSize, View};

pub struct Spacer {
    size: RectSize<f32>,
}

impl Spacer {
    pub(crate) fn new(size: RectSize<f32>) -> Self {
        Self { size }
    }
}

impl<'cx> View<'cx> for Spacer {
    fn preferred_size(&mut self) -> RectSize<f32> {
        self.size
    }

    fn apply_bounds(&mut self, bounds: crate::Bounds<f32>) {
        _ = bounds;
    }

    fn prepare_for_drawing(&mut self, ui_context: &crate::UiContext<'cx>, canvas: &crate::CanvasRef) {
        _ = ui_context;
        _ = canvas;
    }

    fn draw(&self, ui_context: &crate::UiContext<'cx>, render_pass: &mut crate::RenderPass) {
        _ = ui_context;
        _ = render_pass;
    }
}
