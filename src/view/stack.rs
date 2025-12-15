use std::marker::PhantomData;

use cgmath::*;

use crate::{
    Axis, Bounds, BoundsAxisExt as _, CanvasRef, Point2AxisExt as _, RectSize,
    RectSizeAxisExt as _, RectView, RenderPass, Rgba, View, ViewList, computed_property, property,
};

use super::{ControlFlow, UiContext};

pub struct StackView<'cx, Subviews: ViewList<'cx>> {
    axis: Axis,
    subviews: Subviews,
    alignment: StackAlignment,
    padding_type: StackPaddingType,
    fixed_padding: Option<f32>,
    subview_sizes: Vec<RectSize<f32>>,
    size: RectSize<f32>,
    subview_length_alpha_total: f32,
    background_rect: RectView,
    _marker: PhantomData<&'cx ()>,
}

impl<'cx, Subviews: ViewList<'cx>> StackView<'cx, Subviews> {
    pub fn new(axis: Axis, subviews: Subviews) -> Self {
        Self {
            axis,
            subviews,
            alignment: StackAlignment::Center,
            padding_type: StackPaddingType::Interpadded,
            fixed_padding: None,
            subview_sizes: Vec::new(),
            size: RectSize::new(0., 0.),
            subview_length_alpha_total: 0.0f32,
            background_rect: RectView::new(RectSize::new(0., 0.))
                .with_fill_color(Rgba::new(0., 0., 0., 0.)),
            _marker: PhantomData,
        }
    }

    pub fn horizontal(subviews: Subviews) -> Self {
        Self::new(Axis::Horizontal, subviews)
    }

    pub fn vertical(subviews: Subviews) -> Self {
        Self::new(Axis::Vertical, subviews)
    }

    property! {
        vis: pub,
        param_ty: Axis,
        param: axis,
        param_mut: axis_mut,
        set_param: set_axis,
        with_param: with_axis,
        param_mut_preamble: |_: &mut Self| (),
    }

    property! {
        vis: pub,
        param_ty: StackAlignment,
        param: alignment,
        param_mut: alignment_mut,
        set_param: set_alignment,
        with_param: with_alignment,
        param_mut_preamble: |_: &mut Self| (),
    }

    property! {
        vis: pub,
        param_ty: StackPaddingType,
        param: padding_type,
        param_mut: padding_type_mut,
        set_param: set_padding_type,
        with_param: with_padding_type,
        param_mut_preamble: |_: &mut Self| (),
    }

    property! {
        vis: pub,
        param_ty: Option<f32>,
        param: fixed_padding,
        param_mut: fixed_padding_mut,
        set_param: set_fixed_padding,
        with_param: with_fixed_padding,
        param_mut_preamble: |_: &mut Self| (),
    }

    computed_property! {
        vis: pub,
        param_ty: Rgba,
        param: background_color,
        set_param: set_background_color,
        with_param: with_background_color,
        fget: |self_: &Self| self_.background_rect.fill_color(),
        fset: |self_: &mut Self, background_color| self_.background_rect.set_fill_color(background_color),
    }

    fn warn_n_subviews_changed() {
        log::warn!(
            "`StackView::apply_bounds` called, but number of subviews have changed since `StackView::preferred_size`"
        );
    }
}

impl<'cx, Subviews: ViewList<'cx>> View<'cx, Subviews::UiState> for StackView<'cx, Subviews> {
    fn preferred_size(&mut self) -> RectSize<f32> {
        let mut length_alpha = 0.0f32;
        let mut length_beta = 0.0f32;
        self.subview_sizes.clear();
        self.subviews.for_each_subview_mut(|subview| {
            let subview_size = subview.preferred_size();
            self.subview_sizes.push(subview_size);
            length_alpha += subview_size.length_alpha(self.axis);
            length_beta = length_beta.max(subview_size.length_beta(self.axis));
            ControlFlow::Continue
        });
        let n_subviews = self.subview_sizes.len();
        let n_paddings = match self.padding_type() {
            StackPaddingType::Interpadded => n_subviews.saturating_sub(1),
            StackPaddingType::Omnipadded => n_subviews + 1,
        };
        self.subview_length_alpha_total = length_alpha;
        let padding_total = (n_paddings as f32) * self.fixed_padding.unwrap_or(0.);
        length_alpha += padding_total;
        self.size = RectSize::new_on_axis(self.axis, length_alpha, length_beta);
        self.size
    }

    fn apply_bounds(&mut self, bounds: Bounds<f32>) {
        self.background_rect.apply_bounds_(bounds);
        let squeeze = (bounds.length_alpha(self.axis) / self.size.length_alpha(self.axis)).min(1.);
        let mut subview_sizes = self.subview_sizes.iter();
        let n_subviews = self.subview_sizes.len();
        let n_paddings = match self.padding_type {
            StackPaddingType::Interpadded => n_subviews.saturating_sub(1),
            StackPaddingType::Omnipadded => n_subviews + 1,
        };
        let padding = match self.fixed_padding {
            Some(fixed_padding) => fixed_padding,
            None => ((bounds.length_alpha(self.axis) - self.subview_length_alpha_total)
                / (n_paddings as f32))
                .max(0.),
        } * squeeze;
        let mut offset_alpha = match self.padding_type {
            StackPaddingType::Interpadded => bounds.alpha_min(self.axis) + 0.,
            StackPaddingType::Omnipadded => bounds.alpha_min(self.axis) + padding,
        };
        self.subviews.for_each_subview_mut(|subview| {
            let Some(&(mut requested_size)) = subview_sizes.next() else {
                Self::warn_n_subviews_changed();
                return ControlFlow::Break;
            };
            *requested_size.length_alpha_mut(self.axis) *= squeeze;
            let remaining_size = RectSize::new_on_axis(
                self.axis, //
                bounds.length_alpha(self.axis) - offset_alpha + bounds.alpha_min(self.axis),
                bounds.length_beta(self.axis),
            );
            let subview_size = requested_size.min(remaining_size);
            let offset_beta = bounds.beta_min(self.axis)
                + self.alignment.ratio()
                    * (bounds.length_beta(self.axis) - subview_size.length_beta(self.axis));
            let subview_bounds = Bounds::new(
                Point2::new_on_axis(self.axis, offset_alpha, offset_beta),
                subview_size,
            );
            subview.apply_bounds(subview_bounds);
            offset_alpha += padding;
            offset_alpha += subview_size.length_alpha(self.axis);
            ControlFlow::Continue
        });
    }

