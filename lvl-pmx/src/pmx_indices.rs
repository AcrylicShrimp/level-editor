use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::{PmxConfig, PmxIndexSize},
    pmx_primitives::PmxVertexIndex,
};
use std::mem::size_of;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxIndicesParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
}

impl ParseError for PmxIndicesParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxIndices {
    /// vertex indices in CW order (DirectX style)
    pub vertex_indices: Vec<PmxVertexIndex>,
}

impl Parse for PmxIndices {
    type Error = PmxIndicesParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // indices count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;

        // index data (count * vertex_index_size bytes)
        let size = count * config.vertex_index_size.size();
        cursor.ensure_bytes::<Self::Error>(size)?;

        let mut indices = Vec::with_capacity(count);

        match config.vertex_index_size {
            PmxIndexSize::U8 => {
                let bytes = cursor.read_dynamic::<Self::Error>(count * size_of::<u8>())?;

                for index in 0..count {
                    let vertex_index = PmxVertexIndex::new(u8::from_le_bytes(
                        bytes[index * 1..(index + 1) * 1].try_into().unwrap(),
                    ) as u32);
                    indices.push(vertex_index);
                }
            }
            PmxIndexSize::U16 => {
                let bytes = cursor.read_dynamic::<Self::Error>(count * size_of::<u16>())?;

                for index in 0..count {
                    let vertex_index = PmxVertexIndex::new(u16::from_le_bytes(
                        bytes[index * 2..(index + 1) * 2].try_into().unwrap(),
                    ) as u32);
                    indices.push(vertex_index);
                }
            }
            PmxIndexSize::U32 => {
                let bytes = cursor.read_dynamic::<Self::Error>(count * size_of::<u32>())?;

                for index in 0..count {
                    let vertex_index = PmxVertexIndex::new(u32::from_le_bytes(
                        bytes[index * 4..(index + 1) * 4].try_into().unwrap(),
                    ));
                    indices.push(vertex_index);
                }
            }
        }

        Ok(Self {
            vertex_indices: indices,
        })
    }
}
