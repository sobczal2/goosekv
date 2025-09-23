use std::{
    iter::repeat_n,
    net::SocketAddr,
    num::NonZeroUsize,
    str::FromStr,
    thread::available_parallelism,
};

use glommio::channels::channel_mesh::MeshBuilder;
use goosekv_server::{
    executor,
    io,
    worker,
};
use tracing::Level;

fn main() {
    tracing_subscriber::fmt().with_thread_ids(true).with_max_level(Level::ERROR).with_thread_names(true).init();

    let thread_count = available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
    let addr = SocketAddr::from_str("127.0.0.1:6379").unwrap();

    let io_thread = io::thread::Thread::new(addr);
    let executor_thread = executor::thread::Thread::new();

    let mesh = MeshBuilder::partial(1 + thread_count.get(), 256);
    let (io_handle, receiver) = io_thread.start();
    let executor_handle = executor_thread.start(mesh.clone(), receiver);

    let worker_handles = repeat_n((), thread_count.get())
        .map(|_| worker::thread::Thread::new().start(mesh.clone()))
        .collect::<Vec<_>>();

    for worker_handle in worker_handles {
        worker_handle.join().unwrap();
    }
    executor_handle.join().unwrap();
    io_handle.join().unwrap();
}
