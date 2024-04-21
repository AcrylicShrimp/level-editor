use lvl_core::{
    context::{driver::Driver, Context},
    scene::Scene,
};
use winit::window::Window;

use crate::object::make_test_object;

pub struct DriverImpl {}

impl DriverImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl Driver for DriverImpl {
    fn on_init(&mut self, _context: &Context, _window: &Window, scene: &mut Scene) {
        scene.with_proxy(|scene| {
            make_test_object(scene);
        });
    }

    fn on_finish(&mut self, _context: &Context, _window: &Window, _scene: &mut Scene) {}
}
