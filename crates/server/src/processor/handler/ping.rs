use goosekv_protocol::{
    command::PingGCommand, data_type::GString, frame::GFrame
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
            Some(message) => GFrame::BulkString(message.clone()),
            None => GFrame::SimpleString(GString::from_static(PONG_MESSAGE)),
        }
    }
}
