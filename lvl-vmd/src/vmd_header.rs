use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    primitives::ShiftJISString,
};
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmdHeaderParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("`{signature:?}` is not a valid VMD signature")]
    InvalidSignature { signature: [u8; 30] },
}

impl ParseError for VmdHeaderParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct VmdHeader {
    pub version: VmdVersion,
    pub model_name: String,
}

impl Parse for VmdHeader {
    type Error = VmdHeaderParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error> {
        /// Size of VMD header
        /// - 30 bytes: signature (including version)
        const HEADER_SIZE: usize = 30;
        cursor.ensure_bytes::<VmdHeaderParseError>(HEADER_SIZE)?;

        let signature = *cursor.read::<VmdHeaderParseError, 30>()?;
        let version = if &signature[0..25] == b"Vocaloid Motion Data file" {
            VmdVersion::V1
        } else if &signature[0..25] == b"Vocaloid Motion Data 0002" {
            VmdVersion::V2
        } else {
            return Err(VmdHeaderParseError::InvalidSignature { signature });
        };

        let model_name = match version {
            VmdVersion::V1 => ShiftJISString::parse(cursor, 10)?,
            VmdVersion::V2 => ShiftJISString::parse(cursor, 20)?,
        };

        Ok(Self {
            version,
            model_name: model_name.0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VmdVersion {
    V1,
    V2,
}

impl Display for VmdVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
            Self::V2 => write!(f, "v2"),
        }
    }
}
