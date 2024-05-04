use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshSource {
    vertex_data: Vec<u8>,
    index_data: Vec<u8>,
    index_kind: MeshIndexKind,
    elements: Vec<MeshElement>,
}

impl MeshSource {
    pub fn new(
        vertex_data: Vec<u8>,
        index_data: Vec<u8>,
        index_kind: MeshIndexKind,
        elements: Vec<MeshElement>,
    ) -> Self {
        Self {
            vertex_data,
            index_data,
            index_kind,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshIndexKind {
    /// u8
    U8,
    /// u16
    U16,
    /// u32
    U32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshElement {
    pub name: String,
    pub kind: MeshElementKind,
    pub offset: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshElementKind {
    /// Vec3
    Position,
    /// Vec3
    Normal,
    /// Vec2
    TexCoord(u8),
    /// Vec3
    Tangent,
    /// vec4
    Additional(u8),
}
