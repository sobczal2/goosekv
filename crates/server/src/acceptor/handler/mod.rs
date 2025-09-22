use glommio::channels::channel_mesh::Senders;
use goosekv_protocol::frame::Frame;
use goosekv_protocol::command::Command as RespCommand;

use crate::acceptor::handler::set::SetHandler;
use crate::{acceptor::handler::{get::GetHandler, ping::PingHandler}, processor::command::Command};

pub mod get;
pub mod ping;
pub mod set;

pub trait Handler<C> {
    fn handle(
        &self,
        command: C,
        senders: &Senders<Command>,
    ) -> impl Future<Output = Frame>;
}
