use lvl_core::scene::{
    components::{Camera, CameraClearMode},
    SceneProxy,
};
use lvl_math::Vec4;

pub fn make_camera_object(order: i64, clear_color: Vec4, scene: &mut SceneProxy) {
    let id = scene.create_object();
    scene.add_component(
        id,
        Camera {
            order,
            clear_mode: CameraClearMode::All { color: clear_color },
        },
    );
}
