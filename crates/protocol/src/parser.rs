use bytes::{
    Buf, Bytes, BytesMut
};
use thiserror::Error;

use crate::frame::{
    Frame, ARRAY_FIRST_BYTE, BULK_STRING_FIRST_BYTE, INTEGER_FIRST_BYTE, NULL_FIRST_BYTE, SIMPLE_ERROR_FIRST_BYTE, SIMPLE_STRING_FIRST_BYTE, TERMINATOR
};

#[derive(Debug, Error)]
pub enum ParseError {
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

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    buf: BytesMut,
}

struct ParsedFrame {
    frame: Frame,
    advance_by: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self { buf: BytesMut::new() }
    }

    pub fn parse(&mut self) -> ParseResult<Option<Frame>> {
        let result = parse_buf(&self.buf)?;
        match result {
            Some(parsed_frame) => {
                self.buf.advance(parsed_frame.advance_by);
                Ok(Some(parsed_frame.frame))
            },
            None => {
                Ok(None)
            },
        }
    }

    pub fn buf(&self) -> &BytesMut {
        &self.buf
    }

    pub fn buf_mut(&mut self) -> &mut BytesMut {
        &mut self.buf
    }
}

fn parse_buf(buf: &[u8]) -> ParseResult<Option<ParsedFrame>> {
    if buf.is_empty() {
        return Ok(None);
    }

    match buf[0] {
        SIMPLE_STRING_FIRST_BYTE => parse_simple_string(buf),
        SIMPLE_ERROR_FIRST_BYTE => parse_simple_error(buf),
        INTEGER_FIRST_BYTE => parse_integer(buf),
        BULK_STRING_FIRST_BYTE => parse_bulk_string(buf),
        ARRAY_FIRST_BYTE => parse_array(buf),
        NULL_FIRST_BYTE => parse_null(buf),
        _ => Err(ParseError::InvalidFirstByte),
    }
}

fn parse_simple_string(buf: &[u8]) -> ParseResult<Option<ParsedFrame>> {
    if let Some(end_index) = buf.windows(2).position(|w| w == TERMINATOR) {
        let value = &buf[1..end_index];
        let frame = Frame::SimpleString(Bytes::copy_from_slice(value));

        let parsed_frame = ParsedFrame {
            frame,
            advance_by: end_index + TERMINATOR.len(),
        };

        Ok(Some(parsed_frame))
    } else {
        Ok(None)
    }
}

fn parse_simple_error(buf: &[u8]) -> ParseResult<Option<ParsedFrame>> {
    if let Some(end_index) = buf.windows(2).position(|w| w == TERMINATOR) {
        let value = &buf[1..end_index];
        let frame = Frame::SimpleError(Bytes::copy_from_slice(value));

        let parsed_frame = ParsedFrame {
            frame,
            advance_by: end_index + TERMINATOR.len(),
        };

        Ok(Some(parsed_frame))
    } else {
        Ok(None)
    }
}

fn parse_integer(buf: &[u8]) -> ParseResult<Option<ParsedFrame>> {
    if let Some(end_index) = buf.windows(2).position(|w| w == TERMINATOR) {
        let value = &buf[1..end_index];
        let value = str::from_utf8(value).map_err(|_| ParseError::InvalidUtf8)?;
        let value = value.parse().map_err(|_| ParseError::InvalidInteger)?;

        let frame = Frame::Integer(value);

        let parsed_frame = ParsedFrame {
            frame,
            advance_by: end_index + TERMINATOR.len(),
        };

        Ok(Some(parsed_frame))
    } else {
        Ok(None)
    }
}

fn parse_bulk_string(buf: &[u8]) -> ParseResult<Option<ParsedFrame>> {
    if let Some(end_index) = buf.windows(2).position(|w| w == TERMINATOR) {
        let length = &buf[1..end_index];
        let length = str::from_utf8(length).map_err(|_| ParseError::InvalidUtf8)?;
        let length = length.parse::<usize>().map_err(|_| ParseError::InvalidInteger)?;

        let value = &buf[end_index + TERMINATOR.len()..end_index + TERMINATOR.len() + length];

        let frame = Frame::BulkString(Bytes::copy_from_slice(value));

        let parsed_frame = ParsedFrame {
            frame,
            advance_by: end_index + TERMINATOR.len() + length + TERMINATOR.len(),
        };

        Ok(Some(parsed_frame))
    } else {
        Ok(None)
    }
}

fn parse_array(buf: &[u8]) -> ParseResult<Option<ParsedFrame>> {
    if let Some(end_index) = buf.windows(2).position(|w| w == TERMINATOR) {
        let count = &buf[1..end_index];
        let count = str::from_utf8(count).map_err(|_| ParseError::InvalidUtf8)?;
        let count = count.parse::<usize>().map_err(|_| ParseError::InvalidInteger)?;

        let mut advance_by = end_index + TERMINATOR.len();

        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            match parse_buf(&buf[advance_by..])? {
                Some(parsed_frame) => {
                    advance_by += parsed_frame.advance_by;
                    frames.push(parsed_frame.frame);
                },
                None => return Ok(None),
            }
        }

        let frame = Frame::Array(frames.into_boxed_slice());

        let parsed_frame = ParsedFrame {
            frame,
            advance_by,
        };

        Ok(Some(parsed_frame))
    } else {
        Ok(None)
    }
}

fn parse_null(buf: &[u8]) -> ParseResult<Option<ParsedFrame>> {
    if buf.len() < 3 {
        return Ok(None);
    }

    if &buf[1..3] != TERMINATOR {
        return Err(ParseError::InvalidNull);
    }

    let parsed_frame = ParsedFrame {
        frame: Frame::Null,
        advance_by: size_of_val(&NULL_FIRST_BYTE) + TERMINATOR.len(),
    };

    Ok(Some(parsed_frame))
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

    test_parse!(simple_string, b"+OK\r\n", Frame::SimpleString(Bytes::copy_from_slice(b"OK")));
    test_parse!(
        simple_error,
        b"-Error message\r\n",
        Frame::SimpleError(Bytes::copy_from_slice(b"Error message"))
    );
    test_parse!(integer, b":10\r\n", Frame::Integer(10));
    test_parse!(bulk_string, b"$5\r\nhello\r\n", Frame::BulkString(Bytes::copy_from_slice(b"hello")));
    test_parse!(empty_bulk_string, b"$0\r\n\r\n", Frame::BulkString(Bytes::new()));
    test_parse!(
        array,
        b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n",
        Frame::Array(
            vec![Frame::BulkString(Bytes::copy_from_slice(b"hello")), Frame::BulkString(Bytes::copy_from_slice(b"world"))]
                .into_boxed_slice()
        )
    );
    test_parse!(empty_array, b"*0\r\n", Frame::Array([].into()));
    test_parse!(null, b"_\r\n", Frame::Null);
}
