pub mod driver;
pub mod phases;

use crate::gfx::GfxContext;
use std::{cell::RefCell, sync::Arc};
use winit::dpi::PhysicalSize;

pub struct Context<'window> {
    gfx_ctx: Arc<GfxContext<'window>>,
    screen_size: RefCell<PhysicalSize<u32>>,
}

impl<'window> Context<'window> {
    pub(crate) fn new(gfx_ctx: GfxContext<'window>, screen_size: PhysicalSize<u32>) -> Self {
        Self {
            gfx_ctx: Arc::new(gfx_ctx),
            screen_size: RefCell::new(screen_size),
        }
    }

    pub fn gfx_ctx(&self) -> &GfxContext<'window> {
        &self.gfx_ctx
    }

    pub fn screen_size(&self) -> PhysicalSize<u32> {
        *self.screen_size.borrow()
    }

    pub fn update_screen_size(&self, screen_size: PhysicalSize<u32>) {
        *self.screen_size.borrow_mut() = screen_size;
    }
}
