use goosekv_protocol::frame::Frame;

use crate::processor::command::CommandResponse;
use crate::router::SourceRouter;
use crate::{acceptor::handler::{get::GetHandler, ping::PingHandler}, processor::command::Command};

pub mod get;
pub mod ping;
pub mod set;

pub trait Handler<C> {
    fn handle(
        &self,
        command: C,
        router: &SourceRouter<Command, CommandResponse>
    ) -> impl Future<Output = Frame>;
}
