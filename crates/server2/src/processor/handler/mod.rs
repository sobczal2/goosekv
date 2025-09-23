use goosekv_protocol::{command::Command, frame::Frame};

use crate::processor::handler::ping::PingHandler;

pub mod ping;

pub trait Handler<C> {
    fn handle(&self, command: C) -> impl Future<Output = Frame>;
}

pub async fn handle_command(command: Command) -> Frame {
    match command {
        Command::Ping(ping_command) => PingHandler.handle(ping_command).await,
        Command::Get(get_command) => todo!(),
        Command::Set(set_command) => todo!(),
        Command::ConfigGet(config_get_command) => todo!(),
    }
}
