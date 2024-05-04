use lvl_core::{
    gfx::elements::{Material, StaticMesh},
    scene::{
        components::{Camera, CameraClearMode, StaticMeshRenderer},
        ObjectId, SceneProxy, Transform,
    },
};
use lvl_math::Vec4;
use lvl_resource::{ModelElement, ModelSource, ResourceFile};

pub fn make_camera_object(order: i64, clear_color: Vec4, scene: &mut SceneProxy) -> ObjectId {
    let id = scene.create_object();

    scene.add_component(
        id,
        Camera {
            order,
            clear_mode: CameraClearMode::All { color: clear_color },
        },
    );

    id
}

pub fn make_model_object(resource: &ResourceFile, name: &str, scene: &mut SceneProxy) -> ObjectId {
    let model = resource.find::<ModelSource>(name).unwrap();
    let element_objects = model
        .elements()
        .iter()
        .map(|element| make_element_object(resource, element, scene))
        .collect::<Vec<_>>();

    for element in model.elements() {
        if let Some(parent_index) = element.parent_index {
            scene.set_parent(
                element_objects[element.index as usize],
                Some(element_objects[parent_index as usize]),
            );
        }
    }

    element_objects[model.root_element_index() as usize]
}

fn make_element_object(
    resource: &ResourceFile,
    element: &ModelElement,
    scene: &mut SceneProxy,
) -> ObjectId {
    let id = scene.create_object();

    scene.set_transform(
        id,
        Transform {
            position: element.transform.position,
            rotation: element.transform.rotation,
            scale: element.transform.scale,
        },
    );

    for visible_part in &element.visible_parts {
        let material_source = resource.find(&visible_part.material_name).unwrap();
        let mesh_source = resource.find(&visible_part.mesh_name).unwrap();

        let material =
            Material::load_from_source(resource, material_source, scene.context().gfx_ctx());
        let mesh = StaticMesh::load_from_source(mesh_source, scene.context().gfx_ctx());

        let visible_part_id = scene.create_object();
        scene.add_component(visible_part_id, StaticMeshRenderer::new(mesh, material));
        scene.set_parent(visible_part_id, Some(id));
    }

    id
}
