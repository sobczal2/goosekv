use std::{net::SocketAddr, rc::Rc};

use futures_lite::{
    AsyncWriteExt, StreamExt
};
use glommio::{
    channels::shared_channel::{
        self,
        ConnectedSender,
        SharedReceiver,
        SharedSender,
    }, enclose, net::{
        TcpListener, TcpStream,
    }, spawn_local, ExecutorJoinHandle
};
use goosekv_protocol::{
    driver::Driver,
    frame::Frame,
};
use tracing::{
    error,
    info, warn,
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
            spawn_local(enclose!((sender, move stream) async {handle_stream(stream, sender).await })).detach();
        }

        error!("listener.accept failed");

        self
    }
}


async fn handle_stream(
    mut stream: TcpStream,
    sender: Rc<ConnectedSender<Context>>,
) {
    let mut driver = Driver::new();
    match driver.handle(&mut stream).await {
        Ok(frames) => {
            let sender = sender.clone();
            for frame in frames {
                let (respond_sender, respond_receiver) = shared_channel::new_bounded(256);
                let context = Context::new(frame, respond_sender);
                sender.send(context).await.unwrap();
                
                let respond_receiver = respond_receiver.connect().await;
                while let Some(bytes) = respond_receiver.recv().await {
                    stream.write_all(&bytes).await.unwrap();
                }
            }
        },
        Err(error) => {
            error!("error reading stream: {}", error);
            stream
                .write_all(&Frame::SimpleError(error.to_string()).bytes())
                .await
                .unwrap();
        },
    }
}
