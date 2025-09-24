use std::fmt::{
    self,
    Display,
    Formatter, Write,
};

use bytes::{BufMut, Bytes, BytesMut};
use thiserror::Error;

pub const TERMINATOR: &[u8; 2] = b"\r\n";
pub const SIMPLE_STRING_FIRST_BYTE: u8 = b'+';
pub const SIMPLE_ERROR_FIRST_BYTE: u8 = b'-';
pub const INTEGER_FIRST_BYTE: u8 = b':';
pub const BULK_STRING_FIRST_BYTE: u8 = b'$';
pub const ARRAY_FIRST_BYTE: u8 = b'*';
pub const NULL_FIRST_BYTE: u8 = b'_';

#[derive(Debug, PartialEq, Eq)]
pub enum GFrame {
    SimpleString(Bytes),
    SimpleError(Bytes),
    Integer(i64),
    BulkString(Bytes),
    Array(Box<[GFrame]>),
    Null,
}

impl Display for GFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.bytes().as_ref()))
    }
}

#[derive(Debug, Error)]
#[error("invalid frame type")]
pub struct InvalidFrameType;

impl GFrame {
    pub fn bytes(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        self.read_bytes(&mut bytes);
        bytes.freeze()
    }

    fn read_bytes(&self, bytes: &mut BytesMut) {
        match self {
            GFrame::SimpleString(value) => {
                let first_byte = SIMPLE_STRING_FIRST_BYTE;
                bytes.reserve(size_of_val(&first_byte) + value.len() + TERMINATOR.len());
                bytes.put_u8(first_byte);
                bytes.put(value.as_ref());
                bytes.put(TERMINATOR.as_ref());
            },
            GFrame::SimpleError(value) => {
                let first_byte = SIMPLE_ERROR_FIRST_BYTE;
                bytes.reserve(size_of_val(&first_byte) + value.len() + TERMINATOR.len());
                bytes.put_u8(first_byte);
                bytes.put(value.as_ref());
                bytes.put(TERMINATOR.as_ref());
            },
            GFrame::Integer(value) => {
                let first_byte = INTEGER_FIRST_BYTE;
                bytes.reserve(size_of_val(&first_byte) + size_of::<i64>() + TERMINATOR.len());
                bytes.put_u8(first_byte);
                bytes.write_fmt(format_args!("{value}")).unwrap();
                bytes.put(TERMINATOR.as_ref());
            },
            GFrame::BulkString(value) => {
                let first_byte = BULK_STRING_FIRST_BYTE;
                // TODO: reserve?
                bytes.put_u8(first_byte);
                bytes.write_fmt(format_args!("{len}", len = value.len())).unwrap();
                bytes.put(TERMINATOR.as_ref());
                bytes.put(value.as_ref());
                bytes.put(TERMINATOR.as_ref());
            },
            GFrame::Array(frames) => {
                for frame in frames {
                    frame.read_bytes(bytes);
                }
            },
            GFrame::Null => {
                let first_byte = NULL_FIRST_BYTE;
                bytes.reserve(size_of_val(&first_byte) + TERMINATOR.len());
                bytes.put_u8(first_byte);
                bytes.put(TERMINATOR.as_ref());
            },
        }
    }

    pub fn as_simple_string(&self) -> Result<Bytes, InvalidFrameType> {
        match self {
            GFrame::SimpleString(value) => Ok(value.clone()),
            _ => Err(InvalidFrameType),
        }
    }

    pub fn as_simple_error(&self) -> Result<Bytes, InvalidFrameType> {
        match self {
            GFrame::SimpleError(value) => Ok(value.clone()),
            _ => Err(InvalidFrameType),
        }
    }

    pub fn as_integer(&self) -> Result<i64, InvalidFrameType> {
        match self {
            GFrame::Integer(value) => Ok(*value),
            _ => Err(InvalidFrameType),
        }
    }

    pub fn as_bulk_string(&self) -> Result<Bytes, InvalidFrameType> {
        match self {
            GFrame::BulkString(value) => Ok(value.clone()),
            _ => Err(InvalidFrameType),
        }
    }
    
    pub fn as_array(&self) -> Result<&Box<[GFrame]>, InvalidFrameType> {
        match self {
            GFrame::Array(value) => Ok(value),
            _ => Err(InvalidFrameType),
        }
    }
}
