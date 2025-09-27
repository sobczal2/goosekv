use goosekv_protocol::{
    command::GetGCommand,
    frame::GFrame,
};

use crate::{
    processor::handler::Handler,
    storage::{
        request::GetRequest,
        router::StorageRouter,
    },
};

pub struct GetHandler;

impl Handler<GetGCommand> for GetHandler {
    async fn handle(&self, command: GetGCommand, storage: &StorageRouter) -> GFrame {
        let response = storage.get(GetRequest { key: command.key }).await;

        match response.value {
            Some(value) => GFrame::BulkString(value.data.to_gstring()),
            None => GFrame::Null,
        }
    }
}
