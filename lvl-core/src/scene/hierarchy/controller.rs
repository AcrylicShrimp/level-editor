use super::{ObjectId, SceneProxy};
use std::any::Any;

pub trait Controller: Any {
    fn on_ready(&mut self, _object_id: ObjectId, _scene: &mut SceneProxy) {}
    fn on_destroy(&mut self, _object_id: ObjectId, _scene: &mut SceneProxy) {}
    fn on_active(&mut self, _object_id: ObjectId, _scene: &mut SceneProxy) {}
    fn on_inactive(&mut self, _object_id: ObjectId, _scene: &mut SceneProxy) {}
    fn on_update(&mut self, _object_id: ObjectId, _scene: &mut SceneProxy) {}
    fn on_late_update(&mut self, _object_id: ObjectId, _scene: &mut SceneProxy) {}
    fn on_event(
        &mut self,
        _event: &str,
        _param: &dyn Any,
        _object_id: ObjectId,
        _scene: &mut SceneProxy,
    ) {
    }
}
