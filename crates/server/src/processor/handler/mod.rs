use goosekv_protocol::{
    command::GCommand,
    frame::GFrame,
};

use crate::{
    processor::handler::{
        del::DelHandler,
        exists::ExistsHandler,
        get::GetHandler,
        incr::IncrHandler,
        ping::PingHandler,
        set::SetHandler,
    },
    storage::router::StorageRouter,
};

pub mod del;
pub mod exists;
pub mod get;
pub mod incr;
pub mod ping;
pub mod set;

pub trait Handler<C> {
    fn handle(&self, command: C, storage: &StorageRouter) -> impl Future<Output = GFrame>;
}

pub async fn handle_gcommand(command: GCommand, storage: &StorageRouter) -> GFrame {
    match command {
        GCommand::Ping(ping_command) => PingHandler.handle(ping_command, storage).await,
        GCommand::Get(get_command) => GetHandler.handle(get_command, storage).await,
        GCommand::Set(set_command) => SetHandler.handle(set_command, storage).await,
        GCommand::Del(del_command) => DelHandler.handle(del_command, storage).await,
        GCommand::Exists(exists_command) => ExistsHandler.handle(exists_command, storage).await,
        GCommand::Incr(incr_gcommand) => IncrHandler.handle(incr_gcommand, storage).await,
        GCommand::Decr(_decr_gcommand) => todo!("not implemented"),
        GCommand::ConfigGet(_config_get_command) => GFrame::Null,
    }
}
