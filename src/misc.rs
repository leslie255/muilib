use std::{
    fmt::{self, Debug},
    ops::Add,
};

use bytemuck::{Pod, Zeroable};
use cgmath::*;
use derive_more::From;

use crate::utils::*;

#[derive(Debug, Clone, Copy, From)]
pub enum LineWidth {
    /// All borders have the same line width.
    Uniform(f32),
    /// Borders have different line widths.
    PerBorder {
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
}

impl Default for LineWidth {
    fn default() -> Self {
        Self::Uniform(0.)
    }
}

impl LineWidth {
    pub const fn to_array(self) -> [f32; 4] {
        match self {
            Self::Uniform(width) => [width, width, width, width],
            Self::PerBorder {
                left,
                top,
                right,
                bottom,
            } => [left, top, right, bottom],
        }
    }

    pub const fn normalized_in(self, size: RectSize<f32>) -> Self {
        let [left, top, right, bottom] = self.to_array();
        Self::PerBorder {
            left: left / size.width,
            top: top / size.height,
            right: right / size.width,
            bottom: bottom / size.height,
        }
    }

    pub const fn left(&self) -> f32 {
        match self {
            LineWidth::Uniform(width) => *width,
            LineWidth::PerBorder { left, .. } => *left,
        }
    }

    pub const fn top(&self) -> f32 {
        match self {
            LineWidth::Uniform(width) => *width,
            LineWidth::PerBorder { top, .. } => *top,
        }
    }

    pub const fn right(&self) -> f32 {
        match self {
            LineWidth::Uniform(width) => *width,
            LineWidth::PerBorder { right, .. } => *right,
        }
    }

    pub const fn bottom(&self) -> f32 {
        match self {
            LineWidth::Uniform(width) => *width,
            LineWidth::PerBorder { bottom, .. } => *bottom,
        }
    }

    pub const fn set_left(&mut self, left_width: f32) {
        let [_, top, right, bottom] = self.to_array();
        *self = Self::PerBorder {
            left: left_width,
            top,
            right,
            bottom,
        };
    }

    pub const fn set_top(&mut self, top_width: f32) {
        let [left, _, right, bottom] = self.to_array();
        *self = Self::PerBorder {
            left,
            top: top_width,
            right,
            bottom,
        };
    }

    pub const fn set_right(&mut self, right_width: f32) {
        let [left, top, _, bottom] = self.to_array();
        *self = Self::PerBorder {
            left,
            top,
            right: right_width,
            bottom,
        };
    }

    pub const fn set_bottom(&mut self, bottom_width: f32) {
        let [left, top, right, _] = self.to_array();
        *self = Self::PerBorder {
            left,
            top,
            right,
            bottom: bottom_width,
        };
    }
}

impl From<[f32; 4]> for LineWidth {
    fn from([left, top, right, bottom]: [f32; 4]) -> Self {
        Self::PerBorder {
            left,
            top,
            right,
            bottom,
        }
    }
}

pub fn linear_to_srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        linear.powf(1. / 2.4) * 1.055 - 0.055
    }
}

pub fn srgb_to_linear(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear RGBA.
#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_hex(u: u32) -> Self {
        let [r, g, b, a] = u.to_be_bytes();
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: a as f32 / 255.,
        }
    }

    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<Rgba> for [f32; 4] {
    fn from(srgba: Rgba) -> Self {
        srgba.to_array()
    }
}

impl From<[f32; 4]> for Rgba {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Srgba> for Rgba {
    fn from(s: Srgba) -> Self {
        Self::new(
            srgb_to_linear(s.r),
            srgb_to_linear(s.g),
            srgb_to_linear(s.b),
            s.a,
        )
    }
}

impl From<Srgb> for Rgba {
    fn from(s: Srgb) -> Self {
        Self::new(
            srgb_to_linear(s.r),
            srgb_to_linear(s.g),
            srgb_to_linear(s.b),
            1.0,
        )
    }
}

/// sRGB+A.
#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Srgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Srgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_hex(u: u32) -> Self {
        let [r, g, b, a] = u.to_be_bytes();
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: a as f32 / 255.,
        }
    }

    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<Srgba> for [f32; 4] {
    fn from(srgba: Srgba) -> Self {
        srgba.to_array()
    }
}

impl From<[f32; 4]> for Srgba {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Rgba> for Srgba {
    fn from(linear: Rgba) -> Self {
        Self::new(
            linear_to_srgb(linear.r),
            linear_to_srgb(linear.g),
            linear_to_srgb(linear.b),
            linear.a,
        )
    }
}

/// sRGB.
#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Srgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Srgb {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub const fn from_hex(u: u32) -> Self {
        let [zero, r, g, b] = u.to_be_bytes();
        assert!(zero == 0, "`Srgb::from_hex` called with overflowing value");
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
        }
    }

    pub const fn to_array(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<Srgb> for [f32; 3] {
    fn from(srgba: Srgb) -> Self {
        srgba.to_array()
    }
}

impl From<[f32; 3]> for Srgb {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Self { r, g, b }
    }
}

