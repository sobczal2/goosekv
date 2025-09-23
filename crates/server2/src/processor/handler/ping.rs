use bytes::Bytes;
use goosekv_protocol::{command::PingGCommand, frame::Frame};

use crate::processor::handler::Handler;

const PONG_MESSAGE: &[u8] = b"PONG";

pub struct PingHandler;

impl Handler<PingGCommand> for PingHandler {
    async fn handle(&self, command: PingGCommand) -> Frame {
        match command.message {
            Some(message) => Frame::BulkString(message),
            None => Frame::SimpleString(Bytes::from_static(PONG_MESSAGE)),
        }
    }
}
