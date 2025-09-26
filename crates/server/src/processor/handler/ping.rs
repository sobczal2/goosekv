use bytes::Bytes;
use goosekv_protocol::{
    command::PingGCommand,
    frame::GFrame,
};

use crate::{
    processor::handler::Handler,
    storage::router::StorageRouter,
};

const PONG_MESSAGE: &[u8] = b"PONG";

pub struct PingHandler;

impl Handler<PingGCommand> for PingHandler {
    async fn handle(&self, command: PingGCommand, _storage: &StorageRouter) -> GFrame {
        match command.message {
            Some(message) => GFrame::BulkString(message),
            None => GFrame::SimpleString(Bytes::from_static(PONG_MESSAGE)),
        }
    }
}
