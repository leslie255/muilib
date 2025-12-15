use crate::{
    Axis, Bounds, CanvasRef, RectSize, RenderPass, Subview,
    UiContext, View, axis_utils::*,
};

use bumpalo::{Bump, collections::Vec as BumpVec};
use cgmath::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackPaddingType {
    /// Pad only between the subviews.
    Interpadded,
    /// Pad between the subviews, before the first subview, and after the last subview.
    Omnipadded,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum StackAlignment {
    #[default]
    Center,
    Leading,
    Trailing,
    Ratio(f32),
}

impl StackAlignment {
    fn ratio(self) -> f32 {
        match self {
            StackAlignment::Center => 0.5,
            StackAlignment::Leading => 0.0,
            StackAlignment::Trailing => 1.0,
            StackAlignment::Ratio(ratio) => ratio,
        }
    }
}

pub struct Stack<'pass, 'views, 'cx, UiState> {
    axis: Axis,
    alignment: StackAlignment,
    padding_type: StackPaddingType,
    fixed_padding: Option<f32>,
    subviews: BumpVec<'pass, Subview<'views, 'cx, UiState>>,
    /// Sum of the alphas of subviews.
    ///
    /// For the lingo "a", "b", "alpha", "beta", see `axis_utils`.
    alpha_sum: f32,
    /// Max among the betas of subviews.
    ///
    /// For the lingo "a", "b", "alpha", "beta", see `axis_utils`.
    beta_max: f32,
}

impl<'pass, 'views, 'cx, UiState> Stack<'pass, 'views, 'cx, UiState> {
    pub(crate) fn new(bump: &'pass Bump, axis: Axis) -> Self {
        Self {
            axis,
            alignment: StackAlignment::Center,
            padding_type: StackPaddingType::Interpadded,
            fixed_padding: None,
            subviews: BumpVec::new_in(bump),
            alpha_sum: 0.,
            beta_max: 0.,
        }
    }

    pub(crate) fn subview(&mut self, subview: &'views mut (dyn View<'cx, UiState> + 'views)) {
        // For the lingo "a", "b", "alpha", "beta", see `axis_utils`.
        let subview_size = subview.preferred_size();
        let subview_beta = subview_size.beta(self.axis);
        self.beta_max = self.beta_max.max(subview_beta);
        self.alpha_sum += subview_size.alpha(self.axis);
        self.subviews.push(Subview {
            preferred_size: subview_size,
            view: subview,
        });
    }

    pub(crate) fn set_alignment(&mut self, alignment: StackAlignment) {
        self.alignment = alignment;
    }

    pub(crate) fn set_padding_type(&mut self, padding_type: StackPaddingType) {
        self.padding_type = padding_type;
    }

    pub(crate) fn set_fixed_padding(&mut self, fixed_padding: impl Into<Option<f32>>) {
        self.fixed_padding = fixed_padding.into();
    }

    fn n_paddings(n_subviews: usize, padding_type: StackPaddingType) -> usize {
        match padding_type {
            StackPaddingType::Interpadded => n_subviews.saturating_sub(1),
            StackPaddingType::Omnipadded => n_subviews + 1,
        }
    }
}

impl<'pass, 'views, 'cx, UiState> View<'cx, UiState> for Stack<'pass, 'views, 'cx, UiState> {
    fn preferred_size(&mut self) -> RectSize<f32> {
        let n_paddings = Self::n_paddings(self.subviews.len(), self.padding_type) as f32;
        RectSize::new_on_axis(
            self.axis,
            self.alpha_sum + n_paddings * self.fixed_padding.unwrap_or(0.),
            self.beta_max,
        )
    }

    fn apply_bounds(&mut self, bounds: Bounds<f32>) {
        // For the lingo "a", "b", "alpha", "beta", see `axis_utils`.

        let n_paddings = Self::n_paddings(self.subviews.len(), self.padding_type) as f32;

        let min_alpha = self.alpha_sum + n_paddings * self.fixed_padding.unwrap_or(0.);
        let squeeze_a = (bounds.alpha(self.axis) / min_alpha).min(1.);
        let padding = match self.fixed_padding {
            Some(fixed_padding) => fixed_padding * squeeze_a,
            None => {
                let leftover_alpha = bounds.alpha(self.axis) - min_alpha;
                leftover_alpha.max(0.) / n_paddings
            }
        };

        // Accumulator for A-axis offset while we iterate through the subviews.
        let mut offset_a = 0.0f32;
        for (i, subview) in self.subviews.iter_mut().enumerate() {
            if i != 0 || self.padding_type == StackPaddingType::Omnipadded {
                // This accumulation cannot be moved to end of iteration to eliminate the if
                // condition, because `remaining_size` uses offset_a later.
                offset_a += padding;
            }
            let requested_size = subview
                .preferred_size
                .scaled_on_axis(self.axis, squeeze_a, 1.);
            let remaining_size = RectSize::new_on_axis(
                self.axis,
                bounds.alpha(self.axis) - offset_a,
                bounds.beta(self.axis),
            );
            let subview_size = requested_size.min(remaining_size);
            let leftover_beta = bounds.beta(self.axis) - subview_size.beta(self.axis);
            let offset_b = self.alignment.ratio() * leftover_beta;
            let subview_bounds = Bounds::new(
                bounds.origin + Vector2::new_on_axis(self.axis, offset_a, offset_b),
                subview_size,
            );
            subview.apply_bounds(subview_bounds);
            offset_a += subview_size.alpha(self.axis);
        }
    }

    fn prepare_for_drawing(&mut self, ui_context: &UiContext<'cx, UiState>, canvas: &CanvasRef) {
        for subview in &mut self.subviews {
            subview.prepare_for_drawing(ui_context, canvas);
        }
    }

    fn draw(&self, ui_context: &UiContext<'cx, UiState>, render_pass: &mut RenderPass) {
        for subview in &self.subviews {
            subview.draw(ui_context, render_pass);
        }
    }
}

pub struct StackBuilder<'pass, 'views, 'cx, UiState> {
    stack: Stack<'pass, 'views, 'cx, UiState>,
}

impl<'pass, 'views, 'cx, UiState> StackBuilder<'pass, 'views, 'cx, UiState> {
    pub(crate) fn new(bump: &'pass Bump, axis: Axis) -> Self {
        Self {
            stack: Stack::new(bump, axis),
        }
    }

    pub fn subview(&mut self, subview: &'views mut (dyn View<'cx, UiState> + 'views)) {
        self.stack.subview(subview);
    }

    pub fn set_alignment(&mut self, alignment: StackAlignment) {
        self.stack.set_alignment(alignment);
    }

    pub fn set_padding_type(&mut self, padding_type: StackPaddingType) {
        self.stack.set_padding_type(padding_type);
    }

    pub fn set_fixed_padding(&mut self, fixed_padding: impl Into<Option<f32>>) {
        self.stack.set_fixed_padding(fixed_padding);
    }

    pub(crate) fn finish(self) -> Stack<'pass, 'views, 'cx, UiState> {
        self.stack
    }
}
