use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshSource {
    vertex_count: u32,
    vertex_data: Vec<u8>,
    index_data: Vec<u8>,
    index_kind: MeshIndexKind,
    elements: Vec<MeshElement>,
}

impl MeshSource {
    pub fn new(
        vertex_count: u32,
        vertex_data: Vec<u8>,
        index_data: Vec<u8>,
        index_kind: MeshIndexKind,
        elements: Vec<MeshElement>,
    ) -> Self {
        Self {
            vertex_count,
            vertex_data,
            index_data,
            index_kind,
            elements,
        }
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn vertex_data(&self) -> &[u8] {
        &self.vertex_data
    }

    pub fn index_data(&self) -> &[u8] {
        &self.index_data
    }

    pub fn index_kind(&self) -> MeshIndexKind {
        self.index_kind
    }

    pub fn elements(&self) -> &[MeshElement] {
        &self.elements
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
