use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialProperty {
    pub name: String,
    pub value: MaterialPropertyValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialPropertyValue {
    Texture { texture_name: String },
    Dynamic(MaterialPropertyValueDynamic),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialPropertyValueDynamic {
    pub key: String,
}
