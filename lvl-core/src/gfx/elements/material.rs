use super::Shader;
use crate::gfx::GfxContext;
use lvl_math::{Vec2, Vec3, Vec4};
use lvl_resource::{
    MaterialPropertyUniformValue, MaterialRenderState, MaterialSource, ShaderBindingKind,
    ShaderSource,
};
use std::{
    cell::{RefCell, RefMut},
    collections::BTreeMap,
    num::NonZeroU64,
    sync::Arc,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferBinding,
    BufferDescriptor, BufferSize, BufferUsages, Queue, Sampler, SamplerDescriptor, TextureView,
};
use zerocopy::AsBytes;

#[derive(Debug)]
pub struct Material {
    shader: Arc<Shader>,
    render_state: MaterialRenderState,
    uniform_buffers: Vec<Buffer>,
    uniform_structs: Vec<UniformStruct>,
    bind_groups: RefCell<Vec<Option<BindGroup>>>,
    properties: Vec<MaterialProperty>,
    property_name_index_map: BTreeMap<String, usize>,
}

impl Material {
    pub fn load_from_source<'a>(
        mut shader_loader: impl FnMut(&str) -> Option<(Arc<Shader>, &'a ShaderSource)>,
        mut texture_loader: impl FnMut(&str) -> Option<Arc<TextureView>>,
        source: &MaterialSource,
        gfx_ctx: &GfxContext,
    ) -> Self {
        let (shader, shader_source) = shader_loader(source.shader_name()).unwrap();

        let mut uniform_buffers = Vec::new();

        for binding in shader_source.bindings() {
            match &binding.kind {
                ShaderBindingKind::UniformBuffer { size, .. } => {
                    let buffer = gfx_ctx.device.create_buffer(&BufferDescriptor {
                        label: None,
                        size: size.get(),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    uniform_buffers.push(buffer);
                }
                _ => {
                    continue;
                }
            }
        }

        let mut bind_groups = Vec::with_capacity(shader.bind_group_layouts().len());

        for _ in 0..shader.bind_group_layouts().len() {
            bind_groups.push(None);
        }

        let mut uniform_structs = Vec::new();
        let mut properties = Vec::with_capacity(source.properties().len());
        let mut property_name_index_map = BTreeMap::new();

        for binding in shader_source.bindings() {
            let index = match binding.kind {
                ShaderBindingKind::UniformBuffer {
                    index, is_struct, ..
                } if is_struct => index,
                _ => {
                    continue;
                }
            };

            uniform_structs.push(UniformStruct {
                group: binding.group,
                binding: binding.binding,
                buffer_index: index,
            });
        }

        for binding in shader_source.bindings() {
            let kind = match binding.kind {
                ShaderBindingKind::UniformBuffer {
                    index, is_struct, ..
                } if !is_struct => MaterialPropertyKind::UniformBuffer {
                    group: binding.group,
                    binding: binding.binding,
                    buffer_index: index,
                },
                ShaderBindingKind::StorageBuffer { size, .. } => {
                    MaterialPropertyKind::StorageBuffer { size }
                }
                ShaderBindingKind::Texture { .. } => MaterialPropertyKind::Texture,
                ShaderBindingKind::Sampler { .. } => MaterialPropertyKind::Sampler,
                _ => {
                    continue;
                }
            };

            property_name_index_map.insert(binding.name.clone(), properties.len());

            let mut value = match source.properties().get(&binding.name) {
                Some(property) => match &property.value {
                    lvl_resource::MaterialPropertyValue::Texture { texture_name } => {
                        texture_loader(texture_name)
                            .map(|texture_view| MaterialPropertyValue::Texture(texture_view))
                    }
                    lvl_resource::MaterialPropertyValue::Sampler {
                        address_mode_u,
                        address_mode_v,
                        address_mode_w,
                        mag_filter,
                        min_filter,
                        mipmap_filter,
                        lod_min_clamp,
                        lod_max_clamp,
                        compare,
                        anisotropy_clamp,
                        border_color,
                    } => {
                        let sampler = gfx_ctx.device.create_sampler(&SamplerDescriptor {
                            label: None,
                            address_mode_u: *address_mode_u,
                            address_mode_v: *address_mode_v,
                            address_mode_w: *address_mode_w,
                            mag_filter: *mag_filter,
                            min_filter: *min_filter,
                            mipmap_filter: *mipmap_filter,
                            lod_min_clamp: *lod_min_clamp,
                            lod_max_clamp: *lod_max_clamp,
                            compare: *compare,
                            anisotropy_clamp: *anisotropy_clamp,
                            border_color: *border_color,
                        });
                        Some(MaterialPropertyValue::Sampler(Arc::new(sampler)))
                    }
                    lvl_resource::MaterialPropertyValue::Uniform(kind) => match kind {
                        MaterialPropertyUniformValue::Float(value) => {
                            Some(MaterialPropertyValue::Float(*value))
                        }
                        MaterialPropertyUniformValue::Vec2(value) => {
                            Some(MaterialPropertyValue::Vec2(*value))
                        }
                        MaterialPropertyUniformValue::Vec3(value) => {
                            Some(MaterialPropertyValue::Vec3(*value))
                        }
                        MaterialPropertyUniformValue::Vec4(value) => {
                            Some(MaterialPropertyValue::Vec4(*value))
                        }
                        MaterialPropertyUniformValue::U32(value) => {
                            Some(MaterialPropertyValue::U32(*value))
                        }
                    },
                },
                None => None,
            };

            if let Some(preset_value) = value.as_ref() {
                if !kind.is_compatible(&preset_value) {
                    value = None;
                }
            }

            properties.push(MaterialProperty {
                group: binding.group,
                binding: binding.binding,
                kind,
                value,
            });
        }

        for uniform_member in shader_source.uniform_members() {
            property_name_index_map.insert(uniform_member.name.clone(), properties.len());

            let kind = MaterialPropertyKind::UniformMember {
                offset: uniform_member.offset,
                size: uniform_member.size,
                buffer_index: uniform_member.buffer_index,
            };

            let mut value = match source.properties().get(&uniform_member.name) {
                Some(property) => match &property.value {
                    lvl_resource::MaterialPropertyValue::Texture { .. } => None,
                    lvl_resource::MaterialPropertyValue::Sampler { .. } => None,
                    lvl_resource::MaterialPropertyValue::Uniform(kind) => match kind {
                        MaterialPropertyUniformValue::Float(value) => {
                            Some(MaterialPropertyValue::Float(*value))
                        }
                        MaterialPropertyUniformValue::Vec2(value) => {
                            Some(MaterialPropertyValue::Vec2(*value))
                        }
                        MaterialPropertyUniformValue::Vec3(value) => {
                            Some(MaterialPropertyValue::Vec3(*value))
                        }
                        MaterialPropertyUniformValue::Vec4(value) => {
                            Some(MaterialPropertyValue::Vec4(*value))
                        }
                        MaterialPropertyUniformValue::U32(value) => {
                            Some(MaterialPropertyValue::U32(*value))
                        }
                    },
                },
                None => None,
            };

            if let Some(preset_value) = value.as_ref() {
                if !kind.is_compatible(&preset_value) {
                    value = None;
                }
            }

            properties.push(MaterialProperty {
                group: uniform_structs[uniform_member.buffer_index as usize].group,
                binding: uniform_structs[uniform_member.buffer_index as usize].binding,
                kind,
                value,
            });
        }

        Self {
            shader,
            render_state: source.render_state().clone(),
            uniform_buffers,
            uniform_structs,
            bind_groups: RefCell::new(bind_groups),
            properties,
            property_name_index_map,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn render_state(&self) -> &MaterialRenderState {
        &self.render_state
    }

    pub fn get_property(&self, name: &str) -> Option<&MaterialProperty> {
        self.property_name_index_map
            .get(name)
            .map(|index| &self.properties[*index])
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
        self.bind_groups.borrow_mut()[self.properties[index].group as usize] = None;

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

            for uniform_struct in &self.uniform_structs {
                // user-defined bind groups come after the built-in bind group
                let group = group + 1;

                if uniform_struct.group as usize != group {
                    continue;
                }

                entries.push(BindGroupEntry {
                    binding: uniform_struct.binding as u32,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &self.uniform_buffers[uniform_struct.buffer_index as usize],
                        offset: 0,
                        size: None,
                    }),
                });
            }

            for property in &self.properties {
                // user-defined bind groups come after the built-in bind group
                let group = group + 1;

                if property.group as usize != group {
                    continue;
                }

                match &property.kind {
                    MaterialPropertyKind::UniformMember { .. } => {}
                    _ => {
                        continue;
                    }
                };

                let value = match property.value.as_ref() {
                    Some(value) => value,
                    None => {
                        continue;
                    }
                };

                if let Some(bytes) = value.as_bytes() {
                    self.prepare_uniform_value(bytes, &property.kind, &gfx_ctx.queue);
                }
            }

            loop {
                let property = self.properties.iter().find(|property| {
                    // user-defined bind groups come after the built-in bind group
                    let group = group + 1;

                    if property.group as usize != group {
                        return false;
                    }

                    match property.kind {
                        MaterialPropertyKind::UniformMember { .. } => false,
                        _ => property.binding as usize == entries.len(),
                    }
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

                match &property.kind {
                    MaterialPropertyKind::UniformBuffer { .. } => {
                        if let Some(bytes) = value.as_bytes() {
                            let buffer_binding = self
                                .prepare_uniform_value(bytes, &property.kind, &gfx_ctx.queue)
                                .unwrap();
                            entries.push(BindGroupEntry {
                                binding: property.binding as u32,
                                resource: BindingResource::Buffer(buffer_binding),
                            });
                        }
                    }
                    MaterialPropertyKind::StorageBuffer { .. } => {
                        if let MaterialPropertyValue::StorageBuffer {
                            buffer: source_buffer,
                            offset,
                            size,
                        } = value
                        {
                            entries.push(BindGroupEntry {
                                binding: property.binding as u32,
                                resource: BindingResource::Buffer(BufferBinding {
                                    buffer: source_buffer,
                                    offset: *offset,
                                    size: Some(*size),
                                }),
                            });
                        }
                    }
                    MaterialPropertyKind::Texture => {
                        if let MaterialPropertyValue::Texture(value) = value {
                            entries.push(BindGroupEntry {
                                binding: property.binding as u32,
                                resource: BindingResource::TextureView(value),
                            });
                        }
                    }
                    MaterialPropertyKind::Sampler => {
                        if let MaterialPropertyValue::Sampler(value) = value {
                            entries.push(BindGroupEntry {
                                binding: property.binding as u32,
                                resource: BindingResource::Sampler(value),
                            });
                        }
                    }
                    _ => {}
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
        if data.len() == 0 {
            return None;
        }

        let (offset, size, buffer) =
            if let MaterialPropertyKind::UniformBuffer { buffer_index, .. } = &kind {
                (0, None, &self.uniform_buffers[*buffer_index as usize])
            } else if let MaterialPropertyKind::UniformMember {
                offset,
                size,
                buffer_index,
            } = &kind
            {
                (
                    *offset,
                    Some(*size),
                    &self.uniform_buffers[*buffer_index as usize],
                )
            } else {
                return None;
            };

        if let Some(mut view) =
            queue.write_buffer_with(buffer, offset, BufferSize::new(data.len() as u64).unwrap())
        {
            view.copy_from_slice(data);
        }

        Some(BufferBinding {
            buffer,
            offset,
            size,
        })
    }
}

#[derive(Debug)]
pub struct UniformStruct {
    group: u32,
    binding: u32,
    buffer_index: u32,
}

#[derive(Debug)]
pub struct MaterialProperty {
    group: u32,
    binding: u32,
    kind: MaterialPropertyKind,
    value: Option<MaterialPropertyValue>,
}

impl MaterialProperty {
    pub fn value(&self) -> Option<&MaterialPropertyValue> {
        self.value.as_ref()
    }
}

#[derive(Debug)]
pub enum MaterialPropertyKind {
    UniformBuffer {
        group: u32,
        binding: u32,
        buffer_index: u32,
    },
    UniformMember {
        offset: u64,
        size: NonZeroU64,
        buffer_index: u32,
    },
    StorageBuffer {
        size: NonZeroU64,
    },
    Texture,
    Sampler,
}

impl MaterialPropertyKind {
    pub fn is_compatible(&self, value: &MaterialPropertyValue) -> bool {
        match self {
            Self::UniformBuffer { .. } | Self::UniformMember { .. } => {
                matches!(
                    value,
                    MaterialPropertyValue::Float(_)
                        | MaterialPropertyValue::Vec2(_)
                        | MaterialPropertyValue::Vec3(_)
                        | MaterialPropertyValue::Vec4(_)
                        | MaterialPropertyValue::U32(_)
                )
            }
            Self::StorageBuffer { size, .. } => match value {
                MaterialPropertyValue::StorageBuffer {
                    size: source_size, ..
                } if size.get() == source_size.get() => true,
                _ => false,
            },
            Self::Texture => {
                matches!(value, MaterialPropertyValue::Texture(_))
            }
            Self::Sampler => {
                matches!(value, MaterialPropertyValue::Sampler(_))
            }
        }
    }

    pub fn default_value(&self) -> Option<MaterialPropertyValue> {
        match self {
            MaterialPropertyKind::UniformBuffer { .. } => {
                Some(MaterialPropertyValue::Vec4(Vec4::ZERO))
            }
            MaterialPropertyKind::UniformMember { .. } => {
                Some(MaterialPropertyValue::Vec4(Vec4::ZERO))
            }
            MaterialPropertyKind::StorageBuffer { .. } => None,
            MaterialPropertyKind::Texture => None,
            MaterialPropertyKind::Sampler => None,
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
    U32(u32),

    // storage buffer values
    StorageBuffer {
        buffer: Arc<Buffer>,
        offset: u64,
        size: NonZeroU64,
    },

    // texture values
    Texture(Arc<TextureView>),

    // sampler values
    Sampler(Arc<Sampler>),
}

impl MaterialPropertyValue {
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            MaterialPropertyValue::Float(value) => Some(value.as_bytes()),
            MaterialPropertyValue::Vec2(value) => Some(value.as_bytes()),
            MaterialPropertyValue::Vec3(value) => Some(value.as_bytes()),
            MaterialPropertyValue::Vec4(value) => Some(value.as_bytes()),
            MaterialPropertyValue::U32(value) => Some(value.as_bytes()),
            MaterialPropertyValue::StorageBuffer { .. } => None,
            MaterialPropertyValue::Texture(_) => None,
            MaterialPropertyValue::Sampler(_) => None,
        }
    }
}
