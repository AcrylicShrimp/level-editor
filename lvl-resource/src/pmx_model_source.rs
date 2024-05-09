use crate::{FromResourceKind, ResourceKind};
use lvl_math::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelSource {
    vertex_data: Vec<u8>,
    vertex_layout: Vec<PmxModelVertexLayoutElement>,
    index_data: Vec<u8>,
    index_kind: PmxModelIndexKind,
    elements: Vec<PmxModelElement>,
    morphs: Vec<PmxModelMorph>,
    vertex_morph_index_texture_name: String,
    uv_morph_index_texture_name: String,
    vertex_displacement_texture_name: String,
    uv_displacement_texture_name: String,
}

impl PmxModelSource {
    pub fn new(
        vertex_data: Vec<u8>,
        vertex_layout: Vec<PmxModelVertexLayoutElement>,
        index_data: Vec<u8>,
        index_kind: PmxModelIndexKind,
        elements: Vec<PmxModelElement>,
        morphs: Vec<PmxModelMorph>,
        vertex_morph_index_texture_name: String,
        uv_morph_index_texture_name: String,
        vertex_displacement_texture_name: String,
        uv_displacement_texture_name: String,
    ) -> Self {
        Self {
            vertex_data,
            vertex_layout,
            index_data,
            index_kind,
            elements,
            morphs,
            vertex_morph_index_texture_name,
            uv_morph_index_texture_name,
            vertex_displacement_texture_name,
            uv_displacement_texture_name,
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

    pub fn morphs(&self) -> &[PmxModelMorph] {
        &self.morphs
    }

    pub fn vertex_morph_index_texture_name(&self) -> &str {
        &self.vertex_morph_index_texture_name
    }

    pub fn uv_morph_index_texture_name(&self) -> &str {
        &self.uv_morph_index_texture_name
    }

    pub fn vertex_displacement_texture_name(&self) -> &str {
        &self.vertex_displacement_texture_name
    }

    pub fn uv_displacement_texture_name(&self) -> &str {
        &self.uv_displacement_texture_name
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
    /// `u32`
    VertexMorphIndexStart,
    /// `u32`
    VertexMorphCount,
    /// `u32`
    UvMorphIndexStart,
    /// `u32`
    UvMorphCount,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelMorph {
    pub name: String,
    pub kind: PmxModelMorphKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PmxModelMorphKind {
    Group(Vec<PmxModelMorphGroupElement>),
    Vertex,
    Uv,
    Material(Vec<PmxModelMorphMaterialElement>),
    // TODO: implement this
    Bone,
    // TODO: implement this
    Impulse,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct PmxModelMorphGroupElement {
    pub morph_index: u32,
    pub coefficient: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelMorphMaterialElement {
    /// `None` for all materials
    pub material_index: Option<u32>,
    pub offset_mode: PmxModelMorphMaterialOffsetMode,
    pub diffuse_color: Vec4,
    pub specular_color: Vec3,
    pub specular_strength: f32,
    pub ambient_color: Vec3,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub texture_tint_color: Vec4,
    pub environment_tint_color: Vec4,
    pub toon_tint_color: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxModelMorphMaterialOffsetMode {
    Multiply,
    Additive,
}
