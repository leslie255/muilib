use crate::{Bounds, CanvasRef, RectSize};

mod axis;
mod button;
mod image;
mod rect;
mod text;
mod ui_context;

pub use axis::*;
pub use button::*;
pub use image::*;
pub use rect::*;
pub use text::*;
pub use ui_context::*;

pub trait View<'cx, UiState: 'cx> {
    fn preferred_size(&mut self) -> RectSize<f32>;
    fn apply_bounds(&mut self, bounds: Bounds<f32>);
    fn prepare_for_drawing(&mut self, ui_context: &UiContext<'cx, UiState>, canvas: &CanvasRef);
    fn draw(&self, ui_context: &UiContext<'cx, UiState>, render_pass: &mut RenderPass);
}
