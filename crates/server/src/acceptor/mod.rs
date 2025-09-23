use std::{io, net::SocketAddr, rc::Rc};

use bytes::Bytes;
use futures::{Sink, SinkExt, StreamExt};
use glommio::{channels::channel_mesh::Senders, enclose, net::{TcpListener, TcpStream}, spawn_local};
use goosekv_protocol::{command::Command as RespCommand, frame::Frame, stream::FrameStream};
use tracing::{error, info};

use crate::{acceptor::handler::{get::GetHandler, ping::PingHandler, set::SetHandler, Handler}, processor::command::{Command, CommandResponse}, router::SourceRouter};

pub mod handler;

pub struct Acceptor {
    listener: TcpListener,
    router: Rc<SourceRouter<Command, CommandResponse>>
}

impl Acceptor {
    pub fn bind(addr: SocketAddr, router: SourceRouter<Command, CommandResponse>) -> Self {
        Self { listener: TcpListener::bind(addr).expect("failed to bind acceptor"), router: Rc::new(router) }
    }

    pub async fn run(self) -> Self {
        while let Ok(stream) = self.listener.accept().await {
            let router = self.router.clone();
            spawn_local(async move { handle_stream(stream, &router).await }).detach();
        }

        self
    }

}

async fn handle_stream(
    mut stream: TcpStream,
    router: &SourceRouter<Command, CommandResponse>
) {
    let mut stream = FrameStream::new(&mut stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(request) => {
                info!("received request: {request}");
                let command = match RespCommand::from_frame(&request) {
                    Ok(command) => command,
                    Err(error) => {
                        respond_with_error(&mut stream, error.into()).await;
                        continue;
                    },
                };

                let response = handle_command(command, router).await;
                let _ = stream.send(response).await.inspect_err(|error| error!("failed to write to sink: {error}"));
            },
            Err(error) => {
                error!("error reading stream: {}", error);
                respond_with_error(&mut stream, error.into()).await;
            },
        }
    }
}

async fn handle_command(
    command: RespCommand,
    router: &SourceRouter<Command, CommandResponse>
) -> Frame {
    match command {
        RespCommand::Ping(ping_command) => PingHandler.handle(ping_command, router).await,
        RespCommand::Get(get_command) => GetHandler.handle(get_command, router).await,
        RespCommand::Set(set_command) => SetHandler.handle(set_command, router).await,
        // TODO: fix
        RespCommand::ConfigGet(_config_get_command) => Frame::Null,
    }
}

async fn respond_with_error<S>(sink: &mut S, error: anyhow::Error)
where S: Sink<Frame, Error = io::Error> + Unpin
{
    let _ = sink
        .send(Frame::SimpleError(Bytes::from(error.to_string().into_bytes())))
        .await
        .inspect_err(|error| error!("failed to write to sink: {error}")); 
}

