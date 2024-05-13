use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustPrimitiveParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("invalid Shift JIS string found")]
    InvalidShiftJISString,
}

impl ParseError for RustPrimitiveParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

impl Parse for u8 {
    type Error = RustPrimitiveParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(u8::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 1>()?,
        ))
    }
}

impl Parse for u32 {
    type Error = RustPrimitiveParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(u32::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 4>()?,
        ))
    }
}

impl Parse for f32 {
    type Error = RustPrimitiveParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(f32::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 4>()?,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct ShiftJISString(pub String);

impl ShiftJISString {
    pub fn parse(cursor: &mut Cursor, byte_len: usize) -> Result<Self, RustPrimitiveParseError> {
        cursor.ensure_bytes(byte_len)?;

        let bytes = cursor.read_dynamic::<RustPrimitiveParseError>(byte_len)?;
        let string = match encoding_rs::SHIFT_JIS.decode_without_bom_handling(bytes) {
            (_, true) => {
                return Err(RustPrimitiveParseError::InvalidShiftJISString);
            }
            (string, false) => match string.find('\0') {
                Some(index) => string[..index].to_owned(),
                None => string.to_string(),
            },
        };
        Ok(Self(string))
    }
}
