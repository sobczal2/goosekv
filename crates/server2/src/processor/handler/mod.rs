use goosekv_protocol::{command::GCommand, frame::Frame};

use crate::processor::handler::ping::PingHandler;

pub mod ping;

pub trait Handler<C> {
    fn handle(&self, command: C) -> impl Future<Output = Frame>;
}

pub async fn handle_command(command: GCommand) -> Frame {
    match command {
        GCommand::Ping(ping_command) => PingHandler.handle(ping_command).await,
        GCommand::Get(get_command) => todo!(),
        GCommand::Set(set_command) => todo!(),
        GCommand::ConfigGet(config_get_command) => todo!(),
    }
}
