use super::{Material, MaterialPropertyValue, Shader, Texture};
use crate::gfx::GfxContext;
use lvl_resource::{
    PmxModelIndexKind, PmxModelMorphKind, PmxModelSource, PmxModelVertexLayoutElement,
    PmxModelVertexLayoutElementKind, ResourceFile, ShaderSource, TextureKind, TextureSource,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    mem::size_of,
    num::NonZeroU64,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Queue, TextureView,
};
use zerocopy::AsBytes;

#[derive(Debug)]
pub struct PmxModel {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    elements: Vec<PmxModelElement>,
    vertex_layout: PmxModelVertexLayout,
    index_kind: PmxModelIndexKind,
    morphs: HashMap<String, PmxModelMorph>,
    morph_coefficients: Vec<f32>,
    morph_coefficients_buffer: Arc<Buffer>,
    is_morph_dirty: AtomicBool,
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

        // TODO: check if the morph count is less than 128
        // TODO: make engine decide the maximum morph count, not hardcoded
        let morph_coefficients = vec![0f32; 128];
        let morph_coefficients_buffer = gfx_ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: morph_coefficients.as_bytes(),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });
        let morph_coefficients_buffer = Arc::new(morph_coefficients_buffer);

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
            let mut material = Material::load_from_source(
                &mut shader_loader,
                &mut texture_loader,
                material_source,
                gfx_ctx,
            );

            // TODO: can we make the property name configurable?
            material.set_property(
                "morph_coefficients",
                MaterialPropertyValue::StorageBuffer {
                    buffer: morph_coefficients_buffer.clone(),
                    offset: 0,
                    size: NonZeroU64::new((size_of::<[f32; 128]>()) as u64).unwrap(),
                },
            );

            elements.push(PmxModelElement {
                material,
                index_range: pmx_element.index_range,
            });
        }

        let mut morphs = HashMap::with_capacity(source.morphs().len());

        for (index, morph) in source.morphs().iter().enumerate() {
            morphs.insert(
                morph.name.clone(),
                PmxModelMorph {
                    morph_index: index as u32,
                    kind: morph.kind.clone(),
                },
            );
        }

        Self {
            vertex_buffer,
            index_buffer,
            elements,
            vertex_layout: PmxModelVertexLayout::new(Vec::from(source.vertex_layout())),
            index_kind: source.index_kind(),
            morphs,
            morph_coefficients,
            morph_coefficients_buffer,
            is_morph_dirty: AtomicBool::new(false),
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

    pub fn elements_mut(&mut self) -> &mut [PmxModelElement] {
        &mut self.elements
    }

    pub fn vertex_layout(&self) -> &PmxModelVertexLayout {
        &self.vertex_layout
    }

    pub fn index_kind(&self) -> PmxModelIndexKind {
        self.index_kind
    }

    pub fn morphs(&self) -> &HashMap<String, PmxModelMorph> {
        &self.morphs
    }

    pub fn morph_coefficients(&self) -> &[f32] {
        &self.morph_coefficients
    }

    pub fn set_morph(&mut self, name: &str, value: f32) {
        let morph_index = match self.morphs.get(name) {
            Some(morph) => morph.morph_index,
            None => {
                return;
            }
        };

        self.morph_coefficients[morph_index as usize] = value;
        self.is_morph_dirty.store(true, Ordering::SeqCst);
    }

    pub(crate) fn update_morph_coefficients(&self, queue: &Queue) {
        if self.is_morph_dirty.load(Ordering::SeqCst) {
            queue.write_buffer(
                &self.morph_coefficients_buffer,
                0,
                self.morph_coefficients.as_bytes(),
            );

            self.is_morph_dirty.store(false, Ordering::SeqCst);
        }
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

#[derive(Debug)]
pub struct PmxModelMorph {
    pub morph_index: u32,
    pub kind: PmxModelMorphKind,
}
