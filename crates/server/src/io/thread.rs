use std::{net::SocketAddr};

use futures_lite::{AsyncReadExt, AsyncWriteExt};
use glommio::{channels::{channel_mesh::{PartialMesh, Role}, shared_channel::{self}}, net::TcpListener, prelude::*, ExecutorJoinHandle};
use goosekv_protocol::parser::Parser;

use crate::context::Context;

pub struct Thread {
    addr: SocketAddr,
    mesh: PartialMesh<Context>,
}

impl Thread {
    pub fn new(addr: SocketAddr, mesh: PartialMesh<Context>) -> Self {
        Self { addr, mesh }
    }

}

impl Thread
{
    pub fn start(self) -> ExecutorJoinHandle<Self>
    {
        glommio::LocalExecutorBuilder::default()
            .name("IoThread")
            .spawn(async move || { self.run().await })
            .expect("failed to spawn local executor")
    }

    async fn run(self) -> Self {
        let listener = TcpListener::bind(self.addr).expect("failed to bind listener");
        let (sender, _) = self.mesh.clone().join(Role::Producer).await.unwrap();
        let mut parser = Parser::new();

        loop {
            match listener.accept().await {
                Ok(mut stream) => {
                    let (respond_sender, respond_receiver) = shared_channel::new_bounded(16);
                    let read = stream.read(parser.buf_mut()).await.unwrap();
                    let context = Context::new(parser.parse().unwrap().unwrap(), respond_sender);
                    sender.send_to(context.get_shard(sender.nr_consumers()), context).await.unwrap();
                    let respond_receiver = respond_receiver.connect().await;
                    while let Some(response_frame) = respond_receiver.recv().await {
                        stream.write_all(b"test").await.unwrap();
                    }
                },
                Err(err) => {
                    println!("listener.accept failed with: {err:?}");
                    break;
                },
            }
        }

        self
    }
}