impl From<Srgb> for Srgba {
    fn from(s: Srgb) -> Self {
        Self::new(s.r, s.g, s.b, 1.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Bounds<T: Copy> {
    pub origin: Point2<T>,
    pub size: RectSize<T>,
}

impl<T: Copy + Debug> Debug for Bounds<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Bounds")
            .field("x_min", &self.x_min())
            .field("y_min", &self.y_min())
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

impl<T: Copy> Default for Bounds<T>
where
    T: Zero,
{
    fn default() -> Self {
        Self {
            origin: point2(T::zero(), T::zero()),
            size: the_default(),
        }
    }
}

impl<T: Copy> Bounds<T> {
    pub const fn new(origin: Point2<T>, size: RectSize<T>) -> Self {
        Self { origin, size }
    }

    pub const fn from_scalars(x_min: T, y_min: T, width: T, height: T) -> Self {
        Self {
            origin: point2(x_min, y_min),
            size: RectSize::new(width, height),
        }
    }

    pub const fn x_min(self) -> T {
        self.origin.x
    }

    pub const fn y_min(self) -> T {
        self.origin.y
    }

    pub const fn width(self) -> T {
        self.size.width
    }

    pub const fn height(self) -> T {
        self.size.height
    }

    pub const fn with_origin(self, origin: Point2<T>) -> Self {
        Self { origin, ..self }
    }

    pub const fn with_size(self, size: RectSize<T>) -> Self {
        Self { size, ..self }
    }

    pub fn x_max(self) -> T
    where
        T: Add<T, Output = T>,
    {
        self.origin.x + self.size.width
    }

    pub fn y_max(self) -> T
    where
        T: Add<T, Output = T>,
    {
        self.origin.y + self.size.height
    }
}

impl Bounds<f32> {
    /// `const fn` versions of `x_max`, downside is it is `f32`-only.
    pub const fn x_max_(self) -> f32 {
        self.origin.x + self.size.width
    }

    /// `const fn` versions of `x_max`, downside is it is `f32`-only.
    pub const fn y_max_(self) -> f32 {
        self.origin.y + self.size.height
    }

    pub const fn contains(self, point: Point2<f32>) -> bool {
        self.x_min() <= point.x
            && point.x <= self.x_max_()
            && self.y_min() <= point.y
            && point.y <= self.y_max_()
    }

    pub const fn with_inset(self, padding: f32) -> Self {
        Self::from_scalars(
            self.x_min() + padding,
            self.y_min() + padding,
            (self.width() - padding - padding).max(0.),
            (self.height() - padding - padding).max(0.),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectSize<T: Copy> {
    pub width: T,
    pub height: T,
}

impl<T: Copy> Default for RectSize<T>
where
    T: Zero,
{
    fn default() -> Self {
        Self {
            width: T::zero(),
            height: T::zero(),
        }
    }
}

impl<T: Copy> RectSize<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub const fn as_vec(self) -> Vector2<T> {
        vec2(self.width, self.height)
    }
}

impl RectSize<f32> {
    /// `min` per-axis.
    pub fn min(self, other: Self) -> Self {
        Self {
            width: self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }

    /// `min` per-axis.
    pub fn max(self, other: Self) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }

    pub const fn scaled(self, scale_horizontal: f32, scale_vertical: f32) -> Self {
        Self {
            width: self.width * scale_horizontal,
            height: self.height * scale_vertical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Extension traits for parametric-over-axis functions for `Point2`, `Vector2`, `RectSize`, and
/// `Bounds`.
///
/// They are useful for, for instance, stack layout calculation, where the horizontal and vertical
/// stack layout can now share the same code.
///
/// In this module, `a` and `alpha` refers to the provided axis, `b` and `beta` refers to the other
/// axis. The latin letters `a` and `b` corresponds to `x` and `y` in the normal lingo, their greek
/// counterparts `alpha` and `beta` corresponds to `width` and `height`.
/// For example, if `axis == Axis::Horizontal`, then:
/// - `a == x`
/// - `b == y`
/// - `alpha == width`
/// - `beta == height`
pub(crate) mod axis_utils {
    #![allow(dead_code)]
    use std::ops::{Add, Mul};

    use cgmath::*;

    use crate::{Axis, Bounds, RectSize};

    pub(crate) trait Point2AxisExt<T: Copy> {
        fn new_on_axis(axis: Axis, a: T, b: T) -> Self;
        fn a(self, axis: Axis) -> T;
        fn b(self, axis: Axis) -> T;
        fn a_mut(&mut self, axis: Axis) -> &mut T;
        fn b_mut(&mut self, axis: Axis) -> &mut T;
    }

    impl<T: Copy> Point2AxisExt<T> for Point2<T> {
        fn new_on_axis(axis: Axis, a: T, b: T) -> Self {
            match axis {
                Axis::Horizontal => Self::new(a, b),
                Axis::Vertical => Self::new(b, a),
            }
        }

        fn a(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.x,
                Axis::Vertical => self.y,
            }
        }

        fn b(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.y,
                Axis::Vertical => self.x,
            }
        }

        fn a_mut(&mut self, axis: Axis) -> &mut T {
            match axis {
                Axis::Horizontal => &mut self.x,
                Axis::Vertical => &mut self.y,
            }
        }

        fn b_mut(&mut self, axis: Axis) -> &mut T {
            match axis {
                Axis::Horizontal => &mut self.y,
                Axis::Vertical => &mut self.x,
            }
        }
    }

    pub(crate) trait Vector2AxisExt<T: Copy> {
        fn new_on_axis(axis: Axis, a: T, b: T) -> Self;
        fn a(self, axis: Axis) -> T;
        fn b(self, axis: Axis) -> T;
        fn a_mut(&mut self, axis: Axis) -> &mut T;
        fn b_mut(&mut self, axis: Axis) -> &mut T;
    }

    impl<T: Copy> Vector2AxisExt<T> for Vector2<T> {
        fn new_on_axis(axis: Axis, a: T, b: T) -> Self {
            match axis {
                Axis::Horizontal => Self::new(a, b),
                Axis::Vertical => Self::new(b, a),
            }
        }

        fn a(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.x,
                Axis::Vertical => self.y,
            }
        }

        fn b(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.y,
                Axis::Vertical => self.x,
            }
        }

        fn a_mut(&mut self, axis: Axis) -> &mut T {
            match axis {
                Axis::Horizontal => &mut self.x,
                Axis::Vertical => &mut self.y,
            }
        }

        fn b_mut(&mut self, axis: Axis) -> &mut T {
            match axis {
                Axis::Horizontal => &mut self.y,
                Axis::Vertical => &mut self.x,
            }
        }
    }

    pub(crate) trait RectSizeAxisExt<T: Copy> {
        fn new_on_axis(axis: Axis, alpha: T, beta: T) -> Self;

        fn alpha(self, axis: Axis) -> T;

        fn beta(self, axis: Axis) -> T;

        fn alpha_mut(&mut self, axis: Axis) -> &mut T;

        fn beta_mut(&mut self, axis: Axis) -> &mut T;

        fn scaled_on_axis(self, axis: Axis, scale_a: T, scale_b: T) -> Self
        where
            T: Mul<T, Output = T>;
    }

    impl<T: Copy> RectSizeAxisExt<T> for RectSize<T> {
        fn new_on_axis(axis: Axis, alpha: T, beta: T) -> Self {
            match axis {
                Axis::Horizontal => Self::new(alpha, beta),
                Axis::Vertical => Self::new(beta, alpha),
            }
        }

        fn alpha(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.width,
                Axis::Vertical => self.height,
            }
        }

        fn beta(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.height,
                Axis::Vertical => self.width,
            }
        }

        fn alpha_mut(&mut self, axis: Axis) -> &mut T {
            match axis {
                Axis::Horizontal => &mut self.width,
                Axis::Vertical => &mut self.height,
            }
        }

        fn beta_mut(&mut self, axis: Axis) -> &mut T {
            match axis {
                Axis::Horizontal => &mut self.height,
                Axis::Vertical => &mut self.width,
            }
        }

        fn scaled_on_axis(mut self, axis: Axis, scale_a: T, scale_b: T) -> Self
        where
            T: Mul<T, Output = T>,
        {
            let alpha_mut = self.alpha_mut(axis);
            *alpha_mut = (*alpha_mut) * scale_a;
            let beta_mut = self.beta_mut(axis);
            *beta_mut = (*beta_mut) * scale_b;
            self
        }
    }

    pub(crate) trait BoundsAxisExt<T: Copy> {
        fn a_min(self, axis: Axis) -> T;

        fn b_min(self, axis: Axis) -> T;

        fn a_max(self, axis: Axis) -> T
        where
            T: Add<T, Output = T>;

        fn b_max(self, axis: Axis) -> T
        where
            T: Add<T, Output = T>;

        fn alpha(self, axis: Axis) -> T;

        fn beta(self, axis: Axis) -> T;
    }

    impl<T: Copy> BoundsAxisExt<T> for Bounds<T> {
        fn a_min(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.x_min(),
                Axis::Vertical => self.y_min(),
            }
        }

        fn b_min(self, axis: Axis) -> T {
            match axis {
                Axis::Horizontal => self.y_min(),
                Axis::Vertical => self.x_min(),
            }
        }

        fn a_max(self, axis: Axis) -> T
        where
            T: Add<T, Output = T>,
        {
            match axis {
                Axis::Horizontal => self.x_max(),
                Axis::Vertical => self.y_max(),
            }
        }

        fn b_max(self, axis: Axis) -> T
        where
            T: Add<T, Output = T>,
        {
            match axis {
                Axis::Horizontal => self.y_max(),
                Axis::Vertical => self.x_max(),
            }
        }

        fn alpha(self, axis: Axis) -> T {
            self.size.alpha(axis)
        }

        fn beta(self, axis: Axis) -> T {
            self.size.beta(axis)
        }
    }
}
