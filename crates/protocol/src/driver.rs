use std::iter::repeat;

use futures_lite::{
    AsyncRead,
    AsyncReadExt,
};
use thiserror::Error;

use crate::{
    frame::Frame,
    parser::{
        Parser,
        ParsingError,
    },
};

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("parsing error: {0}")]
    Parsing(#[from] ParsingError),
    #[error("read error: {0}")]
    ReadError(String),
    #[error("input too long")]
    InputTooLong,
}

pub type DriverResult<T> = Result<T, DriverError>;

pub struct Driver {
    parser: Parser,
    buf: [u8; 1024],
}

impl Driver {
    pub fn new() -> Self {
        Driver { parser: Parser::new(), buf: [0u8; 1024] }
    }

    pub fn with_buf_size(size: usize) -> Self {
        Driver { parser: Parser::with_buf_size(size), buf: [0u8; 1024] }
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver {
    pub async fn handle<R: AsyncRead + Unpin>(&mut self, read: &mut R) -> DriverResult<Vec<Frame>> {
        self.read_all(read).await?;

        let mut frames = Vec::new();
        while let Some(frame) = self.parser.parse()? {
            frames.push(frame);
        }

        Ok(frames)
    }

    pub fn reset(&mut self) {
        self.parser.reset();
    }

    async fn read_all<R: AsyncRead + Unpin>(&mut self, read: &mut R) -> DriverResult<()> {
        loop {
            let n = read.read(&mut self.buf).await.map_err(|e| DriverError::ReadError(e.to_string()))?;
            self.parser.buf_mut().extend_from_slice(&self.buf[0..n]);
            if n != self.buf.len() {
                return Ok(());
            }
        }
    }
}
