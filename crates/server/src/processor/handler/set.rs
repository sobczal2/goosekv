use bytes::Bytes;
use goosekv_protocol::{
    command::SetGCommand,
    frame::GFrame,
};

use crate::{
    processor::handler::Handler,
    storage::{
        request::SetRequest,
        router::StorageRouter,
        value::{
            Data,
            Value,
        },
    },
};

pub struct SetHandler;

const OK_MESSAGE: &[u8] = b"OK";

impl Handler<SetGCommand> for SetHandler {
    async fn handle(&self, command: SetGCommand, storage: &StorageRouter) -> GFrame {
        storage
            .set(SetRequest {
                key: command.key,
                value: Value { data: Data::String(command.value) },
            })
            .await;

        GFrame::SimpleString(Bytes::from_static(OK_MESSAGE))
    }
}
