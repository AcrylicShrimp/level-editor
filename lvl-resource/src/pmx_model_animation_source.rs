use crate::{FromResourceKind, ResourceKind};
use lvl_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelAnimationSource {
    bone_key_frames: Vec<PmxModelAnimationBoneKeyFrame>,
    morph_key_frames: Vec<PmxModelAnimationMorphKeyFrame>,
}

impl PmxModelAnimationSource {
    pub fn new(
        bone_key_frames: Vec<PmxModelAnimationBoneKeyFrame>,
        morph_key_frames: Vec<PmxModelAnimationMorphKeyFrame>,
    ) -> Self {
        Self {
            bone_key_frames,
            morph_key_frames,
        }
    }

    pub fn bone_key_frames(&self) -> &[PmxModelAnimationBoneKeyFrame] {
        &self.bone_key_frames
    }

    pub fn morph_key_frames(&self) -> &[PmxModelAnimationMorphKeyFrame] {
        &self.morph_key_frames
    }
}

impl FromResourceKind for PmxModelAnimationSource {
    fn from(kind: &ResourceKind) -> Option<&Self> {
        match kind {
            ResourceKind::PmxModelAnimation(pmx_model_animation) => Some(pmx_model_animation),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelAnimationBoneKeyFrame {
    pub frame_index: u32,
    pub elements: Vec<PmxModelAnimationBoneKeyFrameElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelAnimationBoneKeyFrameElement {
    pub bone_name: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub bezier: PmxModelAnimationBoneBezier,
}

/// Four-point Bezier curves: `(0, 0)`, `(x1, y1)`, `(x2, y2)`, `(127, 127)`.
///
/// It represents the parameter for each axis:
/// - X-axis interpolation parameters `(X_x1, X_y1)`, `(X_x2, X_y2)`.
/// - Y-axis interpolation parameters `(Y_x1, Y_y1)`, `(Y_x2, Y_y2)`.
/// - Z-axis interpolation parameters `(Z_x1, Z_y1)`, `(Z_x2, Z_y2)`.
/// - Rotation interpolation parameters `(R_x1, R_y1)`, `(R_x2, R_y2)`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelAnimationBoneBezier {
    pub x_axis: [u8; 4],
    pub y_axis: [u8; 4],
    pub z_axis: [u8; 4],
    pub rotation: [u8; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelAnimationMorphKeyFrame {
    pub frame_index: u32,
    pub elements: Vec<PmxModelAnimationMorphKeyFrameElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PmxModelAnimationMorphKeyFrameElement {
    pub morph_name: String,
    pub weight: f32,
}
