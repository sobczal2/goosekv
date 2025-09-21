use bytes::{
    Buf,
    BytesMut,
};
use thiserror::Error;

use crate::frame::{
    ARRAY_FIRST_CHAR,
    BULK_STRING_FIRST_CHAR,
    Frame,
    INTEGER_FIRST_CHAR,
    NULL_FIRST_CHAR,
    SIMPLE_ERROR_FIRST_CHAR,
    SIMPLE_STRING_FIRST_CHAR,
    TERMINATOR_STR,
};

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("invalid UTF-8")]
    InvalidUtf8,
    #[error("invalid first byte")]
    InvalidFirstByte,
    #[error("invalid integer")]
    InvalidInteger,
    #[error("invalid array")]
    InvalidArray,
    #[error("invalid null")]
    InvalidNull,
}

pub type ParsingResult<T> = Result<T, ParsingError>;

pub struct Parser {
    buf: BytesMut,
}

impl Parser {
    pub fn new() -> Self {
        Self { buf: BytesMut::with_capacity(4096) }
    }

    pub fn with_buf_size(size: usize) -> Self {
        Self { buf: BytesMut::with_capacity(size) }
    }

    pub fn reset(&mut self) {
        self.buf.clear();
    }

    pub fn parse(&mut self) -> ParsingResult<Option<Frame>> {
        if self.buf.is_empty() {
            return Ok(None);
        }

        match self.buf[0] as char {
            SIMPLE_STRING_FIRST_CHAR => self.parse_simple_string(),
            SIMPLE_ERROR_FIRST_CHAR => self.parse_simple_error(),
            INTEGER_FIRST_CHAR => self.parse_integer(),
            BULK_STRING_FIRST_CHAR => self.parse_bulk_string(),
            ARRAY_FIRST_CHAR => self.parse_array(),
            NULL_FIRST_CHAR => self.parse_null(),
            _ => Err(ParsingError::InvalidFirstByte),
        }
    }

    pub fn buf_mut(&mut self) -> &mut BytesMut {
        &mut self.buf
    }

    fn parse_simple_string(&mut self) -> ParsingResult<Option<Frame>> {
        if let Some(end_index) = self.buf.windows(2).position(|w| w == TERMINATOR_STR.as_bytes()) {
            let value = &self.buf[1..end_index];
            let value = str::from_utf8(value).map_err(|_| ParsingError::InvalidUtf8)?;

            let frame = Frame::SimpleString(value.to_string());
            self.buf.advance(end_index + TERMINATOR_STR.len());

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn parse_simple_error(&mut self) -> ParsingResult<Option<Frame>> {
        if let Some(end_index) = self.buf.windows(2).position(|w| w == TERMINATOR_STR.as_bytes()) {
            let value = &self.buf[1..end_index];
            let value = str::from_utf8(value).map_err(|_| ParsingError::InvalidUtf8)?;

            let frame = Frame::SimpleError(value.to_string());
            self.buf.advance(end_index + TERMINATOR_STR.len());

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn parse_integer(&mut self) -> ParsingResult<Option<Frame>> {
        if let Some(end_index) = self.buf.windows(2).position(|w| w == TERMINATOR_STR.as_bytes()) {
            let value = &self.buf[1..end_index];
            let value = str::from_utf8(value).map_err(|_| ParsingError::InvalidUtf8)?;
            let value = value.parse().map_err(|_| ParsingError::InvalidInteger)?;

            let frame = Frame::Integer(value);
            self.buf.advance(end_index + TERMINATOR_STR.len());

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn parse_bulk_string(&mut self) -> ParsingResult<Option<Frame>> {
        if let Some(end_index) = self.buf.windows(2).position(|w| w == TERMINATOR_STR.as_bytes()) {
            let length = &self.buf[1..end_index];
            let length = str::from_utf8(length).map_err(|_| ParsingError::InvalidUtf8)?;
            let length = length.parse::<usize>().map_err(|_| ParsingError::InvalidInteger)?;

            let value = &self.buf
                [end_index + TERMINATOR_STR.len()..end_index + TERMINATOR_STR.len() + length];
            let value = str::from_utf8(value).map_err(|_| ParsingError::InvalidUtf8)?;

            let frame = Frame::BulkString(value.to_string());
            self.buf.advance(end_index + TERMINATOR_STR.len() + length + TERMINATOR_STR.len());

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn parse_array(&mut self) -> ParsingResult<Option<Frame>> {
        if let Some(end_index) = self.buf.windows(2).position(|w| w == TERMINATOR_STR.as_bytes()) {
            let value = &self.buf[1..end_index];
            let value = str::from_utf8(value).map_err(|_| ParsingError::InvalidUtf8)?;
            let value = value.parse::<usize>().map_err(|_| ParsingError::InvalidInteger)?;

            self.buf.advance(end_index + TERMINATOR_STR.len());

            let values = (0..value)
                .flat_map(|_| self.parse().map(|v| v.ok_or(ParsingError::InvalidArray)))
                .collect::<ParsingResult<Box<[_]>>>()?;
            let frame = Frame::Array(values);

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn parse_null(&mut self) -> ParsingResult<Option<Frame>> {
        if self.buf.len() < 3 {
            return Ok(None);
        }

        if &self.buf[1..3] != TERMINATOR_STR.as_bytes() {
            return Err(ParsingError::InvalidNull);
        }

        Ok(Some(Frame::Null))
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use bytes::BufMut;

    use super::*;

    macro_rules! test_parse {
        ($name:ident, $bytes:expr, $frame:expr) => {
            #[test]
            fn $name() {
                let mut parser = Parser::new();
                parser.buf_mut().writer().write_all($bytes).unwrap();

                assert_eq!(parser.parse().unwrap(), Some($frame));
            }
        };
    }

    test_parse!(simple_string, b"+OK\r\n", Frame::SimpleString("OK".to_string()));
    test_parse!(
        simple_error,
        b"-Error message\r\n",
        Frame::SimpleError("Error message".to_string())
    );
    test_parse!(integer, b":10\r\n", Frame::Integer(10));
    test_parse!(bulk_string, b"$5\r\nhello\r\n", Frame::BulkString("hello".to_string()));
    test_parse!(empty_bulk_string, b"$0\r\n\r\n", Frame::BulkString("".to_string()));
    test_parse!(
        array,
        b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n",
        Frame::Array(
            vec![Frame::BulkString("hello".to_string()), Frame::BulkString("world".to_string())]
                .into_boxed_slice()
        )
    );
    test_parse!(empty_array, b"*0\r\n", Frame::Array([].into()));
    test_parse!(null, b"_\r\n", Frame::Null);
}
