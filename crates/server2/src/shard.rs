use std::net::SocketAddr;

use futures::{select, FutureExt};
use glommio::{channels::local_channel, ExecutorJoinHandle, LocalExecutorBuilder};
use tracing::info;

use crate::{acceptor::Acceptor, processor::Processor};

pub struct Shard {
    addr: SocketAddr,
    name: String,
}

impl Shard {
    pub fn new(addr: SocketAddr, name: String) -> Self {
        Self { addr, name }
    }

    pub fn start(self) -> ExecutorJoinHandle<Self> {
        LocalExecutorBuilder::default()
            .name(&self.name)
            .spawn(async move || {
                let (sender, receiver) = local_channel::new_bounded(32);
                let acceptor = Acceptor::new(self.addr);
                let processor = Processor::new();

                select! {
                    _ = acceptor.start(sender).fuse() => {
                        info!("acceptor exited");
                    },
                    _ = processor.start(receiver).fuse() => {
                        info!("processor exited");
                    },
                };

                self
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
        let handles = self.inner.into_iter().map(Shard::start).collect();
        ShardsHandle { inner: handles }
    }
}

pub struct ShardsHandle {
    inner: Box<[ExecutorJoinHandle<Shard>]>
}

impl ShardsHandle {
    pub fn join_all(self) {
        self.inner.into_iter().for_each(|handle| { handle.join().unwrap(); });
    }
}
