use std::{
    net::SocketAddr,
    num::NonZeroUsize,
    str::FromStr,
    thread::available_parallelism,
};

use goosekv_server::{router::Router, shard};

fn main() {
    tracing_subscriber::fmt().with_thread_ids(true).with_thread_names(true).init();

    let thread_count = available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
    let addr = SocketAddr::from_str("127.0.0.1:6379").unwrap();

    let router = Router::new(thread_count.get());

    let mut handles = Vec::new();
    for _ in 0..thread_count.get() {
        handles.push(shard::thread::Thread::new(addr).start(router.clone()));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
