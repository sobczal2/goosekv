use std::{
    rc::Rc,
    time::Duration,
};

use bytes::Bytes;
use futures::{
    SinkExt,
    StreamExt,
};
use glommio::{
    Latency,
    Shares,
    channels::local_channel::{
        self,
        LocalReceiver,
    },
    executor,
    net::TcpStream,
    spawn_local,
    spawn_local_into,
};
use goosekv_protocol::{
    command::GCommand,
    frame::GFrame,
    stream::GFrameStream,
};
use tracing::{
    error,
    info,
};

use crate::{
    processor::{
        command::{
            ProcessCommand,
            ProcessorCommand,
        },
        handle::ProcessorHandle,
        handler::handle_gcommand,
    },
    storage::router::StorageRouter,
};

pub struct ProcessorActor;

impl Default for ProcessorActor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessorActor {
    pub fn new() -> Self {
        Self
    }

    pub fn run(self, router: StorageRouter) -> (impl Future<Output = ()>, ProcessorHandle) {
        let (sender, receiver) = local_channel::new_bounded(32);
        let task_queue = executor().create_task_queue(
            Shares::default(),
            Latency::Matters(Duration::from_millis(1)),
            "PROCESSOR",
        );
        let task = spawn_local_into(async { run(receiver, router).await }, task_queue).unwrap();
        (task, ProcessorHandle::new(sender))
    }
}

async fn run(receiver: LocalReceiver<ProcessorCommand>, router: StorageRouter) {
    let router = Rc::new(router);
    while let Some(command) = receiver.recv().await {
        match command {
            ProcessorCommand::Process(process_command) => {
                let router = router.clone();
                spawn_local(async move { process(process_command, router.clone()).await }).detach();
            }
        }
    }
}

async fn process(mut command: ProcessCommand, router: Rc<StorageRouter>) {
    info!("started processing");
    while let Some(frame) = command.stream.next().await {
        info!("new frame");
        match frame {
            Ok(frame) => {
                let response = handle_frame(frame, &router).await;
                if let Err(error) = command.stream.send(response).await {
                    error!("failed to respond: {error}");
                }
            }
            Err(error) => {
                let message = format!("invalid frame: {error}");
                error!("{message}");
                handle_error(&mut command.stream, message.as_str()).await;
            }
        }
    }
}

async fn handle_frame(frame: GFrame, router: &StorageRouter) -> GFrame {
    let command = GCommand::from_frame(&frame);
    match command {
        Ok(command) => handle_gcommand(command, router).await,
        Err(error) => {
            let message = format!("invalid command: {error}");
            error!("{message}");
            error_frame(&message)
        }
    }
}

fn error_frame(message: &str) -> GFrame {
    GFrame::SimpleError(Bytes::copy_from_slice(message.as_bytes()))
}

async fn handle_error(stream: &mut GFrameStream<TcpStream>, message: &str) {
    error!("{message}");
    if let Err(error) = stream.send(error_frame(message)).await {
        error!("failed to respond: {error}");
    }
}
