use std::marker::PhantomData;

use cgmath::*;

use crate::{
    element::{Bounds, RectSize},
    param_getters_setters,
    view::{ControlFlow, View, ViewContext, ViewList},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    #[default]
    Center,
    /// Left for horizontal stacks; up for vertical stacks.
    Leading,
    /// Right for horizontal stacks; down for vertical stacks.
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackLayoutMethod {
    // ```
    // |......[VIEW] [VIEW] [VIEW]......|
    // ```
    Packed {
        direction: LayoutDirection,
        padding: f32,
    },
    // ```
    // | [..VIEW..] [..VIEW..] [..VIEW..] |
    // ```
    DistributeByStretching {
        padding: f32,
    },
    // ```
    // |...[VIEW]...[VIEW]...[VIEW]...|
    // ```
    DistributeByPadding,
}

pub struct HStackView<'cx, Subviews: ViewList<'cx>> {
    subviews: Subviews,
    subview_sizes: Vec<RectSize>,
    inter_padding: f32,
    _marker: PhantomData<&'cx ()>,
}

impl<'cx, Subviews: ViewList<'cx>> HStackView<'cx, Subviews> {
    pub fn new(subviews: Subviews) -> Self {
        Self {
            subviews,
            subview_sizes: Vec::new(),
            inter_padding: 0.0f32,
            _marker: PhantomData,
        }
    }

    param_getters_setters! {
        vis: pub,
        param_ty: f32,
        param: inter_padding,
        param_mut: inter_padding_mut,
        set_param: set_inter_padding,
        with_param: with_inter_padding,
        param_mut_preamble: |_: &mut Self| (),
    }

    pub fn subviews(&self) -> &Subviews {
        &self.subviews
    }

    pub fn subviews_mut(&mut self) -> &mut Subviews {
        &mut self.subviews
    }
}

impl<'cx, Subviews: ViewList<'cx>> View<Subviews::UiState> for HStackView<'cx, Subviews> {
    fn preferred_size(&mut self) -> RectSize {
        let mut size = RectSize::new(0., 0.);
        self.subview_sizes.clear();
        let mut is_first = true;
        self.subviews.for_each_subview_mut(|subview| {
            let subview_size = subview.preferred_size();
            size.height = size.height.max(subview_size.height);
            size.width += size.width;
            if !is_first {
                size.width += self.inter_padding;
            }
            is_first = false;
            self.subview_sizes.push(subview_size);
            ControlFlow::Continue
        });
        size
    }

    fn apply_bounds(&mut self, bounds: Bounds) {
        let mut subview_sizes = self.subview_sizes.iter();
        let mut offset_counter = bounds.x_min();
        let mut is_first = true;
        self.subviews.for_each_subview_mut(|subview| {
            let Some(&subview_size) = subview_sizes.next() else {
                log::warn!("`HStack::apply_bounds` encountered mismatched view list from the previous `preferred_size`");
                return ControlFlow::Break;
            };
            is_first = false;
            let top_padding = 0.5 * (bounds.height() - subview_size.height);
            let bounds = Bounds::new(
                point2(offset_counter, bounds.y_min() + top_padding),
                subview_size,
            );
            subview.apply_bounds(bounds);
            offset_counter += subview_size.width;
            if !is_first {
                offset_counter += self.inter_padding;
            }
            ControlFlow::Continue
        });
    }

    fn prepare_for_drawing(
        &mut self,
        view_context: &ViewContext<Subviews::UiState>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        canvas: &crate::wgpu_utils::CanvasView,
    ) {
        self.subviews.for_each_subview_mut(|subview| {
            subview.prepare_for_drawing(view_context, device, queue, canvas);
            ControlFlow::Continue
        });
    }

    fn draw(
        &self,
        view_context: &ViewContext<Subviews::UiState>,
        render_pass: &mut wgpu::RenderPass,
    ) {
        self.subviews.for_each_subview(|subview| {
            subview.draw(view_context, render_pass);
            ControlFlow::Continue
        });
    }
}
