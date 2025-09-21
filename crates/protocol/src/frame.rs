use std::fmt::{
    self,
    Display,
    Formatter,
};

use thiserror::Error;

pub const TERMINATOR_STR: &str = "\r\n";
pub const SIMPLE_STRING_FIRST_CHAR: char = '+';
pub const SIMPLE_ERROR_FIRST_CHAR: char = '-';
pub const INTEGER_FIRST_CHAR: char = ':';
pub const BULK_STRING_FIRST_CHAR: char = '$';
pub const ARRAY_FIRST_CHAR: char = '*';
pub const NULL_FIRST_CHAR: char = '_';

#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(Box<[Frame]>),
    Null,
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Frame::SimpleString(value) => {
                write!(f, "{SIMPLE_STRING_FIRST_CHAR}{value}{TERMINATOR_STR}")
            }
            Frame::SimpleError(value) => {
                write!(f, "{SIMPLE_ERROR_FIRST_CHAR}{value}{TERMINATOR_STR}")
            }
            Frame::Integer(value) => write!(f, "{INTEGER_FIRST_CHAR}{value}{TERMINATOR_STR}"),
            Frame::BulkString(value) => {
                let len = value.len();
                write!(f, "{BULK_STRING_FIRST_CHAR}{len}{TERMINATOR_STR}{value}{TERMINATOR_STR}")
            }
            Frame::Array(frames) => {
                let len = frames.len();
                write!(f, "{ARRAY_FIRST_CHAR}{len}{TERMINATOR_STR}")?;
                for frame in frames.iter() {
                    frame.fmt(f)?
                }
                Ok(())
            }
            Frame::Null => write!(f, "{NULL_FIRST_CHAR}{TERMINATOR_STR}"),
        }
    }
}

#[derive(Debug, Error)]
#[error("invalid frame type")]
pub struct InvalidFrameType;

impl Frame {
    pub fn bytes(self) -> Box<[u8]> {
        self.to_string().into_bytes().into_boxed_slice()
    }

    pub fn as_simple_string(&self) -> Result<&str, InvalidFrameType> {
        match self {
            Frame::SimpleString(value) => Ok(value),
            _ => Err(InvalidFrameType),
        }
    }

    pub fn as_simple_error(&self) -> Result<&str, InvalidFrameType> {
        match self {
            Frame::SimpleError(value) => Ok(value),
            _ => Err(InvalidFrameType),
        }
    }

    pub fn as_integer(&self) -> Result<i64, InvalidFrameType> {
        match self {
            Frame::Integer(value) => Ok(*value),
            _ => Err(InvalidFrameType),
        }
    }

    pub fn as_bulk_string(&self) -> Result<&str, InvalidFrameType> {
        match self {
            Frame::BulkString(value) => Ok(value),
            _ => Err(InvalidFrameType),
        }
    }
    
    pub fn as_array(&self) -> Result<&Box<[Frame]>, InvalidFrameType> {
        match self {
            Frame::Array(value) => Ok(value),
            _ => Err(InvalidFrameType),
        }
    }
}
