use derive_more::{AsMut, AsRef, Deref, DerefMut};

use crate::{
    Bounds, CanvasRef, RectSize, RectView, RenderPass, Rgba, UiContext, View, computed_property,
    property,
};

/// An empty view for just leaving a bit of space empty.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct SpacerView {
    size: RectSize<f32>,
}

impl SpacerView {
    pub const fn new(size: RectSize<f32>) -> Self {
        Self { size }
    }

    property! {
        vis: pub,
        param_ty: RectSize<f32>,
        param: size,
        param_mut: size_mut,
        set_param: set_size,
        with_param: with_size,
        param_mut_preamble: |_: &mut Self| {},
    }
}

impl<'cx, UiState: 'cx> View<'cx, UiState> for SpacerView {
    fn preferred_size(&mut self) -> RectSize<f32> {
        self.size
    }

    fn apply_bounds(&mut self, _bounds: Bounds<f32>) {}

    fn prepare_for_drawing(&mut self, _view_context: &UiContext<UiState>, _canvas: &CanvasRef) {}

    fn draw(&self, _view_context: &UiContext<UiState>, _render_pass: &mut RenderPass) {}
}

pub trait ViewExt<'cx, UiState: 'cx>: View<'cx, UiState> + Sized {
    fn into_box_dyn_view(self) -> Box<dyn View<'cx, UiState>> {
        Box::new(self)
    }

    fn into_spread_view(self, axis: SpreadAxis) -> SpreadView<Self> {
        SpreadView::new(axis, self)
    }

    fn into_container_view(self) -> ContainerView<Self> {
        ContainerView::new(self)
    }
}

impl<'cx, UiState: 'cx, T: View<'cx, UiState>> ViewExt<'cx, UiState> for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadAxis {
    Horizontal,
    Vertical,
    Both,
}

/// Makes the view take as much space as possible in one axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRef, AsMut, Deref, DerefMut)]
pub struct SpreadView<Subview> {
    axis: SpreadAxis,
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    subview: Subview,
}

impl<'cx, UiState, Subview> View<'cx, UiState> for SpreadView<Subview>
where
    UiState: 'cx,
    Subview: View<'cx, UiState>,
{
    fn preferred_size(&mut self) -> RectSize<f32> {
        let subview_size = self.subview.preferred_size();
        match self.axis {
            SpreadAxis::Horizontal => RectSize::new(f32::INFINITY, subview_size.height),
            SpreadAxis::Vertical => RectSize::new(subview_size.width, f32::INFINITY),
            SpreadAxis::Both => RectSize::new(f32::INFINITY, f32::INFINITY),
        }
    }

    fn apply_bounds(&mut self, bounds: Bounds<f32>) {
        self.subview.apply_bounds(bounds)
    }

    fn prepare_for_drawing(&mut self, ui_context: &UiContext<'cx, UiState>, canvas: &CanvasRef) {
        self.subview.prepare_for_drawing(ui_context, canvas)
    }

    fn draw(&self, ui_context: &UiContext<'cx, UiState>, render_pass: &mut RenderPass) {
        self.subview.draw(ui_context, render_pass)
    }
}

impl<Subview> SpreadView<Subview> {
    pub fn new(axis: SpreadAxis, subview: Subview) -> Self {
        Self { axis, subview }
    }

    pub fn horizontal(subview: Subview) -> Self {
        Self::new(SpreadAxis::Horizontal, subview)
    }

    pub fn vertical(subview: Subview) -> Self {
        Self::new(SpreadAxis::Vertical, subview)
    }

    pub fn both(subview: Subview) -> Self {
        Self::new(SpreadAxis::Both, subview)
    }

    property! {
        vis: pub,
        param_ty: SpreadAxis,
        param: axis,
        param_mut: axis_mut,
        set_param: set_axis,
        with_param: with_axis,
        param_mut_preamble: |_: &mut Self| {},
    }

    pub fn into_subview(self) -> Subview {
        self.subview
    }

    pub const fn subview(&self) -> &Subview {
        &self.subview
    }

