use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::{FromResourceKind, ResourceKind};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialSource {
    shader_name: String,
    properties: BTreeMap<String, MaterialProperty>,
}

impl MaterialSource {
    pub fn new(shader_name: String, properties: BTreeMap<String, MaterialProperty>) -> Self {
        Self {
            shader_name,
            properties,
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
}
