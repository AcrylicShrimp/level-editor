use std::cell::RefMut;
use wgpu::{BindGroup, BufferSlice, IndexFormat, RenderPass, RenderPipeline};

pub struct RenderCommand<'a> {
    pipeline: RefMut<'a, Option<RenderPipeline>>,
    bind_groups: RefMut<'a, Vec<Option<BindGroup>>>,
    vertex_buffer_slice: BufferSlice<'a>,
    index_buffer_slice: BufferSlice<'a>,
    index_format: IndexFormat,
    count: u32,
}

impl<'a> RenderCommand<'a> {
    pub fn new(
        pipeline: RefMut<'a, Option<RenderPipeline>>,
        bind_groups: RefMut<'a, Vec<Option<BindGroup>>>,
        vertex_buffer_slice: BufferSlice<'a>,
        index_buffer_slice: BufferSlice<'a>,
        index_format: IndexFormat,
        count: u32,
    ) -> Self {
        Self {
            pipeline,
            bind_groups,
            vertex_buffer_slice,
            index_buffer_slice,
            index_format,
            count,
        }
    }

    pub fn render<'pass>(&'a self, render_pass: &'pass mut RenderPass<'a>)
    where
        'a: 'pass,
    {
        render_pass.set_pipeline(self.pipeline.as_ref().unwrap());

        for (group, bind_group) in self.bind_groups.iter().enumerate() {
            let bind_group = match bind_group {
                Some(bind_group) => bind_group,
                None => {
                    return;
                }
            };

            render_pass.set_bind_group(group as u32, bind_group, &[]);
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer_slice);
        render_pass.set_index_buffer(self.index_buffer_slice, self.index_format);

        render_pass.draw_indexed(0..self.count, 0, 0..1);
    }
}
