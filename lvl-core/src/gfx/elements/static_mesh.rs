use super::MeshLayout;
use wgpu::{util::DeviceExt, Buffer};

#[derive(Debug)]
pub struct StaticMesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    layout: MeshLayout,
    vertex_count: u32,
}

impl StaticMesh {
    // pub fn load(source: &MeshSource, gfx_ctx: &GfxContext) -> Self {
    //     // gfx_ctx.device.create_buffer_init(desc)
    //     todo!()
    // }
}
