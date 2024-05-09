use crate::{FromResourceKind, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelSource {
    vertex_data: Vec<u8>,
    vertex_layout: Vec<PmxModelVertexLayoutElement>,
    index_data: Vec<u8>,
    index_kind: PmxModelIndexKind,
    elements: Vec<PmxModelElement>,
}

impl PmxModelSource {
    pub fn new(
        vertex_data: Vec<u8>,
        vertex_layout: Vec<PmxModelVertexLayoutElement>,
        index_data: Vec<u8>,
        index_kind: PmxModelIndexKind,
        elements: Vec<PmxModelElement>,
    ) -> Self {
        Self {
            vertex_data,
            vertex_layout,
            index_data,
            index_kind,
            elements,
        }
    }

    pub fn vertex_data(&self) -> &[u8] {
        &self.vertex_data
    }

    pub fn vertex_layout(&self) -> &[PmxModelVertexLayoutElement] {
        &self.vertex_layout
    }

    pub fn index_data(&self) -> &[u8] {
        &self.index_data
    }

    pub fn index_kind(&self) -> PmxModelIndexKind {
        self.index_kind
    }

    pub fn elements(&self) -> &[PmxModelElement] {
        &self.elements
    }
}

impl FromResourceKind for PmxModelSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::PmxModel(pmx_model) => Some(pmx_model),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PmxModelVertexLayoutElement {
    pub kind: PmxModelVertexLayoutElementKind,
    pub offset: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxModelVertexLayoutElementKind {
    /// `vec3f`
    Position,
    /// `vec3f`
    Normal,
    /// `vec2f`
    TexCoord,
    /// `vec3f`
    Tangent,
    /// `vec4f`
    AdditionalVec4(u8),
    /// `u32`, Bdef1 = 0, Bdef2 = 1, Bdef4 = 2, Sdef = 3, Qdef = 4
    DeformKind,
    /// `vec4i`, -1 for none
    BoneIndex,
    /// `vec4f`
    BoneWeight,
    /// `vec3f`
    SdefC,
    /// `vec3f`
    SdefR0,
    /// `vec3f`
    SdefR1,
    /// `float`
    EdgeSize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxModelIndexKind {
    U16,
    U32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelElement {
    pub material_name: String,
    pub index_range: (u32, u32),
}
