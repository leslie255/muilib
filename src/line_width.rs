use derive_more::From;

use crate::RectSize;

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


