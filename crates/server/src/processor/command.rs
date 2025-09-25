use glommio::net::TcpStream;
use goosekv_protocol::stream::GFrameStream;

pub enum ProcessorCommand {
    Process(ProcessCommand),
}

pub struct ProcessCommand {
    pub stream: GFrameStream<TcpStream>,
}
