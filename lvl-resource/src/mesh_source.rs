use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshSource {
    vertex_data: Vec<f32>,
    index_data: Vec<u8>,
    elements: Vec<MeshSourceElement>,
}

impl MeshSource {
    pub fn new(
        vertex_data: Vec<f32>,
        index_data: Vec<u8>,
        elements: Vec<MeshSourceElement>,
    ) -> Self {
        Self {
            vertex_data,
            index_data,
            elements,
        }
    }
}

impl FromResourceKind for MeshSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::Mesh(mesh) => Some(mesh),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshSourceElement {
    pub name: String,
    pub kind: MeshSourceElementKind,
    pub offset: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshSourceElementKind {
    /// Vec3
    Position,
    /// Vec3
    Normal,
    /// Vec2
    TexCoord(u8),
    /// Vec3
    Tangent,
}
