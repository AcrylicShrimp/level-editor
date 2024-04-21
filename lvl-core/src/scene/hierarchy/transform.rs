use lvl_math::{Mat4, Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_mat4(matrix: &Mat4) -> Self {
        let (position, rotation, scale) = matrix.split();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Returns the transform matrix that transforms from local space to world space.
    /// This matrix does not include the parent transforms.
    pub fn matrix(&self) -> Mat4 {
        Mat4::srt(self.position, self.rotation, self.scale)
    }

    /// Returns the inverse transform matrix that transforms from world space to local space.
    /// This matrix does not include the parent transforms.
    pub fn inverse_matrix(&self) -> Mat4 {
        Mat4::trs(-self.position, -self.rotation, Vec3::recip(self.scale))
    }
}
