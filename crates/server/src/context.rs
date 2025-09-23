use glommio::channels::shared_channel::{
    self,
    SharedReceiver,
    SharedSender,
};
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
    respond: SharedSender<Frame>,
}

impl Context {
    pub fn new(request: Frame) -> (Self, SharedReceiver<Frame>) {
        let (sender, receiver) = shared_channel::new_bounded(1024);
        (Self { request, respond: sender }, receiver)
    }

    pub fn command(&self) -> command::Result<Command> {
        Command::from_frame(&self.request)
    }

    pub async fn respond<F>(self, frames: F) -> Result<(), ResponseClosed>
    where
        F: IntoIterator<Item = Frame>,
    {
        let sender = self.respond.connect().await;
        for frame in frames.into_iter() {
            sender.send(frame).await.map_err(|_| ResponseClosed)?;
        }

        Ok(())
    }
}
