use super::{Material, MaterialPropertyValue, Shader, Texture};
use crate::gfx::GfxContext;
use lvl_math::{Vec3, Vec4};
use lvl_resource::{
    MaterialPropertyValueUniformKind, MaterialSource, PmxModelIndexKind, PmxModelMorphKind,
    PmxModelMorphMaterialElement, PmxModelMorphMaterialOffsetMode, PmxModelSource,
    PmxModelVertexLayoutElement, PmxModelVertexLayoutElementKind, ResourceFile, ShaderSource,
    TextureKind, TextureSource,
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
    material_morph_targets: Vec<Option<PmxMaterialMorphTarget>>,
    morph_name_map: HashMap<String, u32>,
    morphs: Vec<PmxModelMorph>,
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

        let mut material_morph_targets = Vec::with_capacity(source.elements().len());
        let mut elements = Vec::with_capacity(source.elements().len());

        for pmx_element in source.elements() {
            if pmx_element.index_range.1 <= pmx_element.index_range.0 {
                material_morph_targets.push(None);
                continue;
            }

            let material_source = match resource.find::<MaterialSource>(&pmx_element.material_name)
            {
                Some(source) => source,
                None => {
                    material_morph_targets.push(None);
                    continue;
                }
            };

            material_morph_targets.push(Some(PmxMaterialMorphTarget {
                element_index: elements.len() as u32,
                diffuse_color: material_source
                    .properties()
                    .get("diffuse_color")
                    .and_then(|property| match &property.value {
                        lvl_resource::MaterialPropertyValue::Uniform(value) => match value {
                            MaterialPropertyValueUniformKind::Vec4(value) => Some(*value),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_default(),
                specular_color: material_source
                    .properties()
                    .get("specular_color")
                    .and_then(|property| match &property.value {
                        lvl_resource::MaterialPropertyValue::Uniform(value) => match value {
                            MaterialPropertyValueUniformKind::Vec3(value) => Some(*value),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_default(),
                specular_strength: material_source
                    .properties()
                    .get("specular_strength")
                    .and_then(|property| match &property.value {
                        lvl_resource::MaterialPropertyValue::Uniform(value) => match value {
                            MaterialPropertyValueUniformKind::Float(value) => Some(*value),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_default(),
                ambient_color: material_source
                    .properties()
                    .get("ambient_color")
                    .and_then(|property| match &property.value {
                        lvl_resource::MaterialPropertyValue::Uniform(value) => match value {
                            MaterialPropertyValueUniformKind::Vec3(value) => Some(*value),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_default(),
                edge_color: material_source
                    .properties()
                    .get("edge_color")
                    .and_then(|property| match &property.value {
                        lvl_resource::MaterialPropertyValue::Uniform(value) => match value {
                            MaterialPropertyValueUniformKind::Vec4(value) => Some(*value),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_default(),
                edge_size: material_source
                    .properties()
                    .get("edge_size")
                    .and_then(|property| match &property.value {
                        lvl_resource::MaterialPropertyValue::Uniform(value) => match value {
                            MaterialPropertyValueUniformKind::Float(value) => Some(*value),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_default(),
            }));

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
                material: material,
                index_range: pmx_element.index_range,
            });
        }

        let mut morph_name_map = HashMap::with_capacity(source.morphs().len());
        let mut morphs = Vec::with_capacity(source.morphs().len());

        for (index, morph) in source.morphs().iter().enumerate() {
            morph_name_map.insert(morph.name.clone(), morphs.len() as u32);
            morphs.push(PmxModelMorph {
                morph_index: index as u32,
                kind: morph.kind.clone(),
            });
        }

        Self {
            vertex_buffer,
            index_buffer,
            elements,
            vertex_layout: PmxModelVertexLayout::new(Vec::from(source.vertex_layout())),
            index_kind: source.index_kind(),
            material_morph_targets,
            morph_name_map,
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

    pub fn morph_name_map(&self) -> &HashMap<String, u32> {
        &self.morph_name_map
    }

    pub fn morphs(&self) -> &[PmxModelMorph] {
        &self.morphs
    }

    pub fn morph_coefficients(&self) -> &[f32] {
        &self.morph_coefficients
    }

    pub fn set_morph(&mut self, name: &str, value: f32) {
        let morph_index = match self.morph_name_map.get(name) {
            Some(index) => *index as usize,
            None => {
                return;
            }
        };

        self.morph_coefficients[morph_index] = value;
        self.is_morph_dirty.store(true, Ordering::SeqCst);

        match &self.morphs[morph_index].kind {
            PmxModelMorphKind::Group(elements) => {
                for element in elements {
                    self.morph_coefficients[element.morph_index as usize] =
                        value * element.coefficient;

                    match &self.morphs[element.morph_index as usize].kind {
                        PmxModelMorphKind::Material(elements) => {
                            for element in elements {
                                match element.material_index {
                                    Some(index) => {
                                        let target =
                                            match &self.material_morph_targets[index as usize] {
                                                Some(target) => target,
                                                None => {
                                                    continue;
                                                }
                                            };
                                        let material = &mut self.elements
                                            [target.element_index as usize]
                                            .material;
                                        apply_material_morph(value, material, target, &element);
                                    }
                                    None => {
                                        for target in &self.material_morph_targets {
                                            let target = match target {
                                                Some(target) => target,
                                                None => {
                                                    continue;
                                                }
                                            };
                                            let material = &mut self.elements
                                                [target.element_index as usize]
                                                .material;
                                            apply_material_morph(value, material, target, &element);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            PmxModelMorphKind::Material(elements) => {
                for element in elements {
                    match element.material_index {
                        Some(index) => {
                            let target = match &self.material_morph_targets[index as usize] {
                                Some(target) => target,
                                None => {
                                    continue;
                                }
                            };
                            let material =
                                &mut self.elements[target.element_index as usize].material;
                            apply_material_morph(value, material, target, &element);
                        }
                        None => {
                            for target in &self.material_morph_targets {
                                let target = match target {
                                    Some(target) => target,
                                    None => {
                                        continue;
                                    }
                                };
                                let material =
                                    &mut self.elements[target.element_index as usize].material;
                                apply_material_morph(value, material, target, &element);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
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

#[derive(Debug)]
pub struct PmxMaterialMorphTarget {
    pub element_index: u32,
    pub diffuse_color: Vec4,
    pub specular_color: Vec3,
    pub specular_strength: f32,
    pub ambient_color: Vec3,
    pub edge_color: Vec4,
    pub edge_size: f32,
}

fn apply_material_morph(
    coefficient: f32,
    material: &mut Material,
    target: &PmxMaterialMorphTarget,
    element: &PmxModelMorphMaterialElement,
) {
    material.set_property(
        "diffuse_color",
        MaterialPropertyValue::Vec4(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                target.diffuse_color * coefficient * element.diffuse_color
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                target.diffuse_color + coefficient * element.diffuse_color
            }
        }),
    );
    material.set_property(
        "specular_color",
        MaterialPropertyValue::Vec3(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                target.specular_color * coefficient * element.specular_color
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                target.specular_color + coefficient * element.specular_color
            }
        }),
    );
    material.set_property(
        "specular_color",
        MaterialPropertyValue::Vec3(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                target.specular_color * coefficient * element.specular_color
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                target.specular_color + coefficient * element.specular_color
            }
        }),
    );
    material.set_property(
        "specular_strength",
        MaterialPropertyValue::Float(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                target.specular_strength * coefficient * element.specular_strength
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                target.specular_strength + coefficient * element.specular_strength
            }
        }),
    );
    material.set_property(
        "ambient_color",
        MaterialPropertyValue::Vec3(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                target.ambient_color * coefficient * element.ambient_color
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                target.ambient_color + coefficient * element.ambient_color
            }
        }),
    );
    material.set_property(
        "edge_color",
        MaterialPropertyValue::Vec4(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                target.edge_color * coefficient * element.edge_color
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                target.edge_color + coefficient * element.edge_color
            }
        }),
    );
    material.set_property(
        "edge_size",
        MaterialPropertyValue::Float(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                target.edge_size * coefficient * element.edge_size
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                target.edge_size + coefficient * element.edge_size
            }
        }),
    );
    material.set_property(
        "texture_tint_color",
        MaterialPropertyValue::Vec4(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => coefficient * element.texture_tint_color,
            PmxModelMorphMaterialOffsetMode::Additive => coefficient * element.texture_tint_color,
        }),
    );
    material.set_property(
        "environment_tint_color",
        MaterialPropertyValue::Vec4(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => {
                coefficient * element.environment_tint_color
            }
            PmxModelMorphMaterialOffsetMode::Additive => {
                coefficient * element.environment_tint_color
            }
        }),
    );
    material.set_property(
        "toon_tint_color",
        MaterialPropertyValue::Vec4(match element.offset_mode {
            PmxModelMorphMaterialOffsetMode::Multiply => coefficient * element.toon_tint_color,
            PmxModelMorphMaterialOffsetMode::Additive => coefficient * element.toon_tint_color,
        }),
    );
}
