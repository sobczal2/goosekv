use glommio::channels::channel_mesh::Senders;
use goosekv_protocol::frame::Frame;

use crate::worker::command::WorkerCommand;

pub mod get;
pub mod ping;
pub mod set;

pub trait Handler<C> {
    fn handle(
        &self,
        command: C,
        senders: &Senders<WorkerCommand>,
    ) -> impl std::future::Future<Output = Frame>;
}
