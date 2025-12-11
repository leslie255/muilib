use std::fmt::{self, Debug};

use cgmath::*;

use crate::utils::*;

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


