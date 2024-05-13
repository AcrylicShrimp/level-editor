use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    primitives::ShiftJISString,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmdMorphKeyFrameParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a VMD primitive: {0}")]
    VmdPrimitiveParseError(#[from] crate::vmd_primitives::VmdPrimitiveParseError),
}

impl ParseError for VmdMorphKeyFrameParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct VmdMorphKeyFrame {
    pub morph_name: String,
    pub frame_index: u32,
    pub weight: f32,
}

impl Parse for VmdMorphKeyFrame {
    type Error = VmdMorphKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // morph_name (15 bytes)
        // frame_index (4 bytes)
        // weight (4 bytes)
        let size = 15 + 4 + 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let morph_name = ShiftJISString::parse(cursor, 15)?;
        let frame_index = u32::parse(cursor)?;
        let weight = f32::parse(cursor)?;

        Ok(Self {
            morph_name: morph_name.0,
            frame_index,
            weight,
        })
    }
}

impl Parse for Vec<VmdMorphKeyFrame> {
    type Error = VmdMorphKeyFrameParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // morph key frame count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let key_frame_count = u32::parse(cursor)?;
        let mut key_frames = Vec::with_capacity(key_frame_count as usize);

        for _ in 0..key_frame_count {
            key_frames.push(VmdMorphKeyFrame::parse(cursor)?);
        }

        Ok(key_frames)
    }
}
