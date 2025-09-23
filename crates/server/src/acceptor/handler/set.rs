use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use bytes::Bytes;
use goosekv_protocol::{
    command::SetCommand,
    frame::Frame,
};
use tracing::{
    info,
};

use crate::{acceptor::handler::Handler, processor::command::{Command, CommandResponse}, router::SourceRouter};


const OK_MESSAGE: &[u8] = b"OK";
const INTERNAL_ERROR_MESSAGE: &[u8] = b"internal error"; // TODO: global?

pub struct SetHandler;

impl Handler<goosekv_protocol::command::SetCommand> for SetHandler {
    async fn handle(&self, command: SetCommand, router: &SourceRouter<Command, CommandResponse>) -> Frame {
        let mut hasher = DefaultHasher::default();
        command.key.hash(&mut hasher);
        let hash = hasher.finish();

        let target = (hash % router.targets() as u64) as usize;

        let worker_command = Command::Set(command.key, command.value);

        info!("sending command {:?} to worker {}", worker_command, target);
        let _ = router.send(target, worker_command).await;

        Frame::SimpleString(Bytes::from_static(OK_MESSAGE))
    }
}
