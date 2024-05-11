use crate::scene::{Component, SceneProxy};
use lvl_math::{Mat4, Vec3};

pub struct CollectedItem<'a, T: Component> {
    pub component: &'a T,
    pub transform_matrix: &'a Mat4,
}

impl<'a, T: Component> CollectedItem<'a, T> {
    pub fn world_position(&self) -> Vec3 {
        self.transform_matrix.split_translation()
    }
}

pub fn collect_components<'a, T: Component>(scene: &'a SceneProxy) -> Vec<CollectedItem<'a, T>> {
    let ids = match scene.find_object_ids_by_component_type::<T>() {
        Some(ids) => ids,
        None => return vec![],
    };

    let mut components = Vec::new();

    for id in ids {
        let transform_matrix = scene.transform_matrix(*id).unwrap();
        let object = scene.find_object_by_id(*id).unwrap();

        for component in object.find_components_by_type::<T>() {
            components.push(CollectedItem {
                component,
                transform_matrix,
            });
        }
    }

    components
}
