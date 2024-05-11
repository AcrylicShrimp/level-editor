use lvl_core::{
    gfx::elements::{MaterialPropertyValue, PmxModel},
    scene::{
        components::{
            Camera, CameraClearMode, CameraProjectionMode, Light, LightKind, PmxModelRenderer,
        },
        ObjectId, SceneProxy, Transform,
    },
};
use lvl_math::{Vec3, Vec4};
use lvl_resource::{PmxModelSource, ResourceFile};

pub fn make_camera_object(order: i64, clear_color: Vec4, scene: &mut SceneProxy) -> ObjectId {
    let id = scene.create_object();

    scene.add_component(
        id,
        Camera {
            order,
            clear_mode: CameraClearMode::All { color: clear_color },
            projection_mode: CameraProjectionMode::Perspective {
                fov: (60.0f32).to_radians(),
                near: 0.1,
                far: 100.0,
            },
        },
    );

    id
}

pub fn make_pmx_model_renderer(
    resource: &ResourceFile,
    name: &str,
    scene: &mut SceneProxy,
) -> Option<ObjectId> {
    let pmx_model_source = resource.find::<PmxModelSource>(name)?;
    let mut pmx_model =
        PmxModel::load_from_source(resource, pmx_model_source, scene.context().gfx_ctx());

    for element in pmx_model.elements_mut() {
        element
            .material
            .set_property("light_smooth", MaterialPropertyValue::Float(0.1));
        element.material.set_property(
            "light_color",
            MaterialPropertyValue::Vec3(Vec3::new(1.0, 1.0, 1.0)),
        );
        element.material.set_property(
            "light_direction",
            MaterialPropertyValue::Vec3(Vec3::new(0.2, -1.0, 0.2).normalized()),
        );
    }

    let id = scene.create_object();
    scene.add_component(id, PmxModelRenderer::new(pmx_model));
    Some(id)
}

pub fn make_light_object(
    position: Vec3,
    kind: LightKind,
    light_color: Vec3,
    scene: &mut SceneProxy,
) -> ObjectId {
    let id = scene.create_object();

    scene.set_transform(
        id,
        Transform {
            position,
            ..Default::default()
        },
    );
    scene.add_component(id, Light { kind, light_color });

    id
}
