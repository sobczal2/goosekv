use futures_lite::{pin, AsyncRead, AsyncReadExt};
use thiserror::Error;

use crate::{frame::Frame, parser::{Parser, ParsingError}};

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("parsing error: {0}")]
    Parsing(#[from] ParsingError),
    #[error("read error: {0}")]
    ReadError(String),
    #[error("end of bytes")]
    EndOfBytes,
}

pub type DriverResult = Result<Frame, DriverError>;

pub struct AsyncDriver {
    parser: Parser,
}

impl AsyncDriver {
    pub fn new() -> Self {
        AsyncDriver { parser: Parser::new() }
    }
}

impl Default for AsyncDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncDriver
{
    pub async fn handle<R: AsyncRead + Unpin>(mut self, read: &mut R) -> DriverResult {
        loop {
            let read_count = read.read(self.parser.buf_mut()).await.map_err(|e| DriverError::ReadError(e.to_string()))?;

            if read_count == 0 {
                return Err(DriverError::EndOfBytes);
            }

            match self.parser.parse()? {
                Some(frame) => return Ok(frame),
                None => continue,
            }
        }
    }
}
