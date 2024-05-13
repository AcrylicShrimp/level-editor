use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    vmd_primitives::VmdVec3,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmdCameraKeyFrameParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a VMD primitive: {0}")]
    VmdPrimitiveParseError(#[from] crate::vmd_primitives::VmdPrimitiveParseError),
}

impl ParseError for VmdCameraKeyFrameParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct VmdCameraKeyFrame {
    pub frame_index: u32,
    /// Distance from the camera to the target.
    pub distance: f32,
    pub target_position: VmdVec3,
    /// Euler angles of the camera, in yaw, pitch, and roll order.
    pub camera_rotation: VmdVec3,
    pub fov: f32,
    pub bezier: VmdCameraKeyFrameBezier,
    /// `true` if the camera is in perspective mode, orthographic mode otherwise.
    pub is_perspective: bool,
}

impl Parse for VmdCameraKeyFrame {
    type Error = VmdCameraKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // frame_index (4 bytes)
        // distance (4 bytes)
        // target_position (12 bytes)
        // camera_rotation (12 bytes)
        // fov (4 bytes)
        // bezier (24 bytes)
        // is_perspective (1 byte)
        let size = 4 + 4 + 12 + 12 + 4 + 24 + 1;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let frame_index = u32::parse(cursor)?;
        let distance = f32::parse(cursor)?;
        let target_position = VmdVec3::parse(cursor)?;
        let camera_rotation = VmdVec3::parse(cursor)?;
        let fov = f32::parse(cursor)?;
        let bezier = VmdCameraKeyFrameBezier::parse(cursor)?;
        let is_perspective = u8::parse(cursor)?;

        Ok(Self {
            frame_index,
            distance,
            target_position,
            camera_rotation,
            fov,
            bezier,
            is_perspective: is_perspective == 1,
        })
    }
}

impl Parse for Vec<VmdCameraKeyFrame> {
    type Error = VmdCameraKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // camera key frame count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let key_frame_count = u32::parse(cursor)?;
        let mut key_frames = Vec::with_capacity(key_frame_count as usize);

        for _ in 0..key_frame_count {
            key_frames.push(VmdCameraKeyFrame::parse(cursor)?);
        }

        Ok(key_frames)
    }
}

/// Four-point Bezier curves: `(0, 0)`, `(x1, y1)`, `(x2, y2)`, `(127, 127)`.
///
/// It laid out in the following order:
///
/// - `X_x1` `X_x2` `X_y1` `X_y2`
/// - `Y_x1` `Y_x2` `Y_y1` `Y_y2`
/// - `Z_x1` `Z_x2` `Z_y1` `Z_y2`
/// - `R_x1` `R_x2` `R_y1` `R_y2`
/// - `D_x1` `D_x2` `D_y1` `D_y2`
/// - `A_x1` `A_x2` `A_y1` `A_y2`
///
/// Note: `R` means `Rotation`, `D` means `Distance`, `A` means `Angle`.
#[derive(Debug, Clone)]
pub struct VmdCameraKeyFrameBezier {
    pub data: [u8; 24],
}

impl Parse for VmdCameraKeyFrameBezier {
    type Error = VmdCameraKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        cursor.ensure_bytes::<Self::Error>(24)?;

        let data = cursor.read::<Self::Error, 24>()?;
        let data = data.clone();

        Ok(Self { data })
    }
}
