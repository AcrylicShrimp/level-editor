use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    vmd_primitives::VmdVec3,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmdLightKeyFrameParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a VMD primitive: {0}")]
    VmdPrimitiveParseError(#[from] crate::vmd_primitives::VmdPrimitiveParseError),
}

impl ParseError for VmdLightKeyFrameParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct VmdLightKeyFrame {
    pub frame_index: u32,
    pub color: VmdVec3,
    pub direction: VmdVec3,
}

impl Parse for VmdLightKeyFrame {
    type Error = VmdLightKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // frame_index (4 bytes)
        // color (12 bytes)
        // direction (12 bytes)
        let size = 4 + 12 + 12;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let frame_index = u32::parse(cursor)?;
        let color = VmdVec3::parse(cursor)?;
        let direction = VmdVec3::parse(cursor)?;

        Ok(Self {
            frame_index,
            color,
            direction,
        })
    }
}

impl Parse for Vec<VmdLightKeyFrame> {
    type Error = VmdLightKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // light key frame count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let key_frame_count = u32::parse(cursor)?;
        let mut key_frames = Vec::with_capacity(key_frame_count as usize);

        for _ in 0..key_frame_count {
            key_frames.push(VmdLightKeyFrame::parse(cursor)?);
        }

        Ok(key_frames)
    }
}
