use std::{ops::Range, path::Path};

use cgmath::*;
use serde::{Deserialize, Serialize};

use crate::{AppResources, Bounds, ImageRef, LoadResourceError, RectSize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetaJson {
    pub path: String,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub glyph_width: u32,
    pub glyph_height: u32,
    pub present_start: u8,
    pub present_end: u8,
    pub glyphs_per_line: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Font<'cx> {
    present_start: u8,
    present_end: u8,
    glyphs_per_line: u32,
    glyph_size: RectSize<u32>,
    glyph_size_uv: RectSize<f32>,
    atlas_image: ImageRef<'cx>,
}

impl<'cx> Font<'cx> {
    pub fn load_from_resources(
        resources: &'cx AppResources,
        json_subpath: impl AsRef<Path>,
    ) -> Result<Self, LoadResourceError> {
        let json_subpath = json_subpath.as_ref();
        let font_meta = resources.load_json_object::<FontMetaJson>(json_subpath)?;
        let atlas_image_subpath = resources.solve_relative_subpath(json_subpath, &font_meta.path);
        let atlas_image = resources.load_image(&atlas_image_subpath)?;
        Ok(Self {
            present_start: font_meta.present_start,
            present_end: font_meta.present_end,
            glyphs_per_line: font_meta.glyphs_per_line,
            glyph_size: RectSize::new(font_meta.glyph_width, font_meta.glyph_height),
            glyph_size_uv: RectSize::new(
                font_meta.glyph_width as f32 / atlas_image.width_f(),
                font_meta.glyph_height as f32 / atlas_image.height_f(),
            ),
            atlas_image,
        })
    }

    pub fn atlas_image(&self) -> ImageRef<'cx> {
        self.atlas_image
    }

    pub fn present_range(&self) -> Range<u8> {
        self.present_start..self.present_end
    }

    pub fn has_glyph(&self, char: char) -> bool {
        self.present_range().contains(&(char as u8))
    }

    fn uv_position_for_glyph(&self, char: char) -> Option<Point2<f32>> {
        if !self.has_glyph(char) {
            return None;
        }
        let i_glyph = ((char as u8) - self.present_start) as u32;
        let glyph_coord = point2(
            (i_glyph % self.glyphs_per_line) as f32 * self.glyph_size_uv.width,
            (i_glyph / self.glyphs_per_line) as f32 * self.glyph_size_uv.height,
        );
        Some(glyph_coord)
    }

    pub fn uv_bounds_for_char(&self, char: char) -> Option<Bounds<f32>> {
        let top_left = self.uv_position_for_glyph(char)?;
        Some(Bounds::new(top_left, self.glyph_size_uv))
    }

    /// Glyph width if glyph height is 1.
    pub fn glyph_relative_width(&self) -> f32 {
        (self.glyph_size.width as f32) / (self.glyph_size.height as f32)
    }

    pub fn glyph_size(&self) -> RectSize<u32> {
        self.glyph_size
    }

    pub fn glyph_size_uv(&self) -> RectSize<f32> {
        self.glyph_size_uv
    }
}
