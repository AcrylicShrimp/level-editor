use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmdPrimitiveParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
}

impl ParseError for VmdPrimitiveParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VmdVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Parse for VmdVec3 {
    type Error = VmdPrimitiveParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let bytes = cursor.read::<Self::Error, 12>()?;
        let x = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(bytes[8..12].try_into().unwrap());

        Ok(Self { x, y, z })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VmdQuat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Parse for VmdQuat {
    type Error = VmdPrimitiveParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let bytes = cursor.read::<Self::Error, 16>()?;
        let x = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let w = f32::from_le_bytes(bytes[12..16].try_into().unwrap());

        Ok(Self { x, y, z, w })
    }
}
