use std::fmt::{self, Debug};

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
}

impl Bounds<f32> {
    pub const fn x_max(self) -> f32 {
        self.origin.x + self.size.width
    }

    pub const fn y_max(self) -> f32 {
        self.origin.y + self.size.height
    }

    pub const fn xy_max(self) -> Point2<f32> {
        point2(self.x_max(), self.y_max())
    }

    pub const fn xy_min(self) -> Point2<f32> {
        self.origin
    }

    pub const fn contains(self, point: Point2<f32>) -> bool {
        self.x_min() <= point.x
            && point.x <= self.x_max()
            && self.y_min() <= point.y
            && point.y <= self.y_max()
    }

    pub const fn with_padding(self, padding: f32) -> Self {
        Self::from_scalars(
            self.x_min() + padding,
            self.y_min() + padding,
            self.width() - padding - padding,
            self.height() - padding - padding,
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