    pub const fn subview_mut(&mut self) -> &mut Subview {
        &mut self.subview
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerPadding {
    /// Fixed padding.
    Fixed(f32),
    /// As ratio of the view's size on that axis.
    ///
    /// FIXME: RatioOfViewSize does not squeeze properly when not enough space is provided.
    RatioOfViewSize(f32),
    /// Take the rest of the remaining space.
    ///
    /// If both edges of an axis are `Spread`, then the view is positioned somewhere in the center.
    /// The position of the view in this situation is determined by `spread_ratio_{vertical|horizontal}`, as follows:
    ///
    /// - `padding_left = spread_ratio_horizontal * (availible_width - subview_width)`
    /// - `padding_top = spread_ratio_vertical * (availible_height - subview_height)`
    /// - `padding_right = (1.0 - spread_ratio_horizontal) * (availible_width - subview_width)`
    /// - `padding_bottom = (1.0 - spread_ratio_vertical) * (availible_height - subview_height)`
    ///
    /// This means that, if paddings on both edges of axis are `Spread`, and that spread ratio of
    /// that axis is `0.5`, then the view is centered on that axis.
    Spread,
}

impl ContainerPadding {
    fn as_fixed(&self) -> Option<f32> {
        if let &Self::Fixed(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Default for ContainerPadding {
    fn default() -> Self {
        Self::Fixed(0.)
    }
}

#[derive(Debug, AsRef, AsMut, Deref, DerefMut)]
pub struct ContainerView<Subview> {
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    subview: Subview,
    padding_left: ContainerPadding,
    padding_right: ContainerPadding,
    padding_top: ContainerPadding,
    padding_bottom: ContainerPadding,
    spread_ratio_horizontal: f32,
    spread_ratio_vertical: f32,
    subview_size: Option<RectSize<f32>>,
    background_rect: RectView,
}

impl<Subview> ContainerView<Subview> {
    pub fn new(subview: Subview) -> Self {
        Self {
            subview,
            padding_left: ContainerPadding::Fixed(0.),
            padding_right: ContainerPadding::Fixed(0.),
            padding_top: ContainerPadding::Fixed(0.),
            padding_bottom: ContainerPadding::Fixed(0.),
            spread_ratio_horizontal: 0.5,
            spread_ratio_vertical: 0.5,
            subview_size: None,
            background_rect: RectView::new(RectSize::new(0., 0.))
                .with_fill_color(Rgba::new(0., 0., 0., 0.)),
        }
    }

    pub fn subview(&self) -> &Subview {
        &self.subview
    }

    pub fn subview_mut(&mut self) -> &mut Subview {
        &mut self.subview
    }

    property! {
        vis: pub,
        param_ty: ContainerPadding,
        param: padding_left,
        param_mut: padding_left_mut,
        set_param: set_padding_left,
        with_param: with_padding_left,
        param_mut_preamble: |_: &mut Self| {},
    }

    property! {
        vis: pub,
        param_ty: ContainerPadding,
        param: padding_right,
        param_mut: padding_right_mut,
        set_param: set_padding_right,
        with_param: with_padding_right,
        param_mut_preamble: |_: &mut Self| {},
    }

    property! {
        vis: pub,
        param_ty: ContainerPadding,
        param: padding_top,
        param_mut: padding_top_mut,
        set_param: set_padding_top,
        with_param: with_padding_top,
        param_mut_preamble: |_: &mut Self| {},
    }

    property! {
        vis: pub,
        param_ty: ContainerPadding,
        param: padding_bottom,
        param_mut: padding_bottom_mut,
        set_param: set_padding_bottom,
        with_param: with_padding_bottom,
        param_mut_preamble: |_: &mut Self| {},
    }

    property! {
        vis: pub,
        param_ty: f32,
        param: spread_ratio_horizontal,
        param_mut: spread_ratio_horizontal_mut,
        set_param: set_spread_ratio_horizontal,
        with_param: with_spread_ratio_horizontal,
        param_mut_preamble: |_: &mut Self| {},
    }

    property! {
        vis: pub,
        param_ty: f32,
        param: spread_ratio_vertical,
        param_mut: spread_ratio_vertical_mut,
        set_param: set_spread_ratio_vertical,
        with_param: with_spread_ratio_vertical,
        param_mut_preamble: |_: &mut Self| {},
    }

    computed_property! {
        vis: pub,
        param_ty: ContainerPadding,
        param: padding_bottom,
        set_param: set_padding,
        with_param: with_padding,
        fset: |self_: &mut Self, padding| {
            self_.set_padding_left(padding);
            self_.set_padding_right(padding);
            self_.set_padding_top(padding);
            self_.set_padding_bottom(padding);
        },
    }

    computed_property! {
        vis: pub,
        param_ty: Rgba,
        param: background_color,
        set_param: set_background_color,
        with_param: with_background_color,
        fget: |self_: &Self| self_.background_rect.fill_color(),
        fset: |self_: &mut Self, background_color: Rgba| self_.background_rect.set_fill_color(background_color),
    }

    fn padding(
        padding_leading: ContainerPadding,
        padding_trailing: ContainerPadding,
        spread_ratio: f32,
        view_length: f32,
        remaining_length: f32,
    ) -> (f32, f32) {
        use ContainerPadding::*;
        let padding = |padding: ContainerPadding| match padding {
            Fixed(fixed) => fixed,
            RatioOfViewSize(ratio) => ratio * view_length,
            Spread => spread_ratio * remaining_length,
        };
        match (padding_leading, padding_trailing) {
            (Spread, Spread) => (
                spread_ratio * remaining_length,
                spread_ratio * remaining_length,
            ),
            (leading, Spread) => {
                let padding_leading = padding(leading);
                (padding_leading, (remaining_length - padding_leading))
            }
            (Spread, trailing) => {
                let padding_trailing = padding(trailing);
                ((remaining_length - padding_trailing), padding_trailing)
            }
            (leading, trailing) => (padding(leading), padding(trailing)),
        }
    }
}

impl<'cx, UiState, Subview> View<'cx, UiState> for ContainerView<Subview>
where
    UiState: 'cx,
    Subview: View<'cx, UiState>,
{
    fn preferred_size(&mut self) -> RectSize<f32> {
        let subview_size = self.subview.preferred_size();
        self.subview_size = Some(subview_size);
        let (padding_left, padding_right) = Self::padding(
            self.padding_left,
            self.padding_right,
            self.spread_ratio_horizontal,
            subview_size.width,
            f32::INFINITY,
        );
        let (padding_top, padding_bottom) = Self::padding(
            self.padding_top,
            self.padding_bottom,
            self.spread_ratio_vertical,
            subview_size.height,
            f32::INFINITY,
        );
        RectSize {
            width: padding_left + subview_size.width + padding_right,
            height: padding_top + subview_size.height + padding_bottom,
        }
    }

    fn apply_bounds(&mut self, bounds: Bounds<f32>) {
        self.background_rect.apply_bounds_(bounds);
        let requested_size = self.subview_size.unwrap_or_else(|| {
            log::warn!(
                "PaddedView::apply_bounds called without a prior PaddedView::preferred_size"
            );
            self.subview.preferred_size()
        });
        let max_subview_size = RectSize {
            width: (bounds.width()
                - self.padding_left.as_fixed().unwrap_or(0.)
                - self.padding_right.as_fixed().unwrap_or(0.)),
            height: (bounds.height()
                - self.padding_top.as_fixed().unwrap_or(0.)
                - self.padding_bottom.as_fixed().unwrap_or(0.)),
        }
        .max(RectSize::new(0., 0.));
        let subview_size = requested_size.min(max_subview_size);
        let (padding_left, padding_right) = Self::padding(
            self.padding_left,
            self.padding_right,
            self.spread_ratio_horizontal,
            subview_size.width,
            (bounds.width() - subview_size.width).max(0.),
        );
        let (padding_top, padding_bottom) = Self::padding(
            self.padding_top,
            self.padding_bottom,
            self.spread_ratio_vertical,
            subview_size.height,
            (bounds.height() - subview_size.height).max(0.),
        );
        let padded_size = RectSize {
            width: padding_left + subview_size.width + padding_right,
            height: padding_top + subview_size.height + padding_bottom,
        };
        let squeeze_horizontal = (bounds.width() / padded_size.width).min(1.);
        let squeeze_vertical = (bounds.height() / padded_size.height).min(1.);
        let mut subview_bounds = Bounds::from_scalars(
            bounds.x_min() + padding_left,
            bounds.y_min() + padding_top,
            subview_size.width * squeeze_horizontal,
            subview_size.height * squeeze_vertical,
        );
        if subview_bounds.x_max() > bounds.x_max() {
            subview_bounds.size.width = (bounds.x_max() - subview_bounds.x_min()).max(0.);
        }
        if subview_bounds.y_max() > bounds.y_max() {
            subview_bounds.size.height = (bounds.y_max() - subview_bounds.y_min()).max(0.);
        }
        self.subview.apply_bounds(subview_bounds);
    }

    fn prepare_for_drawing(&mut self, ui_context: &UiContext<'cx, UiState>, canvas: &CanvasRef) {
        if self.background_color().a != 0. {
            self.background_rect.prepare_for_drawing(ui_context, canvas);
        }
        self.subview.prepare_for_drawing(ui_context, canvas);
    }

    fn draw(&self, ui_context: &UiContext<'cx, UiState>, render_pass: &mut RenderPass) {
        if self.background_color().a != 0. {
            self.background_rect.draw(ui_context, render_pass);
        }
        self.subview.draw(ui_context, render_pass);
    }
}
