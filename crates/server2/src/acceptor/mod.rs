use std::net::SocketAddr;

use glommio::{channels::local_channel::LocalSender, net::{TcpListener, TcpStream}, ExecutorJoinHandle};
use goosekv_protocol::stream::FrameStream;

pub struct Acceptor {
    addr: SocketAddr,
}

impl Acceptor {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn start(&self, sender: LocalSender<FrameStream<TcpStream>>) {
        let listener = TcpListener::bind(self.addr).unwrap();

        while let Ok(tcp_stream) = listener.accept().await {
            let frame_stream = FrameStream::new(tcp_stream);
            sender.send(frame_stream).await.unwrap();
        }
    }
}

