use crate::cursor::Cursor;

pub trait Parse: Sized {
    type Error: ParseError;

    fn parse(cursor: &mut Cursor) -> Result<Self, Self::Error>;
}

pub trait ParseError {
    fn error_unexpected_eof() -> Self;
}
