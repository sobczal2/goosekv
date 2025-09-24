use goosekv_protocol::{command::GetGCommand, frame::GFrame};
use tracing::info;

use crate::{processor::handler::Handler, storage::{request::GetRequest, router::StorageRouter}};


pub struct GetHandler;

impl Handler<GetGCommand> for GetHandler {
    async fn handle(&self, command: GetGCommand, storage: StorageRouter) -> GFrame {
        info!("getting");
        let response = storage.get(GetRequest { key: command.key }).await;
        info!("got");

        match response.value {
            Some(value) => GFrame::BulkString(value.data.bytes()),
            None => GFrame::Null,
        }
    }
}
