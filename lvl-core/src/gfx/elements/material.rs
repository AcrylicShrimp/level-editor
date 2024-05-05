use super::{Shader, Texture};
use crate::gfx::GfxContext;
use lvl_math::{Vec2, Vec3, Vec4};
use lvl_resource::{
    MaterialPropertyValueUniformKind, MaterialSource, ResourceFile, ShaderBindingElementKind,
    ShaderSource, TextureKind, TextureSource,
};
use std::{
    cell::{RefCell, RefMut},
    collections::BTreeMap,
    num::NonZeroU64,
    sync::Arc,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferBinding,
    BufferDescriptor, BufferUsages, Queue, Sampler, TextureView,
};
use zerocopy::AsBytes;

#[derive(Debug)]
pub struct Material {
    shader: Shader,
    uniform_buffer: Option<Buffer>,
    bind_groups: RefCell<Vec<Option<BindGroup>>>,
    properties: Vec<MaterialProperty>,
    property_name_index_map: BTreeMap<String, usize>,
}

impl Material {
    pub fn load_from_source(
        resource: &ResourceFile,
        source: &MaterialSource,
        gfx_ctx: &GfxContext,
    ) -> Self {
        let shader_source = resource.find::<ShaderSource>(source.shader_name()).unwrap();
        let shader = Shader::load_from_source(shader_source, gfx_ctx);

        let buffer_size = shader_source
            .binding_elements()
            .iter()
            .filter_map(|element| match element.kind {
                ShaderBindingElementKind::Buffer { size } => Some(size.get()),
                ShaderBindingElementKind::Texture { .. } => None,
                ShaderBindingElementKind::Sampler { .. } => None,
            })
            .sum::<u64>();
        let uniform_buffer = if buffer_size == 0 {
            None
        } else {
            Some(gfx_ctx.device.create_buffer(&BufferDescriptor {
                label: None,
                size: buffer_size,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }))
        };

        let mut bind_groups = Vec::with_capacity(shader.bind_group_layouts().len());

        for _ in 0..shader.bind_group_layouts().len() {
            bind_groups.push(None);
        }

        let mut previous_buffer_offset = 0;
        let mut properties = Vec::with_capacity(source.properties().len());
        let mut property_name_index_map = BTreeMap::new();

        for element in shader_source.binding_elements() {
            property_name_index_map.insert(element.name.clone(), properties.len());

            let kind = match element.kind {
                ShaderBindingElementKind::Buffer { size } => {
                    let offset = previous_buffer_offset;
                    previous_buffer_offset += size.get();
                    MaterialPropertyKind::Buffer {
                        offset,
                        size: size.get(),
                    }
                }
                ShaderBindingElementKind::Texture { .. } => MaterialPropertyKind::Texture,
                ShaderBindingElementKind::Sampler { .. } => MaterialPropertyKind::Sampler,
            };

            let mut value = match source.properties().get(&element.name) {
                Some(property) => match &property.value {
                    lvl_resource::MaterialPropertyValue::Texture { texture_name } => {
                        match resource.find::<TextureSource>(texture_name) {
                            Some(source) => match source.kind() {
                                TextureKind::Single(element) => {
                                    let texture = Texture::load_from_source(element, gfx_ctx);
                                    let texture_view =
                                        texture.handle().create_view(&Default::default());
                                    Some(MaterialPropertyValue::Texture(Arc::new(texture_view)))
                                }
                                TextureKind::Cubemap { .. } => todo!(),
                            },
                            _ => None,
                        }
                    }
                    lvl_resource::MaterialPropertyValue::Uniform(uniform_kind) => {
                        match uniform_kind {
                            MaterialPropertyValueUniformKind::Float(value) => {
                                Some(MaterialPropertyValue::Float(*value))
                            }
                            MaterialPropertyValueUniformKind::Vec2(value) => {
                                Some(MaterialPropertyValue::Vec2(*value))
                            }
                            MaterialPropertyValueUniformKind::Vec3(value) => {
                                Some(MaterialPropertyValue::Vec3(*value))
                            }
                            MaterialPropertyValueUniformKind::Vec4(value) => {
                                Some(MaterialPropertyValue::Vec4(*value))
                            }
                        }
                    }
                },
                None => None,
            };

            if let Some(preset_value) = value.as_ref() {
                if !kind.is_compatible(&preset_value) {
                    value = None;
                }
            }

            properties.push(MaterialProperty {
                group: element.group,
                binding: element.binding,
                kind,
                value,
            });
        }

        Self {
            shader,
            uniform_buffer,
            bind_groups: RefCell::new(bind_groups),
            properties,
            property_name_index_map,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn set_property(&mut self, name: &str, value: MaterialPropertyValue) -> bool {
        let index = if let Some(index) = self.property_name_index_map.get(name) {
            *index
        } else {
            return false;
        };

        if !self.properties[index].kind.is_compatible(&value) {
            return false;
        }

        self.properties[index].value = Some(value);
        self.bind_groups.borrow_mut()[index] = None;

        true
    }

    pub fn construct_bind_groups(
        &self,
        gfx_ctx: &GfxContext,
    ) -> Option<RefMut<Vec<Option<BindGroup>>>> {
        let mut bind_groups = self.bind_groups.borrow_mut();

        for group in 0..bind_groups.len() {
            if bind_groups[group].is_some() {
                continue;
            }

            let mut entries = Vec::new();

            loop {
                let property = self.properties.iter().find(|property| {
                    property.group as usize == group && property.binding as usize == entries.len()
                });
                let property = match property {
                    Some(property) => property,
                    None => {
                        break;
                    }
                };

                let value = match property.value.as_ref() {
                    Some(value) => value,
                    None => {
                        return None;
                    }
                };

                match value {
                    MaterialPropertyValue::Float(value) => {
                        entries.push(BindGroupEntry {
                            binding: property.binding as u32,
                            resource: BindingResource::Buffer(
                                self.prepare_uniform_value(
                                    value.as_bytes(),
                                    &property.kind,
                                    &gfx_ctx.queue,
                                )
                                .unwrap(),
                            ),
                        });
                    }
                    MaterialPropertyValue::Vec2(value) => {
                        entries.push(BindGroupEntry {
                            binding: property.binding as u32,
                            resource: BindingResource::Buffer(
                                self.prepare_uniform_value(
                                    value.as_bytes(),
                                    &property.kind,
                                    &gfx_ctx.queue,
                                )
                                .unwrap(),
                            ),
                        });
                    }
                    MaterialPropertyValue::Vec3(value) => {
                        entries.push(BindGroupEntry {
                            binding: property.binding as u32,
                            resource: BindingResource::Buffer(
                                self.prepare_uniform_value(
                                    value.as_bytes(),
                                    &property.kind,
                                    &gfx_ctx.queue,
                                )
                                .unwrap(),
                            ),
                        });
                    }
                    MaterialPropertyValue::Vec4(value) => {
                        entries.push(BindGroupEntry {
                            binding: property.binding as u32,
                            resource: BindingResource::Buffer(
                                self.prepare_uniform_value(
                                    value.as_bytes(),
                                    &property.kind,
                                    &gfx_ctx.queue,
                                )
                                .unwrap(),
                            ),
                        });
                    }
                    MaterialPropertyValue::Texture(value) => {
                        entries.push(BindGroupEntry {
                            binding: property.binding as u32,
                            resource: BindingResource::TextureView(value),
                        });
                    }
                    MaterialPropertyValue::Sampler(value) => {
                        entries.push(BindGroupEntry {
                            binding: property.binding as u32,
                            resource: BindingResource::Sampler(value),
                        });
                    }
                }
            }

            bind_groups[group] = Some(gfx_ctx.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.shader.bind_group_layouts()[group],
                entries: &entries,
            }));
        }

        if bind_groups.iter().any(|group| group.is_none()) {
            return None;
        }

        Some(bind_groups)
    }

