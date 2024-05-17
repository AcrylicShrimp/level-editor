pub mod driver;
pub mod input;
pub mod phases;
pub mod screen_size;
pub mod time;

use self::{input::Input, screen_size::ScreenSize, time::Time};
use crate::gfx::GfxContext;
use std::{
    cell::{Ref, RefCell, RefMut},
    sync::Arc,
};
use winit::dpi::PhysicalSize;

pub struct Context<'window> {
    gfx_ctx: Arc<GfxContext<'window>>,
    screen_size: RefCell<ScreenSize>,
    input: RefCell<Input>,
    time: RefCell<Time>,
}

impl<'window> Context<'window> {
    pub(crate) fn new(gfx_ctx: GfxContext<'window>, screen_size: PhysicalSize<u32>) -> Self {
        Self {
            gfx_ctx: Arc::new(gfx_ctx),
            screen_size: RefCell::new(ScreenSize::new(screen_size)),
            input: RefCell::new(Input::new()),
            time: RefCell::new(Time::new()),
        }
    }

    pub fn gfx_ctx(&self) -> &GfxContext<'window> {
        &self.gfx_ctx
    }

    pub fn screen_size(&self) -> Ref<ScreenSize> {
        self.screen_size.borrow()
    }

    pub fn screen_size_mut(&self) -> RefMut<ScreenSize> {
        self.screen_size.borrow_mut()
    }

    pub fn input(&self) -> Ref<Input> {
        self.input.borrow()
    }

    pub fn input_mut(&self) -> RefMut<Input> {
        self.input.borrow_mut()
    }

    pub fn time(&self) -> Ref<Time> {
        self.time.borrow()
    }

    pub fn time_mut(&self) -> RefMut<Time> {
        self.time.borrow_mut()
    }

    pub(crate) fn update_screen_size(&self, screen_size: PhysicalSize<u32>) {
        self.screen_size.borrow_mut().set_size(screen_size);
    }
}
