use super::render_command::RenderCommand;
use crate::{gfx::GfxContext, scene::components::StaticMeshRenderer};
use lvl_resource::MeshIndexKind;
use wgpu::IndexFormat;

pub fn build_render_command_static_mesh_renderer<'mesh>(
    gfx_ctx: &GfxContext,
    renderer: &'mesh StaticMeshRenderer,
) -> Option<RenderCommand<'mesh>> {
    let pipeline = renderer.construct_render_pipeline(gfx_ctx);
    let bind_groups = match renderer.material().construct_bind_groups(gfx_ctx) {
        Some(bind_groups) => bind_groups,
        None => {
            return None;
        }
    };

    Some(RenderCommand::new(
        pipeline,
        bind_groups,
        renderer.mesh().vertex_buffer().slice(..),
        renderer.mesh().index_buffer().slice(..),
        match renderer.mesh().index_kind() {
            MeshIndexKind::U16 => IndexFormat::Uint16,
            MeshIndexKind::U32 => IndexFormat::Uint32,
        },
        renderer.mesh().vertex_count(),
    ))
}
