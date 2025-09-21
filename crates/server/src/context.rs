use std::str::Bytes;

use glommio::channels::shared_channel::SharedSender;
use goosekv_protocol::frame::Frame;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("response closed")]
pub struct ResponseClosed;

#[derive(Debug)]
pub struct Context
{
    request: Frame,
    respond: SharedSender<Box<[u8]>>,
}

impl Context
{
    pub fn new(request: Frame, respond: SharedSender<Box<[u8]>>) -> Self {
        Self { request, respond }
    }
    pub fn request(&self) -> &Frame {
        &self.request
    }

    pub async fn respond<F>(self, frames: F) -> Result<(), ResponseClosed>
        where F: IntoIterator<Item = Frame>
    {
        let sender = self.respond.connect().await;
        for frame in frames.into_iter() {
            sender.send(frame.bytes()).await.map_err(|_| ResponseClosed)?;
        }

        Ok(())
    }

    pub fn get_shard(&self, nr_shards: usize) -> usize {
        match &self.request {
            Frame::SimpleString(value) => {
                match value.trim() {
                    "PING" => 1,
                    "PONG" => 2,
                    _ => 3,
                }
            },
            Frame::SimpleError(_) => todo!(),
            Frame::Integer(_) => todo!(),
            Frame::BulkString(_) => todo!(),
            Frame::Array(frames) => todo!(),
            Frame::Null => todo!(),
        }
    }
}
