use std::{
    net::SocketAddr,
    rc::Rc,
};

use bytes::Bytes;
use futures::{
    SinkExt,
    StreamExt,
};
use glommio::{
    ExecutorJoinHandle,
    channels::shared_channel::{
        self,
        ConnectedSender,
        SharedReceiver,
        SharedSender,
    },
    enclose,
    net::{
        TcpListener,
        TcpStream,
    },
    spawn_local,
};
use goosekv_protocol::{
    frame::Frame,
    stream::FrameStream,
};
use tracing::{
    error,
    info,
};

use crate::context::Context;

pub struct Thread {
    addr: SocketAddr,
}

impl Thread {
    const CHANNEL_SIZE: usize = 256;

    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

impl Thread {
    pub fn start(self) -> (ExecutorJoinHandle<Self>, SharedReceiver<Context>) {
        let (sender, receiver) = shared_channel::new_bounded(Self::CHANNEL_SIZE);

        let handle = glommio::LocalExecutorBuilder::default()
            .name("IO")
            .spawn(async move || self.run(sender).await)
            .expect("failed to spawn local executor");

        (handle, receiver)
    }

    async fn run(self, sender: SharedSender<Context>) -> Self {
        info!("io thread started");

        let listener = TcpListener::bind(self.addr).expect("failed to bind listener");
        info!("listening on {}", self.addr);

        let sender = Rc::new(sender.connect().await);

        while let Ok(stream) = listener.accept().await {
            info!("client {} connected", stream.peer_addr().unwrap());
            spawn_local(
                enclose!((sender, move stream) async {handle_stream(stream, sender).await }),
            )
            .detach();
        }

        error!("listener.accept failed");

        self
    }
}

async fn handle_stream(mut stream: TcpStream, sender: Rc<ConnectedSender<Context>>) {
    let mut stream = FrameStream::new(&mut stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(frame) => {
                let (context, respond) = Context::new(frame);
                sender.send(context).await.unwrap();

                let respond = respond.connect().await;
                while let Some(frame) = respond.recv().await {
                    stream.send(frame).await.unwrap();
                }
            }
            Err(error) => {
                error!("error reading stream: {}", error);
                stream
                    .send(Frame::SimpleError(Bytes::from(error.to_string().into_bytes())))
                    .await
                    .unwrap();
                break;
            }
        }
    }
}
