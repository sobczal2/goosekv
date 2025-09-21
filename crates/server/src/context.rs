use glommio::channels::shared_channel::SharedSender;
use goosekv_protocol::{
    command::{
        self,
        Command,
    },
    frame::Frame,
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("response closed")]
pub struct ResponseClosed;

#[derive(Debug)]
pub struct Context {
    request: Frame,
    respond: SharedSender<Box<[u8]>>,
}

impl Context {
    pub fn new(request: Frame, respond: SharedSender<Box<[u8]>>) -> Self {
        Self { request, respond }
    }

    pub fn command(&self) -> command::Result<Command<'_>> {
        Command::from_frame(&self.request)
    }

    pub async fn respond<F>(self, frames: F) -> Result<(), ResponseClosed>
    where
        F: IntoIterator<Item = Frame>,
    {
        let sender = self.respond.connect().await;
        for frame in frames.into_iter() {
            sender.send(frame.bytes()).await.map_err(|_| ResponseClosed)?;
        }

        Ok(())
    }
}
