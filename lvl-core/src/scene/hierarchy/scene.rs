use super::{
    ComponentIdAllocator, ControllerStorage, EventReceiverStorage, HierarchyStorage,
    ObjectIdAllocator, ObjectStorage, ReadOnlySceneProxy, SceneActionItem, SceneActionResult,
    SceneProxy,
};
use crate::context::Context;
use winit::window::Window;

pub struct Scene<'ctx, 'window: 'ctx> {
    context: &'ctx Context<'window>,
    window: &'window Window,
    object_id_allocator: ObjectIdAllocator,
    component_id_allocator: ComponentIdAllocator,
    object_storage: ObjectStorage,
    hierarchy_storage: HierarchyStorage,
    controller_storage: ControllerStorage,
    event_receiver_storage: EventReceiverStorage,
}

impl<'ctx, 'window: 'ctx> Scene<'ctx, 'window> {
    pub(crate) fn new(context: &'ctx Context<'window>, window: &'window Window) -> Self {
        Self {
            context,
            window,
            object_id_allocator: ObjectIdAllocator::new(),
            component_id_allocator: ComponentIdAllocator::new(),
            object_storage: ObjectStorage::new(),
            hierarchy_storage: HierarchyStorage::new(),
            controller_storage: ControllerStorage::new(),
            event_receiver_storage: EventReceiverStorage::new(),
        }
    }

    pub fn read_only_proxy(&mut self) -> ReadOnlySceneProxy {
        ReadOnlySceneProxy::new(SceneProxy::new(
            self.context,
            self.window,
            &mut self.object_id_allocator,
            &mut self.component_id_allocator,
            &mut self.object_storage,
            &mut self.hierarchy_storage,
        ))
    }

    pub fn with_proxy<R>(&mut self, f: impl FnOnce(&mut SceneProxy) -> R) -> R {
        let mut scene = SceneProxy::new(
            self.context,
            self.window,
            &mut self.object_id_allocator,
            &mut self.component_id_allocator,
            &mut self.object_storage,
            &mut self.hierarchy_storage,
        );
        let result = f(&mut scene);
        let ctx_result = scene.into_result();
        self.handle_context_result(ctx_result);
        result
    }

    pub(crate) fn trigger_update(&mut self) {
        let mut scene = SceneProxy::new(
            self.context,
            self.window,
            &mut self.object_id_allocator,
            &mut self.component_id_allocator,
            &mut self.object_storage,
            &mut self.hierarchy_storage,
        );
        self.controller_storage.invoke_on_update(&mut scene);

        let result = scene.into_result();
        self.handle_context_result(result);
    }

    pub(crate) fn trigger_late_update(&mut self) {
        let mut scene = SceneProxy::new(
            self.context,
            self.window,
            &mut self.object_id_allocator,
            &mut self.component_id_allocator,
            &mut self.object_storage,
            &mut self.hierarchy_storage,
        );
        self.controller_storage.invoke_on_late_update(&mut scene);

        let result = scene.into_result();
        self.handle_context_result(result);
    }

    fn handle_context_result(&mut self, mut result: SceneActionResult) {
        while !result.action_queue.is_empty() {
            let mut scene = SceneProxy::new(
                self.context,
                self.window,
                &mut self.object_id_allocator,
                &mut self.component_id_allocator,
                &mut self.object_storage,
                &mut self.hierarchy_storage,
            );

            for action in result.action_queue {
                match action {
                    SceneActionItem::RemoveObject { object_id } => {
                        if !scene.object_storage().is_exists(object_id) {
                            continue;
                        }

                        let removed_hierarchy_object_ids =
                            Vec::from(scene.hierarchy_storage().object_and_children(object_id));

                        for &removed_object_id in removed_hierarchy_object_ids.iter().rev() {
                            self.controller_storage
                                .detach_controller(removed_object_id, &mut scene);
                        }

                        for &removed_object_id in removed_hierarchy_object_ids.iter().rev() {
                            self.event_receiver_storage.unlisten_all(removed_object_id);
                            scene.object_storage_mut().remove(removed_object_id);
                            scene.object_id_allocator_mut().deallocate(object_id);
                        }

                        scene.hierarchy_storage_mut().remove(object_id);
                    }
                    SceneActionItem::TriggerOnActive { object_id } => {
                        self.controller_storage
                            .invoke_on_active(object_id, &mut scene);
                    }
                    SceneActionItem::TriggerOnInactive { object_id } => {
                        self.controller_storage
                            .invoke_on_inactive(object_id, &mut scene);
                    }
                    SceneActionItem::AttachController {
                        object_id,
                        controller,
                    } => {
                        self.controller_storage
                            .attach_controller(object_id, controller, &mut scene);
                    }
                    SceneActionItem::DetachController { object_id } => {
                        self.event_receiver_storage.unlisten_all(object_id);
                        self.controller_storage
                            .detach_controller(object_id, &mut scene);
                    }
                    SceneActionItem::ListenOnUpdate { object_id } => {
                        self.controller_storage.listen_on_update(object_id);
                    }
                    SceneActionItem::UnlistenOnUpdate { object_id } => {
                        self.controller_storage.unlisten_on_update(object_id);
                    }
                    SceneActionItem::ListenOnLateUpdate { object_id } => {
                        self.controller_storage.listen_on_late_update(object_id);
                    }
                    SceneActionItem::UnlistenOnLateUpdate { object_id } => {
                        self.controller_storage.unlisten_on_late_update(object_id);
                    }
                    SceneActionItem::ListenEvent { event, object_id } => {
                        self.event_receiver_storage.listen(event, object_id);
                    }
                    SceneActionItem::UnlistenEvent { event, object_id } => {
                        self.event_receiver_storage.unlisten(event, object_id);
                    }
                    SceneActionItem::UnlistenEventAll { object_id } => {
                        self.event_receiver_storage.unlisten_all(object_id);
                    }
                    SceneActionItem::EmitEvent { event, param } => {
                        self.event_receiver_storage.emit(
                            &event,
                            &param,
                            &mut scene,
                            &mut self.controller_storage,
                        );
                    }
                }
            }

            result = scene.into_result();
        }
    }

    pub(crate) fn prepare_render(&mut self) {
        self.hierarchy_storage.copy_dirty_to_current_frame();
        self.hierarchy_storage.update_object_matrices(|object_id| {
            self.object_storage
                .get(object_id)
                .map(|object| object.transform_matrix())
        });
    }
}
