use std::{
    array,
    collections::{HashMap, HashSet},
    fmt::{self, Debug},
    iter,
    sync::{Arc, Mutex, Weak},
};

use cgmath::*;

use winit::event::{MouseButton, WindowEvent};

use crate::Bounds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    HoveringStart,
    HoveringFinish,
    ButtonDown {
        button: MouseButton,
        /// `true` if the button is pressed when the cursor is inside the bounds.
        /// `false` if the button is pressed when the cursor is outside the bounds, and is only moved
        /// into the bounds now.
        started_inside: bool,
    },
    ButtonUp {
        button: MouseButton,
        inside: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub cursor_position: Point2<f32>,
}

impl MouseEvent {
    pub fn new(kind: MouseEventKind, cursor_position: Point2<f32>) -> Self {
        Self {
            kind,
            cursor_position,
        }
    }
}

pub trait MouseEventListener<UiState>: Send + Sync {
    fn mouse_event(&self, event: MouseEvent, ui_state: &mut UiState);
}

pub struct EventRouter<'cx, UiState> {
    inner: Mutex<EventRouterInner<'cx, UiState>>,
    dispatch: Arc<EventRouterDispatch>,
}

struct EventRouterInner<'cx, UiState> {
    /// `None` if we don't know the position of the cursor.
    cursor_position: Option<Point2<f32>>,
    scale_factor: f64,
    listeners: Vec<Option<Listener<'cx, UiState>>>,
    /// Track states of mouse buttons.
    /// `true` for pressed state.
    button_states: [bool; 5],
}

struct EventRouterDispatch {
    bounds_updates: Mutex<HashMap<usize, Bounds<f32>>>,
    /// List of objects to deregister.
    deregisters: Mutex<HashSet<usize>>,
}

