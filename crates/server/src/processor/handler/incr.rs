use std::sync::Arc;

use goosekv_protocol::{
    command::IncrGCommand,
    data_type::{
        GInteger,
        GString,
    },
    frame::GFrame,
};

use crate::{
    processor::handler::Handler,
    storage::{
        request::UpdateRequest,
        router::StorageRouter,
        value::{
            Data,
            Value,
        },
    },
};

pub struct IncrHandler;

impl Handler<IncrGCommand> for IncrHandler {
    async fn handle(&self, command: IncrGCommand, storage: &StorageRouter) -> GFrame {
        let update_f = Arc::new(|value: Option<Value>| match value {
            Some(Value { data: Data::Integer(existing_integer) }) => {
                existing_integer.checked_add(1).map(|value| Value { data: Data::Integer(value) })
            }
            Some(value) => Some(value),
            None => Some(Value { data: Data::Integer(GInteger::new(1)) }),
        });

        let response = storage.update(UpdateRequest { key: command.key, f: update_f }).await;

        match response.updated {
            Some(Value { data: Data::Integer(updated) }) => GFrame::Integer(updated),
            Some(_) => GFrame::SimpleError(GString::copy_from_slice(b"not an integer")),
            None => {
                GFrame::SimpleError(GString::copy_from_slice(b"tried to increment with overflow"))
            }
        }
    }
}
