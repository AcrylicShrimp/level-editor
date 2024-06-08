use crate::{gfx::elements::Sprite, scene::Component};
use std::any::Any;

#[derive(Debug)]
pub struct UISpriteRenderer {
    pub sprite: Sprite,
}

impl UISpriteRenderer {
    pub fn new(sprite: Sprite) -> Self {
        Self { sprite }
    }
}

impl Component for UISpriteRenderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
