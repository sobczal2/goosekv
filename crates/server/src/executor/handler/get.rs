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
    command::GetCommand,
    frame::Frame,
};
use tracing::{
    error,
    info,
};

use crate::{
    executor::handler::Handler,
    worker::command::WorkerCommand,
};

const INTERNAL_ERROR_MESSAGE: &[u8] = b"internal error"; // TODO: global?

pub struct GetHandler;

impl Handler<GetCommand> for GetHandler {
    async fn handle(&self, command: GetCommand, senders: &Senders<WorkerCommand>) -> Frame {
        let mut hasher = DefaultHasher::default();
        command.key.hash(&mut hasher);
        let hash = hasher.finish();

        let target = (hash % senders.nr_consumers() as u64) as usize;

        let (sender, receiver) = shared_channel::new_bounded(1);
        let worker_command = WorkerCommand::Get(command.key, sender);

        info!("sending command {:?} to worker {}", worker_command, target);
        senders.send_to(target, worker_command).await.unwrap();

        if let Some(mut response) = receiver.connect().await.recv().await {
            if response.value.is_some() {
                Frame::BulkString(response.value.take().unwrap().0.into())
            } else {
                Frame::Null
            }
        } else {
            error!("worker {} died", target);
            Frame::SimpleError(Bytes::from_static(INTERNAL_ERROR_MESSAGE))
        }
    }
}
