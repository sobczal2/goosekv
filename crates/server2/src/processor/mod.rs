use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use glommio::{channels::local_channel::LocalReceiver, executor, net::TcpStream, spawn_local, spawn_local_into, Latency, Shares};
use goosekv_protocol::{command::Command, frame::Frame, stream::FrameStream};
use tracing::error;

use crate::processor::handler::handle_command;

mod handler;

pub struct Processor;

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self, receiver: LocalReceiver<FrameStream<TcpStream>>) {
        let task_queue = executor().create_task_queue(Shares::default(), Latency::NotImportant, "PROCESSOR");
        while let Some(stream) = receiver.recv().await {
            spawn_local_into(async { start_processing_stream(stream).await }, task_queue).unwrap().detach();
        }
    }
}

async fn start_processing_stream(mut stream: FrameStream<TcpStream>) {
    while let Some(frame) = stream.next().await {
        match frame {
            Ok(frame) => {
                let response = handle_frame(frame).await;
                if let Err(error) = stream.send(response).await {
                    error!("failed to respond: {error}");
                }
            },
            Err(error) => {
                let message = format!("invalid frame: {error}");
                error!("{message}");
                handle_error(&mut stream, message.as_str()).await;
            },
        }
    }
}

async fn handle_frame(frame: Frame) -> Frame {
    let command = Command::from_frame(&frame);
    match command {
        Ok(command) => {
            handle_command(command).await
        },
        Err(error) => {
            let message = format!("invalid command: {error}");
            error!("{message}");
            error_frame(&message)
        },
    }
}

fn error_frame(message: &str) -> Frame {
    Frame::SimpleError(Bytes::copy_from_slice(message.as_bytes()))
}

async fn handle_error(stream: &mut FrameStream<TcpStream>, message: &str) {
    error!("{message}");
    if let Err(error) = stream.send(error_frame(message)).await {
        error!("failed to respond: {error}");
    }
}