    fn prepare_for_drawing(
        &mut self,
        ui_context: &UiContext<'cx, Subviews::UiState>,
        canvas: &CanvasRef,
    ) {
        if self.background_rect.fill_color().a != 0. {
            self.background_rect.prepare_for_drawing(ui_context, canvas);
        }
        self.subviews.for_each_subview_mut(|subview| {
            subview.prepare_for_drawing(ui_context, canvas);
            ControlFlow::Continue
        });
    }

    fn draw(&self, ui_context: &UiContext<'cx, Subviews::UiState>, render_pass: &mut RenderPass) {
        if self.background_rect.fill_color().a != 0. {
            self.background_rect.draw(ui_context, render_pass);
        }
        self.subviews.for_each_subview(|subview| {
            subview.draw(ui_context, render_pass);
            ControlFlow::Continue
        });
    }
}

pub struct ZStackView<'cx, Subviews: ViewList<'cx>> {
    subviews: Subviews,
    alignment_horizontal: StackAlignment,
    alignment_vertical: StackAlignment,
    subview_sizes: Vec<RectSize<f32>>,
    size: Option<RectSize<f32>>,
    _marker: PhantomData<&'cx ()>,
}

impl<'cx, Subviews: ViewList<'cx>> ZStackView<'cx, Subviews> {
    pub fn new(subviews: Subviews) -> Self {
        Self {
            subviews,
            alignment_horizontal: StackAlignment::Center,
            alignment_vertical: StackAlignment::Center,
            subview_sizes: Vec::new(),
            size: None,
            _marker: PhantomData,
        }
    }

    property! {
        vis: pub,
        param_ty: StackAlignment,
        param: alignment_horizontal,
        param_mut: alignment_horizontal_mut,
        set_param: set_alignment_horizontal,
        with_param: with_alignment_horizontal,
        param_mut_preamble: |_: &mut Self| (),
    }

    property! {
        vis: pub,
        param_ty: StackAlignment,
        param: alignment_vertical,
        param_mut: alignment_vertical_mut,
        set_param: set_alignment_vertical,
        with_param: with_alignment_vertical,
        param_mut_preamble: |_: &mut Self| (),
    }

    fn warn_mismatched_n_subview() {
        log::warn!(
            "ZStackView::apply_bounds internal error: mismatched number of subview_sizes and subviews"
        );
    }
}

impl<'cx, Subviews: ViewList<'cx>> View<'cx, Subviews::UiState> for ZStackView<'cx, Subviews> {
    fn preferred_size(&mut self) -> RectSize<f32> {
        let mut size = RectSize::new(0., 0.);
        self.subview_sizes.clear();
        self.subviews.for_each_subview_mut(|subview| {
            let subview_size = subview.preferred_size();
            size = size.max(subview_size);
            self.subview_sizes.push(subview_size);
            ControlFlow::Continue
        });
        self.size = Some(size);
        size
    }

    fn apply_bounds(&mut self, bounds: Bounds<f32>) {
        let size = self.size.unwrap_or_else(|| {
            log::warn!(
                "ZStackView::apply_bounds called without a prior ZStackView::preferred_size"
            );
            self.preferred_size()
        });
        let squeeze_horizontal = (bounds.width() / size.width).min(1.);
        let squeeze_vertical = (bounds.height() / size.height).min(1.);
        let mut subview_sizes = self.subview_sizes.iter();
        self.subviews.for_each_subview_mut(|subview| {
            let Some(&subview_size_original) = subview_sizes.next() else {
                Self::warn_mismatched_n_subview();
                return ControlFlow::Break;
            };
            let subview_size = subview_size_original.scaled(squeeze_horizontal, squeeze_vertical);
            let padding = (bounds.size.as_vec() - subview_size.as_vec()).mul_element_wise(vec2(
                self.alignment_horizontal.ratio(),
                self.alignment_vertical.ratio(),
            ));
            subview.apply_bounds(Bounds::new(bounds.origin + padding, subview_size));
            ControlFlow::Continue
        });
    }

    fn prepare_for_drawing(
        &mut self,
        ui_context: &UiContext<'cx, Subviews::UiState>,
        canvas: &CanvasRef,
    ) {
        self.subviews.for_each_subview_mut(|subview| {
            subview.prepare_for_drawing(ui_context, canvas);
            ControlFlow::Continue
        });
    }

    fn draw(&self, ui_context: &UiContext<'cx, Subviews::UiState>, render_pass: &mut RenderPass) {
        self.subviews.for_each_subview(|subview| {
            subview.draw(ui_context, render_pass);
            ControlFlow::Continue
        });
    }
}
