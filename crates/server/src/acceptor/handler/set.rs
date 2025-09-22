use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use bytes::Bytes;
use glommio::channels::{
    channel_mesh::Senders,
    shared_channel,
};
use goosekv_protocol::{
    command::SetCommand,
    frame::Frame,
};
use tracing::{
    error,
    info,
};

use crate::{acceptor::handler::Handler, processor::command::Command};


const OK_MESSAGE: &[u8] = b"OK";
const INTERNAL_ERROR_MESSAGE: &[u8] = b"internal error"; // TODO: global?

pub struct SetHandler;

impl Handler<goosekv_protocol::command::SetCommand> for SetHandler {
    async fn handle(&self, command: SetCommand, senders: &Senders<Command>) -> Frame {
        let mut hasher = DefaultHasher::default();
        command.key.hash(&mut hasher);
        let hash = hasher.finish();

        let target = (hash % senders.nr_consumers() as u64) as usize;

        let (sender, receiver) = shared_channel::new_bounded(1);
        let worker_command = Command::Set(command.key, command.value, sender);

        info!("sending command {:?} to worker {}", worker_command, target);
        senders.send_to(target, worker_command).await.unwrap();

        if (receiver.connect().await.recv().await).is_some() {
            Frame::SimpleString(Bytes::from_static(OK_MESSAGE))
        } else {
            error!("worker {} died", target);
            Frame::SimpleError(Bytes::from_static(INTERNAL_ERROR_MESSAGE))
        }
    }
}
