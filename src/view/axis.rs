#![allow(dead_code)]

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
