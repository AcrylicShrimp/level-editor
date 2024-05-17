use crate::scene::{
    components::{UIElement, UIScaler},
    HierarchyStorage, ObjectStorage,
};
use lvl_math::Vec2;

pub(crate) fn mark_root_ui_scaler_dirty(
    object_storage: &ObjectStorage,
    hierarchy_storage: &mut HierarchyStorage,
) {
    let scalers = match object_storage.object_ids_with_component::<UIScaler>() {
        Some(scalers) => scalers,
        None => {
            return;
        }
    };

    for id in scalers {
        if hierarchy_storage.parent(*id).is_none() {
            hierarchy_storage.set_dirty(*id);
        }
    }
}

pub(crate) fn broadcast_ui_scaler_dirty(
    object_storage: &ObjectStorage,
    hierarchy_storage: &mut HierarchyStorage,
) {
    let scalers = match object_storage.object_ids_with_component::<UIScaler>() {
        Some(scalers) => scalers,
        None => {
            return;
        }
    };

    for id in scalers {
        let object = object_storage.get(*id).unwrap();

        for scaler in object.find_components_by_type::<UIScaler>() {
            if scaler.is_dirty() {
                hierarchy_storage.set_dirty(*id);
            }
        }
    }
}

pub(crate) fn update_ui(
    object_storage: &mut ObjectStorage,
    hierarchy_storage: &mut HierarchyStorage,
    screen_width: u32,
    screen_height: u32,
) {
    let mut id_with_index = Vec::new();

    if let Some(scalers) = object_storage.object_ids_with_component::<UIScaler>() {
        for id in scalers {
            let index = hierarchy_storage.index(*id);
            id_with_index.push((index, *id));
        }
    }

    if let Some(elements) = object_storage.object_ids_with_component::<UIElement>() {
        for id in elements {
            let index = hierarchy_storage.index(*id);
            id_with_index.push((index, *id));
        }
    }

    id_with_index.sort_unstable_by_key(|(index, _)| *index);

    for (_, id) in id_with_index {
        let mut parent_size = None;

        for parent_id in hierarchy_storage.parents(id) {
            let object = object_storage.get(*parent_id).unwrap();

            if let Some(parent_element) = object.find_component_by_type::<UIElement>() {
                parent_size = Some(parent_element.size());
                break;
            }
        }

        let mut parent_size =
            parent_size.unwrap_or(Vec2::new(screen_width as f32, screen_height as f32));

        let object = object_storage.get_mut(id).unwrap();
        let is_object_dirty = hierarchy_storage.is_current_frame_dirty(id);

        if let Some(scaler) = object.find_component_by_type_mut::<UIScaler>() {
            if is_object_dirty || scaler.is_dirty() {
                parent_size = scaler.compute_size(parent_size);
            }
        }

        if let Some(element) = object.find_component_by_type_mut::<UIElement>() {
            if is_object_dirty || element.is_dirty() {
                let transform = hierarchy_storage.matrix(id);
                element.compute_properties(parent_size, transform);
            }
        }
    }
}
