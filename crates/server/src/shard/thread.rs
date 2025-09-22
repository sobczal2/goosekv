use std::{cell::RefCell, collections::HashMap, net::SocketAddr, rc::Rc};

use glommio::{channels::channel_mesh::{FullMesh, PartialMesh, Receivers, Role, Senders}, executor, spawn_local, spawn_local_into, task::JoinHandle, ExecutorJoinHandle, Latency, Shares, TaskQueueHandle};
use goosekv_protocol::command::{Key, Value};
use tracing::info;

use crate::{acceptor::Acceptor, processor::command::{Command, GetResponse, SetResponse}};

pub struct Thread {
        mesh: FullMesh<Command>,
        addr: SocketAddr,
}

impl Thread {
    pub fn new(
        mesh: FullMesh<Command>,
        addr: SocketAddr,
    ) -> Self {
        Self { mesh, addr }
    }
    pub fn start(
        self,
    ) -> ExecutorJoinHandle<Self> {
        glommio::LocalExecutorBuilder::default()
            .name("WO")
            .spawn(async move || { self.run().await })
            .expect("failed to spawn local executor")
    }

    async fn run(self) -> Self {
        let (senders, receivers) = self.mesh.clone().join().await.expect("failed to join mesh");

        let acceptor_handle = run_acceptor(senders, self.addr);
        // let processor_handle = run_processor(receivers);

        acceptor_handle.await.unwrap();
        // processor_handle.await.unwrap();

        self
    }

}

fn run_acceptor(senders: Senders<Command>, addr: SocketAddr) -> JoinHandle<()> {
    let acceptor = Acceptor::bind(addr, senders);
    let task_queue = executor().create_task_queue(Shares::default(), Latency::NotImportant, "ACCEPTOR");

    spawn_local_into(async move {
        acceptor.run().await;
    }, task_queue).unwrap().detach()

}

fn run_processor(receivers: Receivers<Command>) -> JoinHandle<()>{
    let map = Rc::new(RefCell::new(HashMap::<Key, Value>::new()));
    let task_queue = executor().create_task_queue(Shares::default(), Latency::NotImportant, "PROCESSOR");

    // TODO: refactor
    spawn_local_into(async move {
        let receivers = Rc::new(receivers);
        (0..receivers.nr_producers()).for_each(|i| {
            if i != receivers.peer_id() {
                let receivers = receivers.clone();
                let map = map.clone();
                spawn_local(async move {
                    while let Some(command) = receivers.recv_from(i).await.unwrap() {
                        info!("got command: {command:?}");
                        match command {
                            Command::Get(key, shared_sender) => {
                                info!("handle GET for key: {:?}", key);
                                let value = map.borrow().get(&key).cloned();
                                shared_sender.connect().await.send(GetResponse { value }).await.unwrap();
                            }
                            Command::Set(key, value, shared_sender) => {
                                info!("handle SET for key: {:?}", key);
                                map.borrow_mut().insert(key, value);
                                shared_sender.connect().await.send(SetResponse).await.unwrap();
                            }
                        }
                    }
                }).detach();
            } else {
            }
        });
    }, task_queue).unwrap().detach()
}

