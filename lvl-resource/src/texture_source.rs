use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureSource {
    kind: TextureKind,
}

impl TextureSource {
    pub fn new(kind: TextureKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> &TextureKind {
        &self.kind
    }
}

impl FromResourceKind for TextureSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::Texture(texture) => Some(texture),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TextureKind {
    Single(TextureElement),
    Cubemap {
        up: TextureElement,
        down: TextureElement,
        left: TextureElement,
        right: TextureElement,
        front: TextureElement,
        back: TextureElement,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureElement {
    pub data: Vec<u8>,
    pub size: TextureElementSize,
    pub texture_format: TextureElementTextureFormat,
    pub sampling_mode: TextureElementSamplingMode,
    pub wrapping_mode_u: TextureElementWrappingMode,
    pub wrapping_mode_v: TextureElementWrappingMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureElementSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureElementTextureFormat {
    RG32Uint,
    RGBA32Uint,
    RGBA32Float,
    RGBA8Unorm,
    RGBA8UnormSrgb,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureElementSamplingMode {
    Point,
    Bilinear,
    Trilinear,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureElementWrappingMode {
    Clamp,
    Repeat,
    Mirror,
}
