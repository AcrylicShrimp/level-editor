use std::mem::size_of;

use super::Material;
use crate::gfx::GfxContext;
use lvl_resource::{
    PmxModelIndexKind, PmxModelSource, PmxModelVertexLayoutElement,
    PmxModelVertexLayoutElementKind, ResourceFile,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages,
};

#[derive(Debug)]
pub struct PmxModel {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    elements: Vec<PmxModelElement>,
    vertex_layout: PmxModelVertexLayout,
    index_kind: PmxModelIndexKind,
}

impl PmxModel {
    pub fn load_from_source(
        resource: &ResourceFile,
        source: &PmxModelSource,
        gfx_ctx: &GfxContext,
    ) -> Self {
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

        let mut elements = Vec::with_capacity(source.elements().len());

        for pmx_element in source.elements() {
            if pmx_element.index_range.1 <= pmx_element.index_range.0 {
                continue;
            }

            let material_source = match resource.find(&pmx_element.material_name) {
                Some(source) => source,
                None => {
                    continue;
                }
            };
            let material = Material::load_from_source(resource, material_source, gfx_ctx);

            elements.push(PmxModelElement {
                material,
                index_range: pmx_element.index_range,
            });
        }

        Self {
            vertex_buffer,
            index_buffer,
            elements,
            vertex_layout: PmxModelVertexLayout::new(Vec::from(source.vertex_layout())),
            index_kind: source.index_kind(),
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn elements(&self) -> &[PmxModelElement] {
        &self.elements
    }

    pub fn vertex_layout(&self) -> &PmxModelVertexLayout {
        &self.vertex_layout
    }

    pub fn index_kind(&self) -> PmxModelIndexKind {
        self.index_kind
    }
}

#[derive(Debug)]
pub struct PmxModelElement {
    pub material: Material,
    pub index_range: (u32, u32),
}

#[derive(Debug, Clone, Hash)]
pub struct PmxModelVertexLayout {
    pub elements: Vec<PmxModelVertexLayoutElement>,
    pub stride: u64,
}

impl PmxModelVertexLayout {
    pub fn new(elements: Vec<PmxModelVertexLayoutElement>) -> Self {
        let stride = elements
            .iter()
            .map(|element| {
                let size = match element.kind {
                    PmxModelVertexLayoutElementKind::Position => size_of::<[f32; 3]>(),
                    PmxModelVertexLayoutElementKind::Normal => size_of::<[f32; 3]>(),
                    PmxModelVertexLayoutElementKind::TexCoord => size_of::<[f32; 2]>(),
                    PmxModelVertexLayoutElementKind::Tangent => size_of::<[f32; 3]>(),
                    PmxModelVertexLayoutElementKind::AdditionalVec4(_) => size_of::<[f32; 4]>(),
                    PmxModelVertexLayoutElementKind::DeformKind => size_of::<u32>(),
                    PmxModelVertexLayoutElementKind::BoneIndex => size_of::<[i32; 4]>(),
                    PmxModelVertexLayoutElementKind::BoneWeight => size_of::<[f32; 4]>(),
                    PmxModelVertexLayoutElementKind::SdefC => size_of::<[f32; 3]>(),
                    PmxModelVertexLayoutElementKind::SdefR0 => size_of::<[f32; 3]>(),
                    PmxModelVertexLayoutElementKind::SdefR1 => size_of::<[f32; 3]>(),
                    PmxModelVertexLayoutElementKind::EdgeSize => size_of::<f32>(),
                };
                element.offset + size as u64
            })
            .max()
            .unwrap_or_default();

        Self { elements, stride }
    }
}
