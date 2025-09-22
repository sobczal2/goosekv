use std::{
    net::SocketAddr,
    num::NonZeroUsize,
    str::FromStr,
    thread::available_parallelism,
};

use glommio::channels::channel_mesh::MeshBuilder;
use goosekv_server::shard;

fn main() {
    tracing_subscriber::fmt().with_thread_ids(true).with_thread_names(true).init();

    let thread_count = available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
    let addr = SocketAddr::from_str("127.0.0.1:6379").unwrap();

    let mesh = MeshBuilder::full(thread_count.get(), 256);

    let mut handles = Vec::new();
    for _ in 0..thread_count.get() {
        handles.push(shard::thread::Thread::new(mesh.clone(), addr).start());
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
