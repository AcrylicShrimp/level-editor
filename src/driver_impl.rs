use crate::object::make_camera_object;
use lvl_core::{
    context::{driver::Driver, Context},
    scene::Scene,
};
use lvl_math::Vec4;
use winit::window::Window;

pub struct DriverImpl {}

impl DriverImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl Driver for DriverImpl {
    fn on_init(&mut self, _context: &Context, _window: &Window, scene: &mut Scene) {
        scene.with_proxy(|scene| {
            make_camera_object(
                0,
                Vec4 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                scene,
            );
            make_camera_object(
                1,
                Vec4 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                    w: 1.0,
                },
                scene,
            );
        });
    }

    fn on_finish(&mut self, _context: &Context, _window: &Window, _scene: &mut Scene) {}
}
