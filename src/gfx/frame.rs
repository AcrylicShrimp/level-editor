use wgpu::{
    Color, CommandBuffer, CommandEncoder, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    TextureView,
};

#[derive(Debug, Clone)]
pub enum ClearMode {
    Keep,
    All {
        color: Color,
        depth: f32,
        stencil: u32,
    },
    DepthStencilOnly {
        depth: f32,
        stencil: u32,
    },
}

#[derive(Debug)]
pub struct RenderPassTarget<'tex> {
    pub view: &'tex TextureView,
    pub writable: bool,
}

pub struct Frame {
    cmd_encoder: CommandEncoder,
}

impl Frame {
    pub fn new(cmd_encoder: CommandEncoder) -> Self {
        Self { cmd_encoder }
    }

    pub fn finish(self) -> CommandBuffer {
        self.cmd_encoder.finish()
    }

    pub fn begin_render_pass<'pass, 'tex: 'pass, 'a: 'pass>(
        &'a mut self,
        clear_mode: ClearMode,
        color_targets: &[Option<RenderPassTarget<'tex>>],
        depth_stencil_target: Option<RenderPassTarget<'tex>>,
    ) -> RenderPass<'pass> {
        let color_attachments = color_targets
            .iter()
            .map(|target| {
                target.as_ref().map(|t| RenderPassColorAttachment {
                    view: t.view,
                    resolve_target: None,
                    ops: Operations {
                        load: match clear_mode {
                            ClearMode::Keep => LoadOp::Load,
                            ClearMode::All { color, .. } => LoadOp::Clear(Color {
                                r: color.r as f64,
                                g: color.g as f64,
                                b: color.b as f64,
                                a: color.a as f64,
                            }),
                            ClearMode::DepthStencilOnly { .. } => LoadOp::Load,
                        },
                        store: if t.writable {
                            StoreOp::Store
                        } else {
                            StoreOp::Discard
                        },
                    },
                })
            })
            .collect::<Vec<_>>();

        let depth_stencil_attachment =
            depth_stencil_target
                .as_ref()
                .map(|t| RenderPassDepthStencilAttachment {
                    view: t.view,
                    depth_ops: Some(Operations {
                        load: match clear_mode {
                            ClearMode::Keep => LoadOp::Load,
                            ClearMode::All { depth, .. } => LoadOp::Clear(depth),
                            ClearMode::DepthStencilOnly { depth, .. } => LoadOp::Clear(depth),
                        },
                        store: if t.writable {
                            StoreOp::Store
                        } else {
                            StoreOp::Discard
                        },
                    }),
                    stencil_ops: Some(Operations {
                        load: match clear_mode {
                            ClearMode::Keep => LoadOp::Load,
                            ClearMode::All { stencil, .. } => LoadOp::Clear(stencil),
                            ClearMode::DepthStencilOnly { stencil, .. } => LoadOp::Clear(stencil),
                        },
                        store: if t.writable {
                            StoreOp::Store
                        } else {
                            StoreOp::Discard
                        },
                    }),
                });

        self.cmd_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("[Frame] begin_render_pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}
