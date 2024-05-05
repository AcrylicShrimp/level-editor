use crate::scene::Component;
use std::any::Any;

#[derive(Debug, Clone, Copy)]
pub struct StaticMeshRendererGroup;

impl Component for StaticMeshRendererGroup {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
