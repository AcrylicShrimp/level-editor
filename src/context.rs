pub mod phase;

use crate::gfx::GfxContext;
use std::sync::Arc;

pub struct Context<'window> {
    gfx_ctx: Arc<GfxContext<'window>>,
}

impl<'window> Context<'window> {
    pub fn new(gfx_ctx: GfxContext<'window>) -> Self {
        let gfx_ctx = Arc::new(gfx_ctx);

        Self { gfx_ctx }
    }

    pub fn gfx_ctx(&self) -> &GfxContext<'window> {
        &self.gfx_ctx
    }
}
