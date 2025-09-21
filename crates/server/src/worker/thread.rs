use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use glommio::{
    ExecutorJoinHandle,
    channels::channel_mesh::{
        PartialMesh,
        Role,
    },
    spawn_local,
};
use goosekv_protocol::command::{
    Key,
    Value,
};
use tracing::info;

use crate::worker::command::{
    GetResponse,
    SetResponse,
    WorkerCommand,
};

pub struct Thread;

impl Thread {
    pub fn new() -> Self {
        Thread
    }
}

impl Default for Thread {
    fn default() -> Self {
        Self::new()
    }
}

impl Thread {
    pub fn start(self, mesh: PartialMesh<WorkerCommand>) -> ExecutorJoinHandle<Self> {
        glommio::LocalExecutorBuilder::default()
            .name("WO")
            .spawn(async move || self.run(mesh).await)
            .expect("failed to spawn local executor")
    }

    async fn run(self, mesh: PartialMesh<WorkerCommand>) -> Self {
        info!("worker thread started");
        let (_, receivers) = mesh.join(Role::Consumer).await.unwrap();

        let storage: Rc<RefCell<HashMap<Key, Value>>> = Default::default();

        while let Some(command) = receivers.recv_from(0).await.unwrap() {
            let storage = storage.clone();
            spawn_local(async {
                handle_command(command, storage).await;
            })
            .await;
        }

        self
    }
}

async fn handle_command(command: WorkerCommand, storage: Rc<RefCell<HashMap<Key, Value>>>) {
    match command {
        WorkerCommand::Get(key, shared_sender) => {
            info!("handle GET for key: {:?}", key);
            let value = storage.borrow().get(&key).cloned();
            shared_sender.connect().await.send(GetResponse { value }).await.unwrap();
        }
        WorkerCommand::Set(key, value, shared_sender) => {
            info!("handle SET for key: {:?}", key);
            storage.borrow_mut().insert(key, value);
            shared_sender.connect().await.send(SetResponse).await.unwrap();
        }
    }
}
