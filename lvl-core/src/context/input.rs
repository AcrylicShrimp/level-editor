use std::collections::HashMap;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::PhysicalKey,
};

#[derive(Debug)]
pub struct Input {
    keys: HashMap<String, InputKey>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn key(&self, name: &str) -> Option<&InputKey> {
        self.keys.get(name)
    }

    pub fn register_key(&mut self, name: impl Into<String>, key: PhysicalKey) {
        self.keys.insert(name.into(), InputKey::new(key));
    }

    /// It must be called at the end of each frame.
    pub(crate) fn reset_current_frame_state(&mut self) {
        for input_key in self.keys.values_mut() {
            input_key.is_pressed_frame = false;
        }
    }

    pub(crate) fn handle_key_event(&mut self, event: &KeyEvent) {
        for input_key in self.keys.values_mut() {
            if input_key.key != event.physical_key {
                continue;
            }

            input_key.is_pressed = event.state == ElementState::Pressed;
            input_key.is_pressed_frame = event.state == ElementState::Pressed;
        }
    }
}

#[derive(Debug)]
pub struct InputKey {
    pub key: PhysicalKey,
    pub is_pressed: bool,
    pub is_pressed_frame: bool,
}

impl InputKey {
    pub fn new(key: PhysicalKey) -> Self {
        Self {
            key,
            is_pressed: false,
            is_pressed_frame: false,
        }
    }
}
