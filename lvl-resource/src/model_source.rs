use crate::{FromResourceKind, ResourceKind};
use lvl_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelSource {
    root_element_index: u32,
    elements: Vec<ModelElement>,
}

impl ModelSource {
    pub fn new(root_element_index: u32, elements: Vec<ModelElement>) -> Self {
        Self {
            root_element_index,
            elements,
        }
    }
}

impl FromResourceKind for ModelSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::Model(model) => Some(model),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelElement {
    pub index: u32,
    pub name: String,
    pub parent_index: Option<u32>,
    pub transform: ModelTransform,
    pub visible_part: Option<ModelVisiblePart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelVisiblePart {
    pub mesh_name: String,
    pub material_name: String,
}
