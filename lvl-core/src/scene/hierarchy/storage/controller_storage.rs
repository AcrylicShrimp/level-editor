use crate::scene::{Controller, ObjectId, SceneProxy};
use std::collections::{hash_map::Entry, HashMap, HashSet};

pub struct ControllerStorage {
    controllers: HashMap<ObjectId, Box<dyn Controller>>,
    on_update_hooked_controllers: HashSet<ObjectId>,
    on_late_update_hooked_controllers: HashSet<ObjectId>,
}

impl ControllerStorage {
    pub(crate) fn new() -> Self {
        Self {
            controllers: HashMap::new(),
            on_update_hooked_controllers: HashSet::new(),
            on_late_update_hooked_controllers: HashSet::new(),
        }
    }

    pub(crate) fn find_controller(&mut self, id: ObjectId) -> Option<&mut dyn Controller> {
        self.controllers.get_mut(&id).map(|c| c.as_mut())
    }

    pub(crate) fn attach_controller(
        &mut self,
        id: ObjectId,
        controller: Box<dyn Controller>,
        scene: &mut SceneProxy,
    ) {
        match self.controllers.entry(id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().on_destroy(id, scene);
                entry.insert(controller);
                entry.get_mut().on_ready(id, scene);
            }
            Entry::Vacant(entry) => {
                entry.insert(controller).on_ready(id, scene);
            }
        }
    }

    pub(crate) fn detach_controller(&mut self, id: ObjectId, scene: &mut SceneProxy) {
        self.on_update_hooked_controllers.remove(&id);
        self.on_late_update_hooked_controllers.remove(&id);

        if let Some(mut controller) = self.controllers.remove(&id) {
            controller.on_destroy(id, scene);
        }
    }

    pub(crate) fn listen_on_update(&mut self, id: ObjectId) {
        if self.controllers.contains_key(&id) {
            self.on_update_hooked_controllers.insert(id);
        }
    }

    pub(crate) fn unlisten_on_update(&mut self, id: ObjectId) {
        self.on_update_hooked_controllers.remove(&id);
    }

    pub(crate) fn listen_on_late_update(&mut self, id: ObjectId) {
        if self.controllers.contains_key(&id) {
            self.on_late_update_hooked_controllers.insert(id);
        }
    }

    pub(crate) fn unlisten_on_late_update(&mut self, id: ObjectId) {
        self.on_late_update_hooked_controllers.remove(&id);
    }

    pub(crate) fn invoke_on_active(&mut self, id: ObjectId, scene: &mut SceneProxy) {
        if let Some(controller) = self.controllers.get_mut(&id) {
            controller.on_active(id, scene);
        }
    }

    pub(crate) fn invoke_on_inactive(&mut self, id: ObjectId, scene: &mut SceneProxy) {
        if let Some(controller) = self.controllers.get_mut(&id) {
            controller.on_inactive(id, scene);
        }
    }

    pub(crate) fn invoke_on_update(&mut self, scene: &mut SceneProxy) {
        for id in &self.on_update_hooked_controllers {
            if let Some(controller) = self.controllers.get_mut(id) {
                controller.on_update(*id, scene);
            }
        }
    }

    pub(crate) fn invoke_on_late_update(&mut self, scene: &mut SceneProxy) {
        for id in &self.on_late_update_hooked_controllers {
            if let Some(controller) = self.controllers.get_mut(id) {
                controller.on_late_update(*id, scene);
            }
        }
    }
}
