use super::{
    AnyComponent, Component, ComponentId, ComponentIdAllocator, Controller, HierarchyStorage,
    Object, ObjectId, ObjectIdAllocator, ObjectSiblingIter, ObjectStorage, Transform,
};
use crate::context::Context;
use lvl_math::Mat4;
use std::{
    any::{Any, TypeId},
    collections::HashSet,
};
use winit::window::Window;

pub(crate) enum SceneActionItem {
    RemoveObject {
        object_id: ObjectId,
    },
    TriggerOnActive {
        object_id: ObjectId,
    },
    TriggerOnInactive {
        object_id: ObjectId,
    },
    AttachController {
        object_id: ObjectId,
        controller: Box<dyn Controller>,
    },
    DetachController {
        object_id: ObjectId,
    },
    ListenOnUpdate {
        object_id: ObjectId,
    },
    UnlistenOnUpdate {
        object_id: ObjectId,
    },
    ListenOnLateUpdate {
        object_id: ObjectId,
    },
    UnlistenOnLateUpdate {
        object_id: ObjectId,
    },
    ListenEvent {
        event: String,
        object_id: ObjectId,
    },
    UnlistenEvent {
        event: String,
        object_id: ObjectId,
    },
    UnlistenEventAll {
        object_id: ObjectId,
    },
    EmitEvent {
        event: String,
        param: Box<dyn Any>,
    },
}

pub(crate) struct SceneActionResult {
    pub action_queue: Vec<SceneActionItem>,
}

pub struct SceneProxy<'scene, 'window> {
    context: &'scene Context<'window>,
    window: &'window Window,
    object_id_allocator: &'scene mut ObjectIdAllocator,
    component_id_allocator: &'scene mut ComponentIdAllocator,
    object_storage: &'scene mut ObjectStorage,
    hierarchy_storage: &'scene mut HierarchyStorage,
    action_queue: Vec<SceneActionItem>,
}

impl<'scene, 'window> SceneProxy<'scene, 'window> {
    pub(crate) fn new(
        context: &'scene Context<'window>,
        window: &'window Window,
        object_id_allocator: &'scene mut ObjectIdAllocator,
        component_id_allocator: &'scene mut ComponentIdAllocator,
        object_storage: &'scene mut ObjectStorage,
        hierarchy_storage: &'scene mut HierarchyStorage,
    ) -> Self {
        Self {
            context,
            window,
            object_id_allocator,
            component_id_allocator,
            object_storage,
            hierarchy_storage,
            action_queue: Vec::new(),
        }
    }

    pub(crate) fn into_result(self) -> SceneActionResult {
        SceneActionResult {
            action_queue: self.action_queue,
        }
    }

    pub fn context(&self) -> &Context<'window> {
        self.context
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub(crate) fn object_id_allocator_mut(&mut self) -> &mut ObjectIdAllocator {
        self.object_id_allocator
    }

    pub(crate) fn object_storage(&self) -> &ObjectStorage {
        self.object_storage
    }

    pub(crate) fn object_storage_mut(&mut self) -> &mut ObjectStorage {
        self.object_storage
    }

    pub(crate) fn hierarchy_storage(&self) -> &HierarchyStorage {
        self.hierarchy_storage
    }

    pub(crate) fn hierarchy_storage_mut(&mut self) -> &mut HierarchyStorage {
        self.hierarchy_storage
    }

    pub fn find_object_by_id(&self, id: ObjectId) -> Option<&Object> {
        self.object_storage.get(id)
    }

