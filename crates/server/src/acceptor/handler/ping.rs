use bytes::Bytes;
use goosekv_protocol::{
    command::{
        PingCommand,
    },
    frame::Frame,
};
use tracing::info;

use crate::{acceptor::handler::Handler, processor::command::{Command, CommandResponse}, router::SourceRouter};

pub struct PingHandler;

const DEFAULT_MESSAGE: &[u8] = b"PONG";

impl Handler<PingCommand> for PingHandler { 
    async fn handle(&self, command: PingCommand, _router: &SourceRouter<Command, CommandResponse>) -> Frame {
        info!("handling PING");
        if let Some(message) = command.message {
            return Frame::BulkString(message);
        }

        Frame::SimpleString(Bytes::from_static(DEFAULT_MESSAGE))
    }
}
