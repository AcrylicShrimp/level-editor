use crate::object::{make_camera_object, make_model_object};
use lvl_core::{
    context::{driver::Driver, Context},
    resource::load_resource_file,
    scene::Scene,
};
use lvl_math::Vec4;
use lvl_resource::ResourceFile;
use winit::window::Window;

pub struct DriverImpl {
    resource: Option<ResourceFile>,
}

impl DriverImpl {
    pub fn new() -> Self {
        Self { resource: None }
    }
}

impl Driver for DriverImpl {
    fn on_init(&mut self, _context: &Context, _window: &Window, scene: &mut Scene) {
        {
            let bytes = std::fs::read("./assets/resources.res").unwrap();
            self.resource = Some(load_resource_file(&bytes).unwrap());
        }

        scene.with_proxy(|scene| {
            make_camera_object(
                0,
                Vec4 {
                    x: 0.05,
                    y: 0.05,
                    z: 0.05,
                    w: 1.0,
                },
                scene,
            );
            make_model_object(self.resource.as_ref().unwrap(), "モナ・Mona", scene);
        });
    }

    fn on_finish(&mut self, _context: &Context, _window: &Window, _scene: &mut Scene) {}
}
