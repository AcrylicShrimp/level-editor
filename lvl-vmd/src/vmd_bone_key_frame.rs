use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    primitives::ShiftJISString,
    vmd_primitives::{VmdQuat, VmdVec3},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmdBoneKeyFrameParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a VMD primitive: {0}")]
    VmdPrimitiveParseError(#[from] crate::vmd_primitives::VmdPrimitiveParseError),
}

impl ParseError for VmdBoneKeyFrameParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct VmdBoneKeyFrame {
    pub bone_name: String,
    pub frame_index: u32,
    pub translation: VmdVec3,
    pub rotation: VmdQuat,
    pub bezier: VmdBoneKeyFrameBezier,
}

impl Parse for VmdBoneKeyFrame {
    type Error = VmdBoneKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // bone_name (15 bytes)
        // frame_index (4 bytes)
        // translation (12 bytes)
        // rotation (16 bytes)
        // bezier (64 bytes)
        let size = 15 + 4 + 12 + 16 + 64;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let bone_name = ShiftJISString::parse(cursor, 15)?;
        let frame_index = u32::parse(cursor)?;
        let translation = VmdVec3::parse(cursor)?;
        let rotation = VmdQuat::parse(cursor)?;
        let bezier = VmdBoneKeyFrameBezier::parse(cursor)?;

        Ok(Self {
            bone_name: bone_name.0,
            frame_index,
            translation,
            rotation,
            bezier,
        })
    }
}

impl Parse for Vec<VmdBoneKeyFrame> {
    type Error = VmdBoneKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // bone key frame count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let key_frame_count = u32::parse(cursor)?;
        let mut key_frames = Vec::with_capacity(key_frame_count as usize);

        for _ in 0..key_frame_count {
            key_frames.push(VmdBoneKeyFrame::parse(cursor)?);
        }

        Ok(key_frames)
    }
}

/// Four-point Bezier curves: `(0, 0)`, `(x1, y1)`, `(x2, y2)`, `(127, 127)`.
///
/// It represents the parameter for each axis:
/// - X-axis interpolation parameters `(X_x1, X_y1)`, `(X_x2, X_y2)`.
/// - Y-axis interpolation parameters `(Y_x1, Y_y1)`, `(Y_x2, Y_y2)`.
/// - Z-axis interpolation parameters `(Z_x1, Z_y1)`, `(Z_x2, Z_y2)`.
/// - Rotation interpolation parameters `(R_x1, R_y1)`, `(R_x2, R_y2)`.
///
/// It contains 4 copy of them, laid out in the following order:
///
/// `[0]`:
/// - `X_x1` `Y_x1` `Z_x1` `R_x1`
/// - `X_y1` `Y_y1` `Z_y1` `R_y1`
/// - `X_x2` `Y_x2` `Z_x2` `R_x2`
/// - `X_y2` `Y_y2` `Z_y2` `R_y2`
///
/// `[1]`:
/// - `Y_x1` `Z_x1` `R_x1` `X_y1`
/// - `Y_y1` `Z_y1` `R_y1` `X_x2`
/// - `Y_x2` `Z_x2` `R_x2` `X_y2`
/// - `Y_y2` `Z_y2` `R_y2` `__01`
///
/// `[2]`:
/// - `Z_x1` `R_x1` `X_y1` `Y_y1`
/// - `Z_y1` `R_y1` `X_x2` `Y_x2`
/// - `Z_x2` `R_x2` `X_y2` `Y_y2`
/// - `Z_y2` `R_y2` `__01` `__00`
///
/// `[3]`:
/// - `R_x1` `X_y1` `Y_y1` `Z_y1`
/// - `R_y1` `X_x2` `Y_x2` `Z_x2`
/// - `R_x2` `X_y2` `Y_y2` `Z_y2`
/// - `R_y2` `__01` `__00` `__00`
///
/// Note 1: `__01` and `__00` are constant values.
///
/// Note 2: **Not all parameters are used**. Original MMD uses only one of them. The exact locations are follow:
///
/// - X-axis: `0`, `8`, `4`, `12`
/// - Y-axis: `16`, `24`, `20`, `28`
/// - Z-axis: `32`, `40`, `36`, `44`
/// - Rotation: `48`, `56`, `52`, `60`
///
/// Rest of the parameters are unused.
#[derive(Debug, Clone)]
pub struct VmdBoneKeyFrameBezier {
    pub data: [u8; 64],
}

impl Parse for VmdBoneKeyFrameBezier {
    type Error = VmdBoneKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        cursor.ensure_bytes::<Self::Error>(64)?;

        let data = cursor.read::<Self::Error, 64>()?;
        let data = data.clone();

        Ok(Self { data })
    }
}
