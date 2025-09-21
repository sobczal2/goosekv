pub const TERMINATOR: &[u8; 2] = b"\r\n";
pub const SIMPLE_STRING_FIRST_BYTE: u8 = b'+';
pub const SIMPLE_ERROR_FIRST_BYTE: u8 = b'-';
pub const INTEGER_FIRST_BYTE: u8 = b':';
pub const BULK_STRING_FIRST_BYTE: u8 = b'$';
pub const ARRAY_FIRST_BYTE: u8 = b'*';
pub const NULL_FIRST_BYTE: u8 = b'_';

#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(Box<[Frame]>),
    Null,
}
