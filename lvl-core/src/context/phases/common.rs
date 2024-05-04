use crate::scene::{components::Camera, ObjectId, SceneProxy};

struct CameraObject<'scene> {
    pub object_id: ObjectId,
    pub camera: &'scene Camera,
}

pub fn get_all_cameras(scene: &SceneProxy) -> Vec<ObjectId> {
    let mut camera_objects = match scene.find_object_ids_by_component_type::<Camera>() {
        Some(object_ids) => object_ids
            .iter()
            .map(|object_id| CameraObject {
                object_id: *object_id,
                camera: scene
                    .find_object_by_id(*object_id)
                    .unwrap()
                    .find_component_by_type::<Camera>()
                    .unwrap(),
            })
            .collect(),
        None => {
            vec![]
        }
    };

    camera_objects.sort_unstable_by_key(|camera_object| camera_object.camera.order);
    camera_objects
        .iter()
        .map(|camera_object| camera_object.object_id)
        .collect()
}
