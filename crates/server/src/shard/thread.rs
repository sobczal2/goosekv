use std::{cell::RefCell, collections::HashMap, net::SocketAddr, rc::Rc};

use glommio::{executor, spawn_local_into, task::JoinHandle, ExecutorJoinHandle, Latency, Shares};
use goosekv_protocol::command::{Key, Value};
use tracing::info;

use crate::{acceptor::Acceptor, processor::command::{Command, CommandResponse}, router::{self, Router, SourceRouter, TargetRouter}};

pub struct Thread {
        addr: SocketAddr,
}

impl Thread {
    pub fn new(
        addr: SocketAddr,
    ) -> Self {
        Self { addr }
    }
    pub fn start(
        self,
        router: Router<Command, CommandResponse>,
    ) -> ExecutorJoinHandle<Self> {
        glommio::LocalExecutorBuilder::default()
            .name("WO")
            .spawn(async move || { self.run(router).await })
            .expect("failed to spawn local executor")
    }

    async fn run(self, router: Router<Command, CommandResponse>) -> Self {
        let (source_router, target_router) = router.join().await; 

        let acceptor_handle = run_acceptor(source_router, self.addr);
        let processor_handle = run_processor(target_router);

        acceptor_handle.await.unwrap();
        processor_handle.await.unwrap();

        self
    }

}

fn run_acceptor(router: SourceRouter<Command, CommandResponse>, addr: SocketAddr) -> JoinHandle<()> {
    let acceptor = Acceptor::bind(addr, router);
    let task_queue = executor().create_task_queue(Shares::default(), Latency::NotImportant, "ACCEPTOR");

    spawn_local_into(async move {
        acceptor.run().await;
    }, task_queue).unwrap().detach()

}

fn run_processor(mut router: TargetRouter<Command, CommandResponse>) -> JoinHandle<()>{
    let map = Rc::new(RefCell::new(HashMap::<Key, Value>::new()));
    let task_queue = executor().create_task_queue(Shares::default(), Latency::NotImportant, "PROCESSOR");

    // TODO: refactor
    spawn_local_into(async move {
        loop {
            router.handle(async |request| {
                info!("got command: {request:?}");
                match request {
                    Command::Get(key) => {
                        info!("handle GET for key: {:?}", key);
                        let value = map.borrow().get(&key).cloned();
                        match value {
                            Some(value) => CommandResponse::Value(value),
                            None => CommandResponse::Empty,
                        }
                    }
                    Command::Set(key, value) => {
                        info!("handle SET for key: {:?}", key);
                        map.borrow_mut().insert(key, value);
                        CommandResponse::Empty
                    }
                }
            }).await;
        }
    }, task_queue).unwrap().detach()
}

