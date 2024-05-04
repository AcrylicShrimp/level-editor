use crate::gfx::GfxContext;
use lvl_resource::{MeshElementKind, MeshIndexKind, MeshSource};
use std::mem::size_of;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages,
};

#[derive(Debug)]
pub struct StaticMesh {
    vertex_count: u32,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_kind: MeshIndexKind,
    layout: MeshLayout,
}

impl StaticMesh {
    pub fn load_from_source(source: &MeshSource, gfx_ctx: &GfxContext) -> Self {
        let vertex_buffer = gfx_ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: source.vertex_data(),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = gfx_ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: source.index_data(),
            usage: BufferUsages::INDEX,
        });
        let layout = MeshLayout::new(
            source
                .elements()
                .iter()
                .map(|element| MeshLayoutElement {
                    name: element.name.clone(),
                    kind: match element.kind {
                        MeshElementKind::Position => MeshLayoutElementKind::Position,
                        MeshElementKind::Normal => MeshLayoutElementKind::Normal,
                        MeshElementKind::TexCoord(index) => MeshLayoutElementKind::TexCoord(index),
                        MeshElementKind::Tangent => MeshLayoutElementKind::Tangent,
                        MeshElementKind::Additional(index) => {
                            MeshLayoutElementKind::Additional(index)
                        }
                    },
                    offset: element.offset,
                })
                .collect(),
        );

        Self {
            vertex_count: source.vertex_count(),
            vertex_buffer,
            index_buffer,
            index_kind: source.index_kind(),
            layout,
        }
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn index_kind(&self) -> MeshIndexKind {
        self.index_kind
    }

    pub fn layout(&self) -> &MeshLayout {
        &self.layout
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshLayout {
    elements: Vec<MeshLayoutElement>,
    stride: u64,
}

impl MeshLayout {
    pub fn new(elements: Vec<MeshLayoutElement>) -> Self {
        let stride = compute_stride_from_elements(&elements);
        Self { elements, stride }
    }

    pub fn with_stride(elements: Vec<MeshLayoutElement>, stride: u64) -> Self {
        Self { elements, stride }
    }

    pub fn elements(&self) -> &[MeshLayoutElement] {
        &self.elements
    }

    pub fn stride(&self) -> u64 {
        self.stride
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshLayoutElement {
    pub name: String,
    pub kind: MeshLayoutElementKind,
    pub offset: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshLayoutElementKind {
    /// Vec3
    Position,
    /// Vec3
    Normal,
    /// Vec2
    TexCoord(u8),
    /// Vec3
    Tangent,
    /// Additional, vec4
    Additional(u8),
}

impl MeshLayoutElementKind {
    pub fn size(self) -> usize {
        match self {
            Self::Position => size_of::<[f32; 3]>(),
            Self::Normal => size_of::<[f32; 3]>(),
            Self::TexCoord(_) => size_of::<[f32; 2]>(),
            Self::Tangent => size_of::<[f32; 3]>(),
            Self::Additional(_) => size_of::<[f32; 4]>(),
        }
    }
}

fn compute_stride_from_elements(elements: &[MeshLayoutElement]) -> u64 {
    elements
        .iter()
        .map(|element| element.kind.size() as u64 + element.offset)
        .max()
        .unwrap_or_default()
}
