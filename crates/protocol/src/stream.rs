use std::{
    io,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use bytes::{
    Buf,
    BytesMut,
};
use futures::{
    AsyncRead,
    AsyncWrite,
    Sink,
    Stream,
};
use thiserror::Error;

use crate::{
    frame::Frame,
    parser::{
        ParseError,
        Parser,
    },
};

#[derive(Debug, Error)]
pub enum FrameStreamError {
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("parsing error: {0}")]
    Parsing(#[from] ParseError),
}

pub type FrameStreamResult<T> = std::result::Result<T, FrameStreamError>;

pub struct FrameStream<'a, I> {
    inner: &'a mut I,
    parser: Parser,
    tmp: [u8; 1024],
    write_buf: BytesMut,
}

impl<'a, I> FrameStream<'a, I> {
    pub fn new(inner: &'a mut I) -> Self {
        Self { inner, parser: Parser::new(), tmp: [0u8; 1024], write_buf: BytesMut::new() }
    }
}

impl<'a, I> Stream for FrameStream<'a, I>
where
    I: AsyncRead + Unpin,
{
    type Item = FrameStreamResult<Frame>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();

        match me.parser.parse() {
            Ok(Some(frame)) => return Poll::Ready(Some(Ok(frame))),
            Ok(None) => (),
            Err(error) => return Poll::Ready(Some(Err(error.into()))),
        }

        match Pin::new(&mut me.inner).poll_read(cx, &mut me.tmp) {
            Poll::Ready(Ok(0)) => {
                if me.parser.buf().is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Err(FrameStreamError::UnexpectedEof)))
                }
            }
            Poll::Ready(Ok(n)) => {
                me.parser.buf_mut().extend_from_slice(&me.tmp[..n]);
                if let Some(frame) = me.parser.parse()? {
                    return Poll::Ready(Some(Ok(frame)));
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Err(error)) => Poll::Ready(Some(Err(error.into()))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'a, I> Sink<Frame> for FrameStream<'a, I>
where
    I: AsyncWrite + Unpin,
{
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Frame) -> Result<(), Self::Error> {
        self.write_buf.extend_from_slice(&item.bytes());
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut buf = self.write_buf.split();

        while !buf.is_empty() {
            let n = match Pin::new(&mut self.inner).poll_write(cx, &buf) {
                Poll::Ready(Ok(n)) => n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            };
            buf.advance(n);
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let flush_result = self.as_mut().poll_flush(cx);

        match flush_result {
            Poll::Ready(Ok(())) => Pin::new(&mut self.inner).poll_close(cx),
            other => other,
        }
    }
}
