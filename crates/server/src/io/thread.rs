use std::{io::Read, net::SocketAddr, pin};

use anyhow::bail;
use futures_lite::{pin, AsyncReadExt, AsyncWriteExt};
use glommio::{channels::{channel_mesh::{PartialMesh, Role, Senders}, shared_channel::{self, SharedReceiver}}, net::{TcpListener, TcpStream}, prelude::*, ExecutorJoinHandle};
use goosekv_protocol::{driver::{AsyncDriver, DriverResult}, frame::Frame, parser::Parser};

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
        let (senders, _) = self.mesh.clone().join(Role::Producer).await.unwrap();

        loop {
            match listener.accept().await {
                Ok(mut stream) => {
                    match handle_stream(&mut stream, &senders).await {
                        Ok(receiver) => {
                            let receiver = receiver.connect().await;
                            while let Some(bytes) = receiver.recv().await {
                                stream.write_all(&bytes).await.unwrap();
                            }
                        },
                        Err(err) => stream.write_all(&Frame::SimpleError(err.to_string()).bytes()).await.unwrap(),
                    }
                }
                Err(err) => {
                    println!("listener.accept failed with: {err:?}");
                    break;
                },
            }
        }

        self
    }
}

async fn handle_stream(stream: &mut TcpStream, senders: &Senders<Context>) -> anyhow::Result<SharedReceiver<Box<[u8]>>> {
    pin!(stream);

    let driver = AsyncDriver::new();
    let request = driver.handle(&mut stream).await?;

    let (sender, receiver) = shared_channel::new_bounded(16);
    let context = Context::new(request, sender);
    if senders.send_to(context.get_shard(senders.nr_consumers()), context).await.is_err() {
        bail!("failed to distribute command");
    }

    Ok(receiver)
}
