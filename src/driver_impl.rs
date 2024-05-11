use crate::object::{make_camera_object, make_light_object, make_pmx_model_renderer};
use lvl_core::{
    context::{driver::Driver, Context},
    resource::load_resource_file,
    scene::{
        components::{LightKind, PmxModelRenderer},
        ObjectId, Scene, Transform,
    },
};
use lvl_math::{Quat, Vec3, Vec4};
use winit::{
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct DriverImpl {
    camera_id: Option<ObjectId>,
    pmx_model_id: Option<ObjectId>,
}

impl DriverImpl {
    pub fn new() -> Self {
        Self {
            camera_id: None,
            pmx_model_id: None,
        }
    }
}

impl Driver for DriverImpl {
    fn on_init(&mut self, context: &Context, _window: &Window, scene: &mut Scene) {
        context
            .input_mut()
            .register_key("W", PhysicalKey::Code(KeyCode::KeyW));
        context
            .input_mut()
            .register_key("S", PhysicalKey::Code(KeyCode::KeyS));
        context
            .input_mut()
            .register_key("A", PhysicalKey::Code(KeyCode::KeyA));
        context
            .input_mut()
            .register_key("D", PhysicalKey::Code(KeyCode::KeyD));

        context
            .input_mut()
            .register_key("Up", PhysicalKey::Code(KeyCode::ArrowUp));
        context
            .input_mut()
            .register_key("Down", PhysicalKey::Code(KeyCode::ArrowDown));
        context
            .input_mut()
            .register_key("Left", PhysicalKey::Code(KeyCode::ArrowLeft));
        context
            .input_mut()
            .register_key("Right", PhysicalKey::Code(KeyCode::ArrowRight));

        context
            .input_mut()
            .register_key("Space", PhysicalKey::Code(KeyCode::Space));

        let resource = {
            let bytes = std::fs::read("./assets/resources.res").unwrap();
            load_resource_file(&bytes).unwrap()
        };

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

            scene.set_transform(
                camera_id,
                Transform::look_at(
                    Vec3::new(0.0, 15.0, -7.0),
                    Vec3::new(0.0, 15.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                ),
            );

            let pmx_model_id = make_pmx_model_renderer(&resource, "モナ・Mona", scene).unwrap();
            self.pmx_model_id = Some(pmx_model_id);

            let pmx_model_object = scene
                .find_object_by_id_mut(self.pmx_model_id.unwrap())
                .unwrap();
            let pmx_model_renderer = pmx_model_object
                .find_component_by_type_mut::<PmxModelRenderer>()
                .unwrap();

            let pmx_model = pmx_model_renderer.model_mut();

            pmx_model.set_morph("+EarPierce", 1f32);
            pmx_model.set_morph("+TKB", 1f32);
            pmx_model.set_morph("+Shoes", 1f32);
            pmx_model.set_morph("+Tebukuro1", 1f32);
            pmx_model.set_morph("+Tebukuro2", 1f32);
            pmx_model.set_morph("+Hat", 1f32);
            pmx_model.set_morph("+ShoulderVeil", 1f32);
            pmx_model.set_morph("+NeckBand", 1f32);
            pmx_model.set_morph("+Tights", 1f32);
            pmx_model.set_morph("+LegAccessory", 1f32);

            make_light_object(
                Vec3::new(10.0, 20.0, 10.0),
                LightKind::Directional {
                    direction: Vec3::new(0.2, -1.0, 0.2).normalized(),
                },
                Vec3::new(1.0, 0.0, 0.0),
                scene,
            );
            make_light_object(
                Vec3::new(-5.0, 1.0, -2.0),
                LightKind::Point,
                Vec3::new(0.0, 0.0, 1.0),
                scene,
            );
        });
    }

    fn on_after_update(&mut self, context: &Context, _window: &Window, scene: &mut Scene) {
        let delta = context.time().delta_time().as_secs_f32();

        scene.with_proxy(|scene| {
            let angle_speed = f32::to_radians(80.0);
            let movement_speed = 4.0;

            let camera = scene.find_object_by_id(self.camera_id.unwrap()).unwrap();
            let mut camera_transform = camera.transform();

            let local_to_world_matrix = scene
                .local_to_world_matrix(self.camera_id.unwrap())
                .unwrap();

            let up = context.input().key("Up").unwrap().is_pressed;
            let down = context.input().key("Down").unwrap().is_pressed;
            let left = context.input().key("Left").unwrap().is_pressed;
            let right = context.input().key("Right").unwrap().is_pressed;

            if up != down {
                let mut basis = Vec4::RIGHT;

                if down {
                    basis = -basis;
                }

                camera_transform.rotation *=
                    Quat::from_axis_angle(Vec3::from_vec4(basis), delta * angle_speed);
            }

            if left != right {
                let mut basis = Vec4::UP * local_to_world_matrix.inversed();

                if right {
                    basis = -basis;
                }

                camera_transform.rotation *=
                    Quat::from_axis_angle(Vec3::from_vec4(basis), delta * angle_speed);
            }

            let w = context.input().key("W").unwrap().is_pressed;
            let s = context.input().key("S").unwrap().is_pressed;
            let d = context.input().key("D").unwrap().is_pressed;
            let a = context.input().key("A").unwrap().is_pressed;

            if w != s {
                let mut forward = Vec4::FORWARD * &local_to_world_matrix;

                if s {
                    forward = -forward;
                }

                camera_transform.position += Vec3::from_vec4(forward) * delta * movement_speed;
            }

            if a != d {
                let mut right = Vec4::RIGHT * &local_to_world_matrix;

                if a {
                    right = -right;
                }

                camera_transform.position += Vec3::from_vec4(right) * delta * movement_speed;
            }

            scene.set_transform(self.camera_id.unwrap(), camera_transform);
        });

        scene.with_proxy(|scene| {
            let pmx_model_object = scene
                .find_object_by_id_mut(self.pmx_model_id.unwrap())
                .unwrap();
            let pmx_model_renderer = pmx_model_object
                .find_component_by_type_mut::<PmxModelRenderer>()
                .unwrap();
            pmx_model_renderer.model_mut().set_morph(
                "あ",
                if context.input().key("Space").unwrap().is_pressed {
                    1f32
                } else {
                    0f32
                },
            );
        });
    }
}
