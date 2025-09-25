use std::{net::SocketAddr, num::NonZeroUsize, str::FromStr, thread::available_parallelism};

use goosekv_server::shard::{ShardBuilder, Shards};

fn main() {
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .init();

    let thread_count = available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
    let addr = SocketAddr::from_str("127.0.0.1:6379").unwrap();

    let shard_builder = ShardBuilder::new(addr);
    let shards = Shards::from_builder(shard_builder, thread_count.get(), "SHARD".to_string());

    shards.start().join_all();
}
