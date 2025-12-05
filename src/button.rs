use std::{
    fmt::Debug,
    sync::{
        Mutex, Weak,
        atomic::{self, AtomicBool},
    },
};

use cgmath::*;
use winit::event::MouseButton;

use crate::{
    shapes::{BoundingBox, LineWidth, Rect, RectRenderer, Text, TextRenderer},
    wgpu_utils::{Srgb, Srgba},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    Hovered,
    Clicked,
}

/// Button style for all `ButtonState`s.
#[derive(Debug, Clone, Copy)]
pub struct ButtonStyle {
    pub idle_style: ButtonStateStyle,
    pub hovered_style: ButtonStateStyle,
    pub clicked_style: ButtonStateStyle,
}

impl ButtonStyle {
    pub const fn style_for(&self, state: ButtonState) -> ButtonStateStyle {
        match state {
            ButtonState::Idle => self.idle_style,
            ButtonState::Hovered => self.hovered_style,
            ButtonState::Clicked => self.clicked_style,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ButtonRenderer<'cx, UiState: 'cx> {
    ui_state: Weak<Mutex<UiState>>,
    text_renderer: TextRenderer<'cx>,
    rect_renderer: RectRenderer<'cx>,
    style: ButtonStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonStateStyle {
    pub line_width: LineWidth,
    pub font_size: f32,
    pub text: Srgb,
    pub fill: Srgb,
    pub line: Srgb,
}

impl<'cx, UiState: 'cx> ButtonRenderer<'cx, UiState> {
    pub fn new(
        ui_state: Weak<Mutex<UiState>>,
        text_renderer: TextRenderer<'cx>,
        rect_renderer: RectRenderer<'cx>,
        style: ButtonStyle,
    ) -> Self {
        Self {
            ui_state,
            text_renderer,
            rect_renderer,
            style,
        }
    }

    /// Create a new `ButtonRenderer` that has a different `ButtonStyle`.
    pub fn fork(&self, style: ButtonStyle) -> Self {
        Self {
            ui_state: self.ui_state.clone(),
            text_renderer: self.text_renderer.clone(),
            rect_renderer: self.rect_renderer.clone(),
            style,
        }
    }

    pub fn create_button(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounding_box: BoundingBox,
        title: &str,
        callback: Option<ButtonCallback<UiState>>,
    ) -> Button<UiState> {
        let rect = self.rect_renderer.create_rect(device);
        let text = self.text_renderer.create_text(device, title);
        assert!(callback.is_none(), "TODO: button events");
        let button = Button {
            title_len: title.len(),
            bounding_box,
            state: ButtonState::Idle,
            needs_updating: true.into(),
            rect,
            text,
            ui_state: Weak::clone(&self.ui_state),
            f_click: callback,
        };
        self.update_state_style(queue, &button);
        button
    }

    pub fn prepare_button_for_drawing(&self, queue: &wgpu::Queue, button: &Button<UiState>) {
        let style_needs_updating = button
            .needs_updating
            .fetch_and(false, atomic::Ordering::Relaxed);
        if style_needs_updating {
            self.update_state_style(queue, button);
        }
    }

    pub fn draw_button(&self, render_pass: &mut wgpu::RenderPass, button: &Button<UiState>) {
        self.rect_renderer.draw_rect(render_pass, &button.rect);
        self.text_renderer.draw_text(render_pass, &button.text);
    }

    fn update_state_style(&self, queue: &wgpu::Queue, button: &Button<UiState>) {
        let style = self.style.style_for(button.state);
        button.rect.set_fill_color(queue, style.fill);
        button.rect.set_line_color(queue, style.line);
        button
            .rect
            .set_parameters(queue, button.bounding_box, style.line_width);
        button.text.set_fg_color(queue, Srgb::from_hex(0xFFFFFF));
        button.text.set_bg_color(queue, Srgba::from_hex(0x00000000));
        // Assuming text is single-line.
        let text_height = style.font_size;
        let text_width = (button.title_len as f32)
            * self.text_renderer.font().glyph_relative_height()
            * text_height;
        let top_padding = 0.5 * (button.bounding_box.size.height - text_height);
        let left_padding = 0.5 * (button.bounding_box.size.width - text_width);
        let text_origin = point2(
            button.bounding_box.x_min() + left_padding,
            button.bounding_box.y_min() + top_padding,
        );
        button
            .text
            .set_parameters(queue, text_origin, style.font_size);
    }
}

pub type ButtonCallback<UiState> =
    Box<dyn for<'a> FnMut(&'a mut UiState, ButtonEvent<'a, UiState>)>;

pub struct Button<UiState> {
    title_len: usize,
    bounding_box: BoundingBox,
    state: ButtonState,
    /// Flag for when GPU-side things needs updating because state, bounding box, or something else
    /// have changed.
    needs_updating: AtomicBool,
    rect: Rect,
    text: Text,
    /// For callbacks.
    ui_state: Weak<Mutex<UiState>>,
    f_click: Option<ButtonCallback<UiState>>,
}

impl<UiState> Button<UiState> {
    pub fn set_projection(&self, queue: &wgpu::Queue, projection: Matrix4<f32>) {
        self.rect.set_projection(queue, projection);
        self.text.set_projection(queue, projection);
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEventKind {
    HoveringStart,
    HoveringFinish,
    ButtonDown { button: MouseButton },
    ButtonUp { button: MouseButton, inside: bool },
}

#[derive(Clone, Copy)]
pub struct ButtonEvent<'a, UiState> {
    kind: ButtonEventKind,
    button: &'a Button<UiState>,
    mouse_position: Point2<f32>,
}