impl<'cx, UiState> EventRouter<'cx, UiState> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(EventRouterInner {
                cursor_position: None,
                scale_factor: 1.0f64,
                listeners: Vec::new(),
                button_states: array::from_fn(|_| false),
            }),
            dispatch: Arc::new(EventRouterDispatch {
                bounds_updates: Mutex::new(HashMap::new()),
                deregisters: Mutex::new(HashSet::new()),
            }),
        }
    }

    pub fn register_listener(
        &self,
        bounds: Bounds<f32>,
        listener: impl MouseEventListener<UiState> + 'cx,
    ) -> ListenerHandle {
        let mut inner = self.inner.lock().unwrap();
        let listeners = &mut inner.listeners;
        let index = listeners.len();
        listeners.push(Some(Listener {
            bounds,
            is_hovered: false,
            button_states: array::from_fn(|_| false),
            object: Box::new(listener),
        }));
        ListenerHandle {
            router_dispatch: Arc::downgrade(&self.dispatch),
            index,
        }
    }

    fn listeners_iter_mut<'a>(
        listeners: &'a mut Vec<Option<Listener<'cx, UiState>>>,
    ) -> impl Iterator<Item = &'a mut Listener<'cx, UiState>> + use<'a, 'cx, UiState> {
        listeners.iter_mut().filter_map(Option::as_mut)
    }

    #[allow(dead_code)]
    fn listeners_iter<'a>(
        listeners: &'a Vec<Option<Listener<'cx, UiState>>>,
    ) -> impl Iterator<Item = &'a Listener<'cx, UiState>> + use<'a, 'cx, UiState> {
        listeners.iter().filter_map(Option::as_ref)
    }

    /// Returns if should request redraw.
    #[must_use = "Make sure to redraw if returns `true`"]
    pub fn window_event(&self, event: &WindowEvent, ui_state: &mut UiState) -> bool {
        match event {
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                let mut inner = self.inner.lock().unwrap();
                inner.scale_factor = *scale_factor;
                self.scan_events(ui_state, &mut inner)
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                let mut inner = self.inner.lock().unwrap();
                let position_logical = position.to_logical::<f32>(inner.scale_factor);
                let cursor_position = point2(position_logical.x, position_logical.y);
                inner.cursor_position = Some(cursor_position);
                self.scan_events(ui_state, &mut inner)
            }
            WindowEvent::CursorLeft { device_id: _ } => {
                let mut inner = self.inner.lock().unwrap();
                inner.cursor_position = None;
                self.scan_events(ui_state, &mut inner)
            }
            &WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let index = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Back => 3,
                    MouseButton::Forward => 4,
                    MouseButton::Other(_) => return false,
                };
                let mut inner = self.inner.lock().unwrap();
                inner.button_states[index] = state.is_pressed();
                self.scan_events(ui_state, &mut inner)
            }
            WindowEvent::RedrawRequested => {
                let mut inner = self.inner.lock().unwrap();
                self.scan_events(ui_state, &mut inner)
            }
            _ => false,
        }
    }

    fn deregister(&self, inner: &mut EventRouterInner<'cx, UiState>) -> usize {
        let listeners = &mut inner.listeners;
        let mut deregisters = self.dispatch.deregisters.lock().unwrap();
        let count = deregisters.len();
        for &index in deregisters.iter() {
            if let Some(listener) = listeners.get_mut(index) {
                *listener = None;
            }
        }
        deregisters.clear();
        count
    }

    fn update_bounds(&self, inner: &mut EventRouterInner<'cx, UiState>) -> usize {
        let listeners = &mut inner.listeners;
        let mut bounds_updates = self.dispatch.bounds_updates.lock().unwrap();
        let count = bounds_updates.len();
        for (&index, &bounds) in bounds_updates.iter() {
            if let Some(Some(listener)) = listeners.get_mut(index) {
                listener.bounds = bounds;
            }
        }
        bounds_updates.clear();
        count
    }

    /// Returns if should redraw.
    fn scan_events(
        &self,
        ui_state: &mut UiState,
        inner: &mut EventRouterInner<'cx, UiState>,
    ) -> bool {
        self.deregister(inner);
        self.update_bounds(inner);
        let Some(cursor_position) = inner.cursor_position else {
            return false;
        };
        let listeners_locked = &mut inner.listeners;
        let mut should_redraw = false;
        let button_states = &mut inner.button_states;
        // Scan for button hovering events.
        for listener in Self::listeners_iter_mut(listeners_locked) {
            let inside = listener.bounds.contains(cursor_position);
            let is_hovered_before = listener.is_hovered;
            // Scan for hovering changes.
            if inside && !listener.is_hovered {
                // Hovering start.
                listener.is_hovered = true;
                listener.object.mouse_event(
                    MouseEvent::new(MouseEventKind::HoveringStart, cursor_position),
                    ui_state,
                );
                should_redraw = true;
            } else if !inside && listener.is_hovered {
                // Hovering finish.
                listener.is_hovered = false;
                listener.object.mouse_event(
                    MouseEvent::new(MouseEventKind::HoveringFinish, cursor_position),
                    ui_state,
                );
                should_redraw = true;
            }
            // Scan for button up/down events.
            // Sanity check in case of future refractors.
            debug_assert!(listener.button_states.len() == button_states.len());
            for (i, (state, listener_state)) in
                iter::zip(button_states.iter(), &mut listener.button_states).enumerate()
            {
                let button = match i {
                    0 => MouseButton::Left,
                    1 => MouseButton::Right,
                    2 => MouseButton::Middle,
                    3 => MouseButton::Forward,
                    4 => MouseButton::Back,
                    _ => unreachable!(),
                };
                if !*state && *listener_state {
                    // Button up event.
                    *listener_state = *state;
                    let event = MouseEvent::new(
                        MouseEventKind::ButtonUp { button, inside },
                        cursor_position,
                    );
                    listener.object.mouse_event(event, ui_state);
                    should_redraw = true;
                } else if *state && !*listener_state && inside {
                    // Button down event.
                    *listener_state = *state;
                    let started_inside = is_hovered_before;
                    let event = MouseEvent::new(
                        MouseEventKind::ButtonDown {
                            button,
                            started_inside,
                        },
                        cursor_position,
                    );
                    listener.object.mouse_event(event, ui_state);
                    should_redraw = true;
                }
            }
        }
        should_redraw
    }
}

impl<'cx, UiState> Default for EventRouter<'cx, UiState> {
    fn default() -> Self {
        Self::new()
    }
}

struct Listener<'cx, UiState> {
    /// The bounds of this listener.
    bounds: Bounds<f32>,
    /// Is the cursor currently hovering over this listener?
    is_hovered: bool,
    /// Records the buttons that the listener is currently being pressed by.
    button_states: [bool; 5],
    /// The listener object type erased and boxed.
    object: Box<dyn MouseEventListener<UiState> + 'cx>,
}

/// Unregisters the listener when dropped.
#[derive(Clone)]
pub struct ListenerHandle {
    router_dispatch: Weak<EventRouterDispatch>,
    index: usize,
}

impl Debug for ListenerHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ListenerHandle")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        if let Some(router) = self.router_dispatch.upgrade() {
            router.deregisters.lock().unwrap().insert(self.index);
        };
    }
}

impl ListenerHandle {
    pub fn update_bounds(&self, bounds: Bounds<f32>) {
        if let Some(router_dispatch) = self.router_dispatch.upgrade() {
            router_dispatch
                .bounds_updates
                .lock()
                .unwrap()
                .insert(self.index, bounds);
        }
    }
}
