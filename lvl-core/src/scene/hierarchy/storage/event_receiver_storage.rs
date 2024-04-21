use super::ControllerStorage;
use crate::scene::{ObjectId, SceneProxy};
use std::{
    any::Any,
    collections::{BTreeSet, HashMap},
};

pub struct EventReceiverStorage {
    event_to_object_ids: HashMap<String, BTreeSet<ObjectId>>,
    object_id_to_events: HashMap<ObjectId, Vec<String>>,
}

impl EventReceiverStorage {
    pub(crate) fn new() -> Self {
        Self {
            event_to_object_ids: HashMap::new(),
            object_id_to_events: HashMap::new(),
        }
    }

    pub(crate) fn listen(&mut self, event: String, object_id: ObjectId) {
        self.event_to_object_ids
            .entry(event.clone())
            .or_default()
            .insert(object_id);
        self.object_id_to_events
            .entry(object_id)
            .or_default()
            .push(event);
    }

    pub(crate) fn unlisten(&mut self, event: String, object_id: ObjectId) {
        if let Some(object_ids) = self.event_to_object_ids.get_mut(&event) {
            object_ids.remove(&object_id);
        }

        if let Some(events) = self.object_id_to_events.get_mut(&object_id) {
            if let Some(index) = events.iter().position(|e| e.as_str() == event) {
                events.swap_remove(index);
            }
        }
    }

    pub(crate) fn unlisten_all(&mut self, object_id: ObjectId) {
        if let Some(events) = self.object_id_to_events.remove(&object_id) {
            for event in events {
                if let Some(object_ids) = self.event_to_object_ids.get_mut(&event) {
                    object_ids.remove(&object_id);
                }
            }
        }
    }

    pub(crate) fn emit(
        &self,
        event: &str,
        param: &dyn Any,
        scene: &mut SceneProxy,
        controller_storage: &mut ControllerStorage,
    ) {
        if let Some(object_ids) = self.event_to_object_ids.get(event) {
            for object_id in object_ids {
                if let Some(controller) = controller_storage.find_controller(*object_id) {
                    controller.on_event(event, param, *object_id, scene);
                }
            }
        }
    }
}
