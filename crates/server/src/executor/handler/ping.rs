use glommio::channels::channel_mesh::Senders;
use goosekv_protocol::{
    command::{
        PingCommand,
    },
    frame::Frame,
};
use tracing::info;

use crate::{
    executor::handler::Handler,
    worker::command::WorkerCommand,
};

pub struct PingHandler;

const DEFAULT_MESSAGE: &str = "PONG";

impl<'a> Handler<PingCommand<'a>> for PingHandler {
    async fn handle(&self, command: PingCommand<'a>, _senders: &Senders<WorkerCommand>) -> Frame {
        info!("handling PING");
        if let Some(message) = command.message {
            return Frame::BulkString(message.to_string());
        }

        Frame::SimpleString(DEFAULT_MESSAGE.to_string())
    }
}
