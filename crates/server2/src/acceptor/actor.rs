use std::net::SocketAddr;

use glommio::net::TcpListener;
use goosekv_protocol::stream::GFrameStream;
use tracing::info;

use crate::processor::{command::ProcessCommand, handle::ProcessorHandle};

pub struct AcceptorActor {
    addr: SocketAddr,
}

impl AcceptorActor {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn run(self, mut handle: ProcessorHandle) {
        let listener = TcpListener::bind(self.addr).unwrap();

        while let Ok(tcp_stream) = listener.accept().await {
            info!("accepted");
            let stream = GFrameStream::new(tcp_stream);
            handle.process(ProcessCommand { stream }).await;
        }
    }
}

