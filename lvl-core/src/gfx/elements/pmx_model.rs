mod morph;

use self::morph::Morph;
use super::{Material, Shader, Texture};
use crate::gfx::GfxContext;
use lvl_resource::{
    MaterialSource, PmxModelIndexKind, PmxModelSource, PmxModelVertexLayoutElement,
    PmxModelVertexLayoutElementKind, ResourceFile, ShaderSource, TextureKind, TextureSource,
};
use std::{
    cell::{Ref, RefCell},
    collections::{hash_map::Entry, HashMap},
    mem::size_of,
    ops::Range,
    sync::Arc,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, TextureView,
};

#[derive(Debug)]
pub struct PmxModel {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    elements: Vec<PmxModelElement>,
    vertex_layout: PmxModelVertexLayout,
    index_kind: PmxModelIndexKind,
    morph: RefCell<Morph>,
}

impl PmxModel {
    pub fn load_from_source<'a>(
        resource: &'a ResourceFile,
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

        let mut shader_cache = HashMap::<String, (Arc<Shader>, &'a ShaderSource)>::new();
        let mut shader_loader = |name: &str| -> Option<(Arc<Shader>, &'a ShaderSource)> {
            match shader_cache.entry(name.to_owned()) {
                Entry::Occupied(entry) => Some(entry.get().clone()),
                Entry::Vacant(entry) => {
                    let shader_source = match resource.find::<ShaderSource>(name) {
                        Some(source) => source,
                        None => {
                            return None;
                        }
                    };

                    let shader = Arc::new(Shader::load_from_source(shader_source, gfx_ctx));
                    entry.insert((shader.clone(), shader_source));
                    Some((shader, shader_source))
                }
            }
        };

        let mut texture_cache = HashMap::<String, Arc<TextureView>>::new();
        let mut texture_loader = |name: &str| -> Option<Arc<TextureView>> {
            match texture_cache.entry(name.to_owned()) {
                Entry::Occupied(entry) => Some(entry.get().clone()),
                Entry::Vacant(entry) => {
                    let texture_source = match resource.find::<TextureSource>(name) {
                        Some(source) => source,
                        None => {
                            return None;
                        }
                    };

                    match texture_source.kind() {
                        TextureKind::Single(element) => {
                            let texture = Texture::load_from_source(element, gfx_ctx);
                            let texture_view =
                                Arc::new(texture.handle().create_view(&Default::default()));
                            entry.insert(texture_view.clone());
                            Some(texture_view)
                        }
                        TextureKind::Cubemap { .. } => None,
                    }
                }
            }
        };

        let mut elements = Vec::with_capacity(source.elements().len());

        for pmx_element in source.elements() {
            let material_source = resource
                .find::<MaterialSource>(&pmx_element.material_name)
                .unwrap();

            let material = Material::load_from_source(
                &mut shader_loader,
                &mut texture_loader,
                material_source,
                gfx_ctx,
            );

            elements.push(PmxModelElement {
                material,
                index_range: pmx_element.index_range.0..pmx_element.index_range.1,
            });
        }

        let morph: Morph = Morph::new(source.morphs(), &mut elements, &gfx_ctx.device);

        Self {
            vertex_buffer,
            index_buffer,
            elements,
            vertex_layout: PmxModelVertexLayout::new(Vec::from(source.vertex_layout())),
            index_kind: source.index_kind(),
            morph: RefCell::new(morph),
        }
    }

    pub fn morph(&self) -> Ref<Morph> {
        self.morph.borrow()
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

    pub fn elements_mut(&mut self) -> &mut [PmxModelElement] {
        &mut self.elements
    }

    pub fn vertex_layout(&self) -> &PmxModelVertexLayout {
        &self.vertex_layout
    }

    pub fn index_kind(&self) -> PmxModelIndexKind {
        self.index_kind
    }

    pub fn set_morph(&mut self, name: &str, coefficient: f32) {
        let mut morph = self.morph.borrow_mut();
        morph.set_morph(name, coefficient);
        morph.update_material_values(&mut self.elements);
    }
}

#[derive(Debug)]
pub struct PmxModelElement {
    pub material: Material,
    pub index_range: Range<u32>,
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
                    PmxModelVertexLayoutElementKind::VertexMorphIndexStart => size_of::<u32>(),
                    PmxModelVertexLayoutElementKind::UvMorphIndexStart => size_of::<u32>(),
                    PmxModelVertexLayoutElementKind::VertexMorphCount => size_of::<u32>(),
                    PmxModelVertexLayoutElementKind::UvMorphCount => size_of::<u32>(),
                };
                element.offset + size as u64
            })
            .max()
            .unwrap_or_default();

        Self { elements, stride }
    }
}
