use std::{borrow::Cow, cell::OnceCell};

use cgmath::*;

use crate::{
    Bounds, CanvasRef, Font, RectSize, RenderPass, Rgba, UiContext, View, element::TextElement,
    property,
};

#[derive(Debug)]
pub struct TextView<'cx> {
    n_lines: usize,
    n_columns: usize,
    text: Cow<'cx, str>,
    font_size: f32,
    font: Font<'cx>,
    fg_color: Rgba,
    bg_color: Rgba,
    origin: Point2<f32>,
    needs_update: bool,
    squeeze_horizontal: f32,
    squeeze_vertical: f32,
    text_needs_update: bool,
    raw: OnceCell<TextElement>,
}

impl<'cx> TextView<'cx> {
    pub fn new(ui_context: &UiContext<'cx>) -> Self {
        Self {
            n_lines: 1,
            n_columns: 0,
            text: "".into(),
            font_size: 12.,
            font: ui_context.text_renderer().font(),
            fg_color: Rgba::from_hex(0xFFFFFFFF),
            bg_color: Rgba::from_hex(0x00000000),
            origin: point2(0., 0.),
            needs_update: false,
            text_needs_update: false,
            squeeze_horizontal: 1.,
            squeeze_vertical: 1.,
            raw: OnceCell::new(),
        }
    }

    property! {
        vis: pub,
        param_ty: f32,
        param: font_size,
        param_mut: font_size_mut,
        set_param: set_font_size,
        with_param: with_font_size,
        param_mut_preamble: |self_: &mut Self| self_.needs_update = true,
    }

    property! {
        vis: pub,
        param_ty: Rgba,
        param: fg_color,
        param_mut: fg_color_mut,
        set_param: set_fg_color,
        with_param: with_fg_color,
        param_mut_preamble: |self_: &mut Self| self_.needs_update = true,
    }

    property! {
        vis: pub,
        param_ty: Rgba,
        param: bg_color,
        param_mut: bg_color_mut,
        set_param: set_bg_color,
        with_param: with_bg_color,
        param_mut_preamble: |self_: &mut Self| self_.needs_update = true,
    }

    pub fn set_text(&mut self, text: impl Into<Cow<'cx, str>>) {
        self.text_needs_update = true;
        let text = text.into();
        self.n_lines = 1usize;
        let mut n_columns = 0usize;
        self.n_columns = 0;
        self.n_lines = 1;
        for char in text.chars() {
            match char {
                '\n' => {
                    self.n_lines += 1;
                    self.n_columns = self.n_columns.max(n_columns);
                    n_columns = 0;
                }
                '\r' => {
                    self.n_columns = self.n_columns.max(n_columns);
                    n_columns = 0;
                }
                _ => {
                    n_columns += 1;
                    self.n_columns = self.n_columns.max(n_columns)
                }
            }
        }
        self.text = text;
    }

    pub fn with_text(mut self, text: impl Into<Cow<'cx, str>>) -> Self {
        self.set_text(text);
        self
    }

    pub fn n_columns(&self) -> usize {
        self.n_columns
    }

    pub fn n_lines(&self) -> usize {
        self.n_lines
    }

    pub fn size(&self) -> RectSize<f32> {
        RectSize::new(
            (self.n_columns as f32) * self.font.glyph_relative_width() * self.font_size(),
            self.n_lines as f32 * self.font_size(),
        )
    }
}

impl<'cx> View<'cx> for TextView<'cx> {
    fn preferred_size(&mut self) -> RectSize<f32> {
        self.size()
    }

    fn apply_bounds(&mut self, bounds: Bounds<f32>) {
        let size = self.size();
        self.squeeze_horizontal = (bounds.width() / size.width).min(1.);
        self.squeeze_vertical = (bounds.height() / size.height).min(1.);
        self.needs_update = true;
        self.origin = bounds.origin;
    }

    fn prepare_for_drawing(&mut self, ui_context: &UiContext<'cx>, _canvas: &CanvasRef) {
        let raw = self.raw.get_or_init(|| {
            self.text_needs_update = false; // `create_text` updates the text
            ui_context
                .text_renderer()
                .create_text(ui_context.wgpu_device(), &self.text)
        });
        if self.needs_update {
            self.needs_update = false;
            let this = &raw;
            let origin = self.origin;
            let font_size = self.font_size;
            this.set_model_view(
                ui_context.wgpu_queue(),
                Matrix4::from_translation(origin.to_vec().extend(0.))
                    * Matrix4::from_nonuniform_scale(
                        self.squeeze_horizontal * font_size,
                        self.squeeze_vertical * font_size,
                        1.,
                    ),
            );
            raw.set_fg_color(ui_context.wgpu_queue(), self.fg_color);
            raw.set_bg_color(ui_context.wgpu_queue(), self.bg_color);
        }
        if self.text_needs_update {
            self.text_needs_update = false;
            let raw = self.raw.get_mut().unwrap();
            ui_context
                .text_renderer()
                .update_text(ui_context.wgpu_device(), raw, &self.text);
        }
    }

    fn draw(&self, ui_context: &UiContext<'cx>, render_pass: &mut RenderPass) {
        ui_context
            .text_renderer()
            .draw_text(render_pass.wgpu_render_pass(), self.raw.get().unwrap());
    }
}
