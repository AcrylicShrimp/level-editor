use super::render_command::RenderCommand;
use crate::{
    gfx::{GfxContext, InstanceDataProvider},
    scene::components::StaticMeshRenderer,
};
use lvl_math::Mat4;
use lvl_resource::MeshIndexKind;
use wgpu::IndexFormat;

pub fn build_render_command_static_mesh_renderer<'mesh>(
    gfx_ctx: &GfxContext,
    transform_matrix: &Mat4,
    renderer: &'mesh StaticMeshRenderer,
    instance_data_provider: &InstanceDataProvider,
) -> Option<RenderCommand<'mesh>> {
    let pipeline = renderer.construct_render_pipeline(
        gfx_ctx,
        instance_data_provider.instance_data_size(),
        instance_data_provider.instance_data_attributes(),
    );
    let bind_groups = match renderer.material().construct_bind_groups(gfx_ctx) {
        Some(bind_groups) => bind_groups,
        None => {
            return None;
        }
    };
    let instance_buffer = instance_data_provider.create_instance_buffer(transform_matrix);

    Some(RenderCommand::new(
        renderer
            .material()
            .shader()
            .reflection()
            .builtin_uniform_bind_group,
        pipeline,
        bind_groups,
        renderer.mesh().vertex_buffer().slice(..),
        instance_buffer,
        renderer.mesh().index_buffer().slice(..),
        match renderer.mesh().index_kind() {
            MeshIndexKind::U16 => IndexFormat::Uint16,
            MeshIndexKind::U32 => IndexFormat::Uint32,
        },
        renderer.mesh().vertex_count(),
    ))
}
