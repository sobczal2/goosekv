use std::{net::SocketAddr, num::NonZeroUsize, str::FromStr, thread::available_parallelism};

use glommio::{channels::{channel_mesh::{MeshBuilder, Role}, sharding::{Handler, HandlerResult, Sharded}}, enclose, spawn_local, LocalExecutorBuilder};
use goosekv_protocol::frame::Frame;
use goosekv_server::{context::Context, io};




fn main() {
    let thread_count = available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
    let addr = SocketAddr::from_str("127.0.0.1:6979").unwrap();

    let mesh = MeshBuilder::partial(1 + thread_count.get(), 256);

    let io_thread = io::thread::Thread::new(addr, mesh.clone());
    let handle = io_thread.start();

    let handles = (0..thread_count.get()).map(|_| {
        LocalExecutorBuilder::default().name("shard-{i}").spawn(enclose!((mesh) move || async move {
            let (_, receiver) = mesh.join(Role::Consumer).await.unwrap();

            while let Some(context) = receiver.recv_from(0).await.unwrap() {
                context.respond([Frame::SimpleString("OK".to_string())]).await.unwrap();
            }
        })).unwrap()
    }).collect::<Vec<_>>();

    handle.join().unwrap();
    for handle in handles {
        handle.join().unwrap();
    }
}
