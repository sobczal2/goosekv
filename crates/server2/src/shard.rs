use std::net::SocketAddr;

use futures::{select, FutureExt};
use glommio::{ExecutorJoinHandle, LocalExecutorBuilder};
use tracing::info;

use crate::{acceptor::actor::AcceptorActor, processor::actor::ProcessorActor, storage::{actor::StorageActor, router::StorageRouter}};

pub struct Shard {
    name: String,
    acceptor: AcceptorActor,
    processor: ProcessorActor,
    storage: StorageActor,
}

impl Shard {
    pub fn new(addr: SocketAddr, name: String) -> Self {
        Self { name, acceptor: AcceptorActor::new(addr), processor: ProcessorActor::new(), storage: StorageActor::new() }
    }

    pub fn start(self, storage: StorageRouter) -> ExecutorJoinHandle<()> {
        LocalExecutorBuilder::default()
            .name(&self.name)
            .spawn(async move || {
                let storage_task = self.storage.run();
                let (processor_task, processor_handle) = self.processor.run(storage);
                let acceptor_task = self.acceptor.run(processor_handle);

                select! {
                    _ = acceptor_task.fuse() => {
                        info!("acceptor exited");
                    },
                    _ = processor_task.fuse() => {
                        info!("processor exited");
                    },
                    _ = storage_task.fuse() => {
                        info!("storage exited");
                    },
                };
            })
            .unwrap()
    }
}

pub struct ShardBuilder {
    addr: SocketAddr,
}

impl ShardBuilder {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub fn build(&self, name: String) -> Shard {
        Shard::new(self.addr, name)
    }
}

pub struct Shards {
    inner: Box<[Shard]>,
}

impl Shards {
    pub fn from_builder(builder: ShardBuilder, count: usize, name: String) -> Shards {
        let shards = (0..count).map(|_| builder.build(name.clone())).collect();
        Shards { inner: shards }
    }

    pub fn start(self) -> ShardsHandle {
        let storage_handles = self.inner.iter().map(|shard| shard.storage.handle()).collect();
        let router = StorageRouter::new(storage_handles);

        let handles = self.inner.into_iter().map(|shard| {
            shard.start(router.clone())
        }).collect();

        ShardsHandle { inner: handles }
    }
}

pub struct ShardsHandle {
    inner: Box<[ExecutorJoinHandle<()>]>
}

impl ShardsHandle {
    pub fn join_all(self) {
        self.inner.into_iter().for_each(|handle| { handle.join().unwrap(); });
    }
}
