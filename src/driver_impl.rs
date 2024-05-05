use crate::object::{make_camera_object, make_model_object};
use lvl_core::{
    context::{driver::Driver, Context},
    resource::load_resource_file,
    scene::{ObjectId, Scene, Transform},
};
use lvl_math::{Vec3, Vec4};
use lvl_resource::ResourceFile;
use winit::window::Window;

pub struct DriverImpl {
    resource: Option<ResourceFile>,
    camera_id: Option<ObjectId>,
}

impl DriverImpl {
    pub fn new() -> Self {
        Self {
            resource: None,
            camera_id: None,
        }
    }
}

impl Driver for DriverImpl {
    fn on_init(&mut self, _context: &Context, _window: &Window, scene: &mut Scene) {
        {
            let bytes = std::fs::read("./assets/resources.res").unwrap();
            self.resource = Some(load_resource_file(&bytes).unwrap());
        }

        scene.with_proxy(|scene| {
            let camera_id = make_camera_object(
                0,
                Vec4 {
                    x: 0.05,
                    y: 0.05,
                    z: 0.05,
                    w: 1.0,
                },
                scene,
            );
            self.camera_id = Some(camera_id);

            make_model_object(self.resource.as_ref().unwrap(), "モナ・Mona", scene);

            scene.set_transform(
                camera_id,
                Transform::look_at(
                    Vec3::new(0.0, 15.0, -7.0),
                    Vec3::new(0.0, 15.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                ),
            );
        });
    }

    fn on_finish(&mut self, _context: &Context, _window: &Window, _scene: &mut Scene) {}
}
