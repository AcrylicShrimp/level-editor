use serde::{Deserialize, Serialize};

use crate::{FromResourceKind, ResourceKind};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpriteSource {
    texture_name: String,
    mapping: SpriteMapping,
}

impl SpriteSource {
    pub fn new(texture_name: String, mapping: SpriteMapping) -> Self {
        Self {
            texture_name,
            mapping,
        }
    }

    pub fn texture_name(&self) -> &str {
        &self.texture_name
    }

    pub fn mapping(&self) -> &SpriteMapping {
        &self.mapping
    }
}

impl FromResourceKind for SpriteSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteMapping {
    pub min: (u16, u16),
    pub max: (u16, u16),
}
