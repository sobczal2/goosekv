use bytes::Bytes;
use goosekv_protocol::{command::PingCommand, frame::Frame};

use crate::processor::handler::Handler;

const PONG_MESSAGE: &[u8] = b"PONG";

pub struct PingHandler;

impl Handler<PingCommand> for PingHandler {
    async fn handle(&self, command: PingCommand) -> Frame {
        match command.message {
            Some(message) => Frame::BulkString(message),
            None => Frame::SimpleString(Bytes::from_static(PONG_MESSAGE)),
        }
    }
}
