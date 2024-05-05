use crate::scene::Component;
use lvl_math::{Mat4, Vec4};
use std::any::Any;

pub struct Camera {
    pub order: i64,
    pub clear_mode: CameraClearMode,
    pub projection_mode: CameraProjectionMode,
}

impl Component for Camera {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub enum CameraClearMode {
    All { color: Vec4 },
    DepthStencilOnly,
    Keep,
}

#[derive(Debug, Clone)]
pub enum CameraProjectionMode {
    Perspective {
        fov: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

impl CameraProjectionMode {
    pub fn to_mat4(&self, aspect: f32, transform_matrix: &Mat4) -> Mat4 {
        let projection_matrix = match self {
            &CameraProjectionMode::Perspective { fov, near, far } => {
                Mat4::perspective(fov, aspect, near, far)
            }
            &CameraProjectionMode::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Mat4::orthographic(left, right, bottom, top, near, far),
        };
        transform_matrix * projection_matrix
    }
}
