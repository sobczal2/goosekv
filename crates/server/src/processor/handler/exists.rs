use futures::future::join_all;
use goosekv_protocol::{
    command::ExistsGCommand, data_type::GInteger, frame::GFrame
};

use crate::{
    processor::handler::Handler,
    storage::{
        request::GetRequest,
        router::StorageRouter,
    },
};

pub struct ExistsHandler;

impl Handler<ExistsGCommand> for ExistsHandler {
    async fn handle(&self, command: ExistsGCommand, storage: &StorageRouter) -> GFrame {
        let tasks =
            command.keys.iter().map(|key| storage.get(GetRequest { key: key.clone() }));
        let existing =
            join_all(tasks).await.iter().filter(|response| response.value.is_some()).count();
        GFrame::Integer(GInteger::new(existing as i64))
    }
}
