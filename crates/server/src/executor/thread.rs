use std::rc::Rc;

use bytes::Bytes;
use glommio::{
    ExecutorJoinHandle,
    channels::{
        channel_mesh::{
            PartialMesh,
            Role,
            Senders,
        },
        shared_channel::SharedReceiver,
    },
    enclose,
    spawn_local,
};
use goosekv_protocol::{
    command::Command,
    frame::Frame,
};
use tracing::{
    error,
    info,
};

use crate::{
    context::Context,
    executor::handler::{
        Handler,
        get::GetHandler,
        ping::PingHandler,
        set::SetHandler,
    },
    worker::command::WorkerCommand,
};

pub struct Thread;

impl Thread {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Thread {
    fn default() -> Self {
        Self::new()
    }
}

impl Thread {
    pub fn start(
        self,
        mesh: PartialMesh<WorkerCommand>,
        receiver: SharedReceiver<Context>,
    ) -> ExecutorJoinHandle<Self> {
        glommio::LocalExecutorBuilder::default()
            .name("EX")
            .spawn(async move || self.run(mesh, receiver).await)
            .expect("failed to spawn local executor")
    }

    async fn run(
        self,
        mesh: PartialMesh<WorkerCommand>,
        receiver: SharedReceiver<Context>,
    ) -> Self {
        info!("executor thread started");
        let receiver = receiver.connect().await;
        let (senders, _) = mesh.join(Role::Producer).await.unwrap();

        let senders = Rc::new(senders);

        while let Some(context) = receiver.recv().await {
            spawn_local(enclose!((senders) async move {
                match handle_context(&context, &senders).await {
                    Ok(frame) => {
                        if context.respond([frame]).await.is_err() {
                            error!("failed to respond");
                        }
                    }
                    Err(error) => {
                        error!("execution failed with error: {}", error);
                        if context.respond([Frame::SimpleError(Bytes::from(error.to_string().into_bytes()))]).await.is_err() {
                            error!("failed to respond");
                        }
                    }
                }
            })).detach();
        }

        self
    }
}

async fn handle_context(
    context: &Context,
    senders: &Senders<WorkerCommand>,
) -> anyhow::Result<Frame> {
    let command = context.command()?;

    let frame = match command {
        Command::Ping(ping_command) => PingHandler.handle(ping_command, senders).await,
        Command::Get(get_command) => GetHandler.handle(get_command, senders).await,
        Command::Set(set_command) => SetHandler.handle(set_command, senders).await,
        // TODO: clean
        Command::ConfigGet(_) => Frame::Array(
            vec![
                Frame::BulkString(Bytes::from("save".to_string().into_bytes())),
                Frame::BulkString(Bytes::new()),
            ]
            .into_boxed_slice(),
        ),
    };

    Ok(frame)
}
