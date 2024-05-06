use crate::scene::Component;
use lvl_math::Vec3;
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub kind: LightKind,
    pub light_color: Vec3,
}

impl Component for Light {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightKind {
    Point,
    Directional { direction: Vec3 },
}
