use crate::scene::Component;
use lvl_math::Vec4;
use std::any::Any;

#[derive(Debug, Clone)]
pub enum CameraClearMode {
    All { color: Vec4 },
    DepthStencilOnly,
    Keep,
}

pub struct Camera {
    pub order: i64,
    pub clear_mode: CameraClearMode,
}

impl Component for Camera {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