    fn prepare_uniform_value(
        &self,
        data: &[u8],
        kind: &MaterialPropertyKind,
        queue: &Queue,
    ) -> Option<BufferBinding> {
        debug_assert!(self.uniform_buffer.is_some());

        let uniform_buffer = self.uniform_buffer.as_ref().unwrap();
        let (offset, size) = if let MaterialPropertyKind::Buffer { offset, size } = &kind {
            (
                *offset,
                match NonZeroU64::new(*size) {
                    Some(size) => size,
                    None => {
                        return None;
                    }
                },
            )
        } else {
            return None;
        };

        debug_assert!(data.len() as u64 == size.get());

        if let Some(mut view) = queue.write_buffer_with(uniform_buffer, offset, size) {
            view.copy_from_slice(data);
        }

        Some(BufferBinding {
            buffer: uniform_buffer,
            offset,
            size: Some(size),
        })
    }
}

#[derive(Debug)]
pub struct MaterialProperty {
    group: u32,
    binding: u32,
    kind: MaterialPropertyKind,
    value: Option<MaterialPropertyValue>,
}

#[derive(Debug)]
pub enum MaterialPropertyKind {
    Buffer { offset: u64, size: u64 },
    Texture,
    Sampler,
}

impl MaterialPropertyKind {
    pub fn is_compatible(&self, value: &MaterialPropertyValue) -> bool {
        match self {
            MaterialPropertyKind::Buffer { .. } => {
                matches!(
                    value,
                    MaterialPropertyValue::Float(_)
                        | MaterialPropertyValue::Vec2(_)
                        | MaterialPropertyValue::Vec3(_)
                        | MaterialPropertyValue::Vec4(_)
                )
            }
            MaterialPropertyKind::Texture => {
                matches!(value, MaterialPropertyValue::Texture(_))
            }
            MaterialPropertyKind::Sampler => {
                matches!(value, MaterialPropertyValue::Sampler(_))
            }
        }
    }
}

#[derive(Debug)]
pub enum MaterialPropertyValue {
    // buffer values
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),

    // texture values
    Texture(Arc<TextureView>),

    // sampler values
    Sampler(Arc<Sampler>),
}
