use crate::{FromResourceKind, ResourceKind};
use lvl_math::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    Texture { texture_name: String },
    Uniform(MaterialPropertyValueUniformKind),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialPropertyValueUniformKind {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}
