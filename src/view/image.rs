use std::cell::OnceCell;

use crate::{
    element::{Bounds, ImageElement, RectSize, Texture2d},
    param_getters_setters,
    utils::*,
    view::{View, ViewContext},
    wgpu_utils::CanvasView,
};

#[derive(Debug, Clone)]
pub struct ImageView {
    size: RectSize,
    bounds: Bounds,
    bounds_updated: bool,
    texture: Texture2d,
    raw: OnceCell<ImageElement>,
}

impl ImageView {
    pub fn new(texture: Texture2d) -> Self {
        Self {
            size: texture.size(),
            bounds: the_default(),
            bounds_updated: false,
            texture,
            raw: OnceCell::new(),
        }
    }

    param_getters_setters! {
        vis: pub,
        param_ty: RectSize,
        param: size,
        param_mut: size_mut,
        set_param: set_size,
        with_param: with_size,
        param_mut_preamble: |_: &mut Self| {},
    }
}

impl<UiState> View<UiState> for ImageView {
    fn preferred_size(&mut self) -> RectSize {
        self.size
    }

    fn apply_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
        self.bounds_updated = true
    }

    fn prepare_for_drawing(
        &mut self,
        view_context: &ViewContext<UiState>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        canvas: &CanvasView,
    ) {
        self.raw.get_or_init(|| {
            view_context
                .image_renderer()
                .create_image(device, &self.texture)
        });
        let raw = self.raw.get().unwrap();
        raw.set_projection(queue, canvas.projection);
        if self.bounds_updated {
            self.bounds_updated = false;
            raw.set_parameters(queue, self.bounds);
        }
    }

    fn draw(&self, view_context: &ViewContext<UiState>, render_pass: &mut wgpu::RenderPass) {
        let raw = self.raw.get().unwrap();
        view_context.image_renderer().draw_image(render_pass, raw);
    }
}
