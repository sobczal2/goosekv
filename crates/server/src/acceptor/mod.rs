use std::{io, net::SocketAddr, rc::Rc};

use bytes::Bytes;
use futures::{Sink, SinkExt, StreamExt};
use glommio::{channels::channel_mesh::Senders, net::{TcpListener, TcpStream}, spawn_local};
use goosekv_protocol::{command::Command as RespCommand, frame::Frame, stream::FrameStream};
use tracing::error;

use crate::{acceptor::handler::{get::GetHandler, ping::PingHandler, set::SetHandler, Handler}, processor::command::Command};

pub mod handler;

pub struct Acceptor {
    listener: TcpListener,
    senders: Rc<Senders<Command>>
}

impl Acceptor {
    pub fn bind(addr: SocketAddr, senders: Senders<Command>) -> Self {
        Self { listener: TcpListener::bind(addr).expect("failed to bind acceptor"), senders: Rc::new(senders) }
    }

    pub async fn run(self) -> Self {
        while let Ok(stream) = self.listener.accept().await {
            let senders = self.senders.clone();
            spawn_local(async move { handle_stream(stream, &senders).await }).detach();
        }

        self
    }

}

async fn handle_stream(
    mut stream: TcpStream,
    senders: &Senders<Command>
) {
    let mut stream = FrameStream::new(&mut stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(request) => {
                let command = match RespCommand::from_frame(&request) {
                    Ok(command) => command,
                    Err(error) => {
                        respond_with_error(&mut stream, error.into()).await;
                        continue;
                    },
                };

                let response = handle_command(command, senders).await;
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
    senders: &Senders<Command>,
) -> Frame {
    match command {
        RespCommand::Ping(ping_command) => PingHandler.handle(ping_command, senders).await,
        RespCommand::Get(get_command) => GetHandler.handle(get_command, senders).await,
        RespCommand::Set(set_command) => SetHandler.handle(set_command, senders).await,
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

