use goosekv_protocol::{
    command::SetGCommand,
    data_type::GString,
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
                value: Value { data: Data::from_gstring(command.value) },
            })
            .await;

        GFrame::SimpleString(GString::from_static(OK_MESSAGE))
    }
}
