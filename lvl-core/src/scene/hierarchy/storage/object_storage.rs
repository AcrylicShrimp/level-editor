use crate::scene::{Component, Object, ObjectId};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

pub struct ObjectStorage {
    objects: HashMap<ObjectId, Object>,
    component_type_indices: HashMap<TypeId, HashSet<ObjectId>>,
}

impl ObjectStorage {
    pub(crate) fn new() -> Self {
        Self {
            objects: HashMap::new(),
            component_type_indices: HashMap::new(),
        }
    }

    pub fn is_exists(&self, object_id: ObjectId) -> bool {
        self.objects.contains_key(&object_id)
    }

    pub fn get(&self, object_id: ObjectId) -> Option<&Object> {
        self.objects.get(&object_id)
    }

    pub fn get_mut(&mut self, object_id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(&object_id)
    }

    pub fn object_ids_with_component<T>(&self) -> Option<&HashSet<ObjectId>>
    where
        T: Component,
    {
        self.component_type_indices.get(&TypeId::of::<T>())
    }

    pub(crate) fn add(&mut self, object: Object) {
        for component in object.components() {
            self.register_component(object.id(), component.type_id());
        }

        self.objects.entry(object.id()).or_insert(object);
    }

    pub(crate) fn remove(&mut self, object_id: ObjectId) -> bool {
        match self.objects.remove(&object_id) {
            Some(object) => {
                for component in object.components() {
                    self.unregister_component(object_id, component.type_id());
                }

                true
            }
            None => false,
        }
    }

    pub(crate) fn register_component(&mut self, object_id: ObjectId, type_id: TypeId) {
        self.component_type_indices
            .entry(type_id)
            .or_insert(HashSet::new())
            .insert(object_id);
    }

    pub(crate) fn unregister_component(&mut self, object_id: ObjectId, type_id: TypeId) {
        if let Some(component_type_index) = self.component_type_indices.get_mut(&type_id) {
            component_type_index.remove(&object_id);
        }
    }
}
