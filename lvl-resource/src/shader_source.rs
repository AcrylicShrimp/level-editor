use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};
use wgpu_types::{SamplerBindingType, TextureSampleType, TextureViewDimension};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderSource {
    render_type: ShaderRenderType,
    source: String,
    vs_main: String,
    fs_main: String,
    binding_elements: Vec<ShaderBindingElement>,
    locations: BTreeMap<String, u32>,
}

impl ShaderSource {
    pub fn new(
        render_type: ShaderRenderType,
        source: String,
        vs_main: String,
        fs_main: String,
        binding_elements: Vec<ShaderBindingElement>,
        locations: BTreeMap<String, u32>,
    ) -> Self {
        Self {
            render_type,
            source,
            vs_main,
            fs_main,
            binding_elements,
            locations,
        }
    }

    pub fn render_type(&self) -> &ShaderRenderType {
        &self.render_type
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn vs_main(&self) -> &str {
        &self.vs_main
    }

    pub fn fs_main(&self) -> &str {
        &self.fs_main
    }

    pub fn binding_elements(&self) -> &[ShaderBindingElement] {
        &self.binding_elements
    }

    pub fn locations(&self) -> &BTreeMap<String, u32> {
        &self.locations
    }
}

impl FromResourceKind for ShaderSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::Shader(shader) => Some(shader),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShaderRenderType {
    Opaque,
    Transparent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderBindingElement {
    pub name: String,
    pub group: u32,
    pub binding: u32,
    pub kind: ShaderBindingElementKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShaderBindingElementKind {
    Buffer {
        size: NonZeroU64,
    },
    Texture {
        sample_type: TextureSampleType,
        view_dimension: TextureViewDimension,
        multisampled: bool,
    },
    Sampler {
        binding_type: SamplerBindingType,
    },
}
