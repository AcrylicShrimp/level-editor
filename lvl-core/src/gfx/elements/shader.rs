use crate::gfx::GfxContext;
use lvl_resource::{ShaderBindingElementKind, ShaderSource};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, PipelineLayout, PipelineLayoutDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderStages,
};

use super::ShaderReflection;

#[derive(Debug)]
pub struct Shader {
    module: ShaderModule,
    bind_group_layouts: Vec<BindGroupLayout>,
    pipeline_layout: PipelineLayout,
    reflection: ShaderReflection,
}

impl Shader {
    pub fn load_from_source(source: &ShaderSource, gfx_ctx: &GfxContext) -> Self {
        let max_group = source
            .binding_elements()
            .iter()
            .map(|element| element.group)
            .max()
            .unwrap_or_default();
        let mut bind_group_layouts = Vec::with_capacity(max_group as usize);

        for group in 0..=max_group {
            let mut in_group = source
                .binding_elements()
                .iter()
                .filter(|element| element.group == group)
                .collect::<Vec<_>>();

            in_group.sort_unstable_by_key(|element| element.binding);

            let mut bind_group_layout_entries = Vec::with_capacity(in_group.len());

            for (index, element) in in_group.into_iter().enumerate() {
                if element.binding as usize != index {
                    break;
                }

                let ty = match element.kind {
                    ShaderBindingElementKind::Buffer { size } => BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(size),
                    },
                    ShaderBindingElementKind::Texture {
                        sample_type,
                        view_dimension,
                        multisampled,
                    } => BindingType::Texture {
                        sample_type,
                        view_dimension,
                        multisampled,
                    },
                    ShaderBindingElementKind::Sampler { binding_type } => {
                        BindingType::Sampler(binding_type)
                    }
                };

                bind_group_layout_entries.push(BindGroupLayoutEntry {
                    binding: element.binding,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty,
                    count: None,
                });
            }

            if bind_group_layout_entries.is_empty() {
                break;
            }

            bind_group_layouts.push(gfx_ctx.device.create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    label: None,
                    entries: &bind_group_layout_entries,
                },
            ));
        }

        let layouts = bind_group_layouts.iter().collect::<Vec<_>>();
        let pipeline_layout = gfx_ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &layouts,
                push_constant_ranges: &[],
            });

        let module = gfx_ctx.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.source().into()),
        });

        Self {
            module,
            bind_group_layouts,
            pipeline_layout,
            reflection: ShaderReflection::from_shader_source(source),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn bind_group_layouts(&self) -> &[BindGroupLayout] {
        &self.bind_group_layouts
    }

    pub fn pipeline_layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }

    pub fn reflection(&self) -> &ShaderReflection {
        &self.reflection
    }
}
