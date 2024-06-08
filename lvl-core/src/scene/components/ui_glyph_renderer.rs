use crate::{
    gfx::{elements::Font, glyph::GlyphLayoutConfig},
    scene::Component,
};
use std::{any::Any, sync::Arc};

pub struct UIGlyphRenderer {
    pub font: Arc<Font>,
    pub layout_config: GlyphLayoutConfig,
    pub text: String,
}

impl UIGlyphRenderer {
    pub fn new(font: Arc<Font>, text: impl Into<String>) -> Self {
        Self {
            font,
            layout_config: Default::default(),
            text: text.into(),
        }
    }
}

impl Component for UIGlyphRenderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
