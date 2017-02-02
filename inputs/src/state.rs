use std::collections::HashSet;
use cgmath::Point2;
use glutin::{MouseButton, VirtualKeyCode, ElementState};

pub struct InputState {
    mouse: MouseState,
    keyboard: KeyboardState,
    window: WindowState,
}

impl InputState {
    pub(crate) fn new() -> Self {
        InputState {
            mouse: MouseState::new(),
            keyboard: KeyboardState::new(),
            window: WindowState::new(),
        }
    }

    pub(crate) fn update_mouse_position(&mut self, position: Point2<i32>) {
        self.mouse.position = position;
    }

    pub fn mouse_position(&self) -> Point2<i32> {
        self.mouse.position
    }

    pub(crate) fn update_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        use glutin::ElementState::*;

        match state {
            Pressed => self.mouse.buttons.insert(button),
            Released => self.mouse.buttons.remove(&button),
        };
    }

    pub fn is_mouse_button_held(&self, button: &MouseButton) -> bool {
        self.mouse.buttons.contains(button)
    }

    pub(crate) fn update_key(&mut self, key: VirtualKeyCode, state: ElementState) {
        use glutin::ElementState::*;

        match state {
            Pressed => self.keyboard.keys.insert(key),
            Released => self.keyboard.keys.remove(&key),
        };
    }

    pub fn is_key_held(&self, key: &VirtualKeyCode) -> bool {
        self.keyboard.keys.contains(key)
    }

    pub(crate) fn update_window_focus(&mut self, focused: bool) {
        self.window.focused = focused
    }

    pub fn is_window_focused(&self) -> bool {
        self.window.focused
    }
}

pub(crate) struct MouseState {
    position: Point2<i32>,
    buttons: HashSet<MouseButton>,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            position: Point2::new(0, 0),
            buttons: HashSet::new(),
        }
    }
}

pub(crate) struct KeyboardState {
    keys: HashSet<VirtualKeyCode>,
}

impl KeyboardState {
    pub fn new() -> Self {
        KeyboardState { keys: HashSet::new() }
    }
}

pub(crate) struct WindowState {
    focused: bool,
}

impl WindowState {
    pub fn new() -> Self {
        WindowState { focused: true }
    }
}