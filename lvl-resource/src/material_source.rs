use crate::{FromResourceKind, ResourceKind};
use lvl_math::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wgpu_types::{AddressMode, CompareFunction, FilterMode, SamplerBorderColor};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialSource {
    shader_name: String,
    properties: BTreeMap<String, MaterialProperty>,
}

impl MaterialSource {
    pub fn new(shader_name: String, properties: Vec<MaterialProperty>) -> Self {
        Self {
            shader_name,
            properties: BTreeMap::from_iter(properties.into_iter().map(|p| (p.name.clone(), p))),
        }
    }

    pub fn shader_name(&self) -> &str {
        &self.shader_name
    }

    pub fn properties(&self) -> &BTreeMap<String, MaterialProperty> {
        &self.properties
    }
}

impl FromResourceKind for MaterialSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::Material(material) => Some(material),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialProperty {
    pub name: String,
    pub value: MaterialPropertyValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialPropertyValue {
    Texture {
        texture_name: String,
    },
    Sampler {
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        address_mode_w: AddressMode,
        mag_filter: FilterMode,
        min_filter: FilterMode,
        mipmap_filter: FilterMode,
        lod_min_clamp: f32,
        lod_max_clamp: f32,
        compare: Option<CompareFunction>,
        anisotropy_clamp: u16,
        border_color: Option<SamplerBorderColor>,
    },
    Uniform(MaterialPropertyValueUniformKind),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialPropertyValueUniformKind {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}
