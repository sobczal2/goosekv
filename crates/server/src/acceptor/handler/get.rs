use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use goosekv_protocol::{
    command::GetGCommand,
    frame::Frame,
};
use tracing::{
    info,
};

use crate::{acceptor::handler::Handler, processor::command::{Command, CommandResponse}, router::SourceRouter};

const INTERNAL_ERROR_MESSAGE: &[u8] = b"internal error"; // TODO: global?

pub struct GetHandler;

impl Handler<GetGCommand> for GetHandler {
    async fn handle(&self, command: GetGCommand, router: &SourceRouter<Command, CommandResponse>) -> Frame {
        let mut hasher = DefaultHasher::default();
        command.key.hash(&mut hasher);
        let hash = hasher.finish();

        let target = (hash % router.targets() as u64) as usize;

        let worker_command = Command::Get(command.key);

        info!("sending command {:?} to worker {}", worker_command, target);
        let response = router.send(target, worker_command).await;
        
        match response {
            CommandResponse::Empty => Frame::Null,
            CommandResponse::Value(value) => Frame::BulkString(value.0.into()),
        }
    }
}
