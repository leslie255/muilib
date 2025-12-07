use crate::{
    element::{Bounds, RectSize},
    wgpu_utils::CanvasView,
};

mod button;
mod rect;
mod stack;
mod text;
mod view_context;
mod abstract_views;

pub use button::*;
pub use rect::*;
pub use stack::*;
pub use text::*;
pub use view_context::*;
pub use abstract_views::*;

pub trait View<UiState> {
    fn preferred_size(&mut self) -> RectSize;
    fn apply_bounds(&mut self, bounds: Bounds);
    fn prepare_for_drawing(
        &mut self,
        view_context: &ViewContext<UiState>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        canvas: &CanvasView,
    );
    fn draw(&self, view_context: &ViewContext<UiState>, render_pass: &mut wgpu::RenderPass);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
    Break,
    Continue,
}

pub trait ViewList<'cx> {
    type UiState: 'cx;
    fn for_each_subview(
        &self,
        visitor: impl FnMut(&(dyn View<Self::UiState> + 'cx)) -> ControlFlow,
    );
    fn for_each_subview_mut(
        &mut self,
        visitor: impl FnMut(&mut (dyn View<Self::UiState> + 'cx)) -> ControlFlow,
    );
}

#[macro_export]
macro_rules! impl_view_list_ref {
    ( $self:expr, $visitor:expr $(,)? ) => {};
    ( $self:expr, $visitor:expr, $field:ident $(,$($tts:tt)*)? ) => {
        if $visitor(&$self.$field) == ControlFlow::Break { return; }
        $crate::impl_view_list_ref!($self, $visitor, $($($tts)*)?)
    };
    ( $self:expr, $visitor:expr, $field:ident(iter) $(,$($tts:tt)*)? ) => {
        for subview in &$self.$field {
            if $visitor(subview) == ControlFlow::Break {
                return;
            }
        }
        $crate::impl_view_list_ref!($self, $visitor, $($($tts)*)?)
    };
}

#[macro_export]
macro_rules! impl_view_list_mut {
    ( $self:expr, $visitor:expr $(,)? ) => {};
    ( $self:expr, $visitor:expr, $field:ident $(,$($tts:tt)*)? ) => {
        if $visitor(&mut $self.$field) == ControlFlow::Break { return; }
        $crate::impl_view_list_mut!($self, $visitor, $($($tts)*)?)
    };
    ( $self:expr, $visitor:expr, $field:ident(iter) $(,$($tts:tt)*)? ) => {
        for subview in &mut $self.$field {
            if $visitor(subview) == ControlFlow::Break {
                return;
            }
        }
        $crate::impl_view_list_mut!($self, $visitor, $($($tts)*)?)
    };
}

#[macro_export]
macro_rules! impl_view_list {
    ($cx:lifetime , $($fields:tt)*) => {
        fn for_each_subview(
            &self,
            mut visitor: impl FnMut(&(dyn View<Self::UiState> + $cx)) -> ControlFlow,
        ) {
            $crate::impl_view_list_ref!(self, visitor, $($fields)*);
        }
        fn for_each_subview_mut(
            &mut self,
            mut visitor: impl FnMut(&mut (dyn View<Self::UiState> + $cx)) -> ControlFlow,
        ) {
            $crate::impl_view_list_mut!(self, visitor, $($fields)*);
        }
    };
}
