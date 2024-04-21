use lvl_core::scene::{Component, Controller, ObjectId, SceneProxy};
use std::any::Any;

pub fn make_test_object(scene: &mut SceneProxy) {
    let id = scene.create_object();
    scene.add_component(id, TestComponent);
    scene.attach_controller(id, TestController);
}

pub struct TestComponent;

impl Component for TestComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct TestController;

impl Controller for TestController {
    fn on_ready(&mut self, object_id: ObjectId, scene: &mut SceneProxy) {
        println!("[on_ready]: {:?}", object_id);
        scene.listen_on_update(object_id);
    }

    fn on_destroy(&mut self, object_id: ObjectId, _scene: &mut SceneProxy) {
        println!("[on_destroy]: {:?}", object_id);
    }

    fn on_update(&mut self, object_id: ObjectId, _scene: &mut SceneProxy) {
        println!("[on_update]: {:?}", object_id);
    }
}
