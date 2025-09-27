use futures::future::join_all;
use goosekv_protocol::{
    command::DelGCommand, data_type::GInteger, frame::GFrame
};

use crate::{
    processor::handler::Handler,
    storage::{
        request::DeleteRequest,
        router::StorageRouter,
    },
};

pub struct DelHandler;

impl Handler<DelGCommand> for DelHandler {
    async fn handle(&self, command: DelGCommand, storage: &StorageRouter) -> GFrame {
        let tasks =
            command.keys.iter().map(|key| storage.delete(DeleteRequest { key: key.clone() }));
        let deleted =
            join_all(tasks).await.iter().filter(|response| response.value.is_some()).count();
        GFrame::Integer(GInteger::new(deleted as i64))
    }
}
