use crate::gfx::BufferSlicer;
use std::{cell::RefMut, sync::Arc};
use wgpu::{BindGroup, BufferSlice, IndexFormat, RenderPass, RenderPipeline};

pub struct RenderCommand<'a> {
    builtin_uniform_bind_group: Option<u32>,
    pipeline: Arc<RenderPipeline>,
    bind_groups: RefMut<'a, Vec<Option<BindGroup>>>,
    instance_buffer: BufferSlicer,
    vertex_buffer_slice: BufferSlice<'a>,
    index_buffer_slice: BufferSlice<'a>,
    index_format: IndexFormat,
    index_range: (u32, u32),
}

impl<'a> RenderCommand<'a> {
    pub fn new(
        builtin_uniform_bind_group: Option<u32>,
        pipeline: Arc<RenderPipeline>,
        bind_groups: RefMut<'a, Vec<Option<BindGroup>>>,
        instance_buffer: BufferSlicer,
        vertex_buffer_slice: BufferSlice<'a>,
        index_buffer_slice: BufferSlice<'a>,
        index_format: IndexFormat,
        index_range: (u32, u32),
    ) -> Self {
        Self {
            builtin_uniform_bind_group,
            pipeline,
            bind_groups,
            vertex_buffer_slice,
            instance_buffer,
            index_buffer_slice,
            index_format,
            index_range,
        }
    }

    pub fn render<'pass>(
        &'a self,
        render_pass: &'pass mut RenderPass<'a>,
        builtin_bind_group: &'a BindGroup,
    ) where
        'a: 'pass,
    {
        render_pass.set_pipeline(&self.pipeline);

        if let Some(builtin_uniform_bind_group) = self.builtin_uniform_bind_group {
            render_pass.set_bind_group(builtin_uniform_bind_group, builtin_bind_group, &[]);
        }

        for (group, bind_group) in self.bind_groups.iter().enumerate() {
            // user-defined bind groups come after the built-in bind group
            let group = group + 1;
            let bind_group = match bind_group {
                Some(bind_group) => bind_group,
                None => {
                    return;
                }
            };

            render_pass.set_bind_group(group as u32, bind_group, &[]);
        }

        render_pass.set_vertex_buffer(0, self.instance_buffer.slice());
        render_pass.set_vertex_buffer(1, self.vertex_buffer_slice);
        render_pass.set_index_buffer(self.index_buffer_slice, self.index_format);

        render_pass.draw_indexed(self.index_range.0..self.index_range.1, 0, 0..1);
    }
}
