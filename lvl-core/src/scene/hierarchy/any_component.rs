use super::{Component, ComponentId};
use std::any::TypeId;

pub struct AnyComponent {
    id: ComponentId,
    inner: Box<dyn Component>,
}

impl AnyComponent {
    pub fn new<T>(id: ComponentId, inner: T) -> Self
    where
        T: Component,
    {
        Self {
            id,
            inner: Box::new(inner),
        }
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.inner.name()
    }

    pub fn type_id(&self) -> TypeId {
        self.inner.as_any().type_id()
    }

    pub fn is_type_of<T>(&self) -> bool
    where
        T: Component,
    {
        self.inner.as_any().type_id() == TypeId::of::<T>()
    }

    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Component,
    {
        self.inner.as_any().downcast_ref::<T>()
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Component,
    {
        self.inner.as_any_mut().downcast_mut::<T>()
    }
}
