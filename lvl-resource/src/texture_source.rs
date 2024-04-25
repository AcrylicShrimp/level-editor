use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub source: TextureElementSource,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TextureElementSource {
    Image(TextureElementSourceImage),
    Raw(TextureElementSourceRaw),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureElementSourceImage {
    /// Relative to the resource file.
    pub path: PathBuf,
    pub image_format: TextureElementImageFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureElementSourceRaw {
    /// In little-endian. It always follows `texture_format`.
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureElementImageFormat {
    R8,
    R16,
    R32,
    RGB8,
    RGB16,
    RGB32,
    RGBA8,
    RGBA16,
    RGBA32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureElementTextureFormat {
    /// 8-bit unsigned normalized
    R8U,
    /// 8-bit signed normalized
    R8S,
    /// 16-bit unsigned integer
    R16U,
    /// 16-bit signed integer
    R16S,
    /// 16-bit floating-point
    R16F,
    /// 16-bit unsigned integer
    R32U,
    /// 32-bit signed integer
    R32S,
    /// 32-bit floating-point
    R32F,
    /// 8-bit unsigned normalized
    RG8U,
    /// 8-bit signed normalized
    RG8S,
    /// 16-bit unsigned integer
    RG16U,
    /// 16-bit signed integer
    RG16S,
    /// 16-bit floating-point
    RG16F,
    /// 16-bit unsigned integer
    RG32U,
    /// 32-bit signed integer
    RG32S,
    /// 32-bit floating-point
    RG32F,
    /// 8-bit unsigned normalized
    RGBA8U,
    /// 8-bit signed normalized
    RGBA8S,
    /// 16-bit unsigned integer
    RGBA16U,
    /// 16-bit signed integer
    RGBA16S,
    /// 16-bit floating-point
    RGBA16F,
    /// 16-bit unsigned integer
    RGBA32U,
    /// 32-bit signed integer
    RGBA32S,
    /// 32-bit floating-point
    RGBA32F,
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
