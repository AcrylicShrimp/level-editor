use super::render_command::RenderCommand;
use crate::{
    gfx::{GfxContext, InstanceDataProvider},
    scene::components::PmxModelRenderer,
};
use lvl_math::Mat4;
use lvl_resource::PmxModelIndexKind;
use wgpu::IndexFormat;

pub fn build_render_command_pmx_model_renderer<'r>(
    transform_matrix: &Mat4,
    renderer: &'r PmxModelRenderer,
    instance_data_provider: &InstanceDataProvider,
    gfx_ctx: &GfxContext,
) -> Vec<RenderCommand<'r>> {
    let instance_buffer = instance_data_provider.create_instance_buffer(
        transform_matrix,
        &gfx_ctx.per_frame_buffer_pool,
        &gfx_ctx.device,
        &gfx_ctx.queue,
    );

    let model = renderer.model();
    let render_pipelines = renderer.construct_render_pipelines(
        instance_data_provider.instance_data_size(),
        instance_data_provider.instance_data_attributes(),
        gfx_ctx,
    );
    let index_format = match model.index_kind() {
        PmxModelIndexKind::U16 => IndexFormat::Uint16,
        PmxModelIndexKind::U32 => IndexFormat::Uint32,
    };

    let mut commands = Vec::with_capacity(model.elements().len());

    for (index, element) in model.elements().iter().enumerate() {
        let bind_groups = match element.material.construct_bind_groups(gfx_ctx) {
            Some(bind_groups) => bind_groups,
            None => {
                continue;
            }
        };

        commands.push(RenderCommand::new(
            element
                .material
                .shader()
                .reflection()
                .builtin_uniform_bind_group,
            render_pipelines[index].clone(),
            bind_groups,
            instance_buffer.clone(),
            model.vertex_buffer().slice(..),
            model.index_buffer().slice(..),
            index_format,
            element.index_range,
        ));
    }

    commands
}