    pub fn find_object_by_id_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.object_storage.get_mut(id)
    }

    pub fn find_object_ids_by_component_type<T>(&self) -> Option<&HashSet<ObjectId>>
    where
        T: Component,
    {
        self.object_storage.object_ids_with_component::<T>()
    }

    pub fn is_active(&self, object_id: ObjectId) -> bool {
        if !self.object_storage.is_exists(object_id) {
            return false;
        }

        self.hierarchy_storage.is_active(object_id)
    }

    pub fn is_active_self(&self, object_id: ObjectId) -> bool {
        if !self.object_storage.is_exists(object_id) {
            return false;
        }

        self.hierarchy_storage.is_active_self(object_id)
    }

    pub fn name(&self, object_id: ObjectId) -> &str {
        self.hierarchy_storage.name(object_id)
    }

    pub fn name_interned(&self, object_id: ObjectId) -> string_interner::DefaultSymbol {
        self.hierarchy_storage.name_interned(object_id)
    }

    pub fn local_to_world_matrix(&self, object_id: ObjectId) -> Option<Mat4> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        let mut matrix = self
            .object_storage
            .get(object_id)
            .unwrap()
            .transform_matrix();

        for parent_id in self.hierarchy_storage.parents(object_id) {
            matrix *= self
                .object_storage
                .get(*parent_id)
                .unwrap()
                .transform_matrix();
        }

        Some(matrix)
    }

    pub fn transform_matrix(&self, object_id: ObjectId) -> Option<&Mat4> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        Some(self.hierarchy_storage.matrix(object_id))
    }

    pub fn parent(&self, object_id: ObjectId) -> Option<ObjectId> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        self.hierarchy_storage.parent(object_id)
    }

    pub fn parents(&self, object_id: ObjectId) -> Option<&[ObjectId]> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        Some(self.hierarchy_storage.parents(object_id))
    }

    pub fn children(&self, object_id: ObjectId) -> Option<&[ObjectId]> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        Some(self.hierarchy_storage.children(object_id))
    }

    pub fn object_and_children(&self, object_id: ObjectId) -> Option<&[ObjectId]> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        Some(self.hierarchy_storage.object_and_children(object_id))
    }

    pub fn sibling_iter(&self, object_id: ObjectId) -> Option<ObjectSiblingIter> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        Some(self.hierarchy_storage.sibling_iter(object_id))
    }

    pub fn direct_children_iter(&self, object_id: ObjectId) -> Option<ObjectSiblingIter> {
        if !self.object_storage.is_exists(object_id) {
            return None;
        }

        self.hierarchy_storage.direct_children_iter(object_id)
    }

    pub fn create_object(&mut self) -> ObjectId {
        let object_id = self.object_id_allocator.allocate();
        let object = Object::new(object_id);
        self.object_storage.add(object);
        self.hierarchy_storage.add(object_id);

        object_id
    }

    pub fn create_object_with_components(&mut self, components: Vec<AnyComponent>) -> ObjectId {
        let object_id = self.object_id_allocator.allocate();
        let object = Object::with_components(object_id, components);
        self.object_storage.add(object);
        self.hierarchy_storage.add(object_id);

        object_id
    }

    pub fn remove_object(&mut self, object_id: ObjectId) {
        self.action_queue
            .push(SceneActionItem::RemoveObject { object_id });
    }

    pub fn set_active(&mut self, object_id: ObjectId, is_active: bool) {
        if !self.object_storage.is_exists(object_id) {
            return;
        }

        let hierarchy_object_ids = Vec::from(self.hierarchy_storage.object_and_children(object_id));
        let hierarchy_object_is_active_before = hierarchy_object_ids
            .iter()
            .map(|object_id| self.hierarchy_storage.is_active(*object_id))
            .collect::<Vec<_>>();

        self.hierarchy_storage.set_active(object_id, is_active);

        let hierarchy_object_is_active_after = hierarchy_object_ids
            .iter()
            .map(|object_id| self.hierarchy_storage.is_active(*object_id))
            .collect::<Vec<_>>();

        for index in (0..hierarchy_object_ids.len()).rev() {
            match (
                hierarchy_object_is_active_before[index],
                hierarchy_object_is_active_after[index],
            ) {
                (false, true) => {
                    self.action_queue.push(SceneActionItem::TriggerOnActive {
                        object_id: hierarchy_object_ids[index],
                    });
                }
                (true, false) => {
                    self.action_queue.push(SceneActionItem::TriggerOnInactive {
                        object_id: hierarchy_object_ids[index],
                    });
                }
                _ => {}
            }
        }
    }

    pub fn intern_name(&mut self, name: &str) -> string_interner::DefaultSymbol {
        self.hierarchy_storage.intern_name(name)
    }

    pub fn set_name(&mut self, object_id: ObjectId, name: &str) {
        self.hierarchy_storage.set_name(object_id, name);
    }

    pub fn set_transform(&mut self, object_id: ObjectId, transform: Transform) {
        if let Some(object) = self.object_storage.get_mut(object_id) {
            object.set_transform(transform);
            self.hierarchy_storage.set_dirty(object_id);
        }
    }

    pub fn set_parent(&mut self, object_id: ObjectId, mut parent_id: Option<ObjectId>) {
        if !self.object_storage.is_exists(object_id) {
            return;
        }

        if let Some(id) = parent_id {
            if !self.object_storage.is_exists(id) {
                parent_id = None;
            }
        }

        self.hierarchy_storage.set_parent(object_id, parent_id);
    }

    pub fn add_component<T>(&mut self, object_id: ObjectId, component: T) -> Option<ComponentId>
    where
        T: Component,
    {
        match self.object_storage.get_mut(object_id) {
            Some(object) => {
                let component_id = self.component_id_allocator.allocate();
                let component = AnyComponent::new(component_id, component);
                object.add_component(component);

                self.object_storage
                    .register_component(object_id, TypeId::of::<T>());

                Some(component_id)
            }
            None => None,
        }
    }

    pub fn remove_component(&mut self, object_id: ObjectId, component_id: ComponentId) {
        if let Some(object) = self.object_storage.get_mut(object_id) {
            if let Some(component) = object.remove_component(component_id) {
                self.object_storage
                    .unregister_component(object_id, component.type_id());
                self.component_id_allocator.deallocate(component_id);
            }
        }
    }

    pub fn attach_controller<T>(&mut self, object_id: ObjectId, controller: T)
    where
        T: Controller,
    {
        self.action_queue.push(SceneActionItem::AttachController {
            object_id,
            controller: Box::new(controller),
        });
    }

    pub fn detach_controller(&mut self, object_id: ObjectId) {
        self.action_queue
            .push(SceneActionItem::DetachController { object_id });
    }

    pub fn listen_on_update(&mut self, object_id: ObjectId) {
        self.action_queue
            .push(SceneActionItem::ListenOnUpdate { object_id });
    }

    pub fn unlisten_on_update(&mut self, object_id: ObjectId) {
        self.action_queue
            .push(SceneActionItem::UnlistenOnUpdate { object_id });
    }

    pub fn listen_on_late_update(&mut self, object_id: ObjectId) {
        self.action_queue
            .push(SceneActionItem::ListenOnLateUpdate { object_id });
    }

    pub fn unlisten_on_late_update(&mut self, object_id: ObjectId) {
        self.action_queue
            .push(SceneActionItem::UnlistenOnLateUpdate { object_id });
    }

    pub fn listen_event(&mut self, event: impl Into<String>, object_id: ObjectId) {
        self.action_queue.push(SceneActionItem::ListenEvent {
            event: event.into(),
            object_id,
        });
    }

    pub fn unlisten_event(&mut self, event: impl Into<String>, object_id: ObjectId) {
        self.action_queue.push(SceneActionItem::UnlistenEvent {
            event: event.into(),
            object_id,
        });
    }

    pub fn unlisten_event_all(&mut self, object_id: ObjectId) {
        self.action_queue
            .push(SceneActionItem::UnlistenEventAll { object_id });
    }

    pub fn emit_event(&mut self, event: impl Into<String>, param: impl Any) {
        self.action_queue.push(SceneActionItem::EmitEvent {
            event: event.into(),
            param: Box::new(param),
        });
    }
}
