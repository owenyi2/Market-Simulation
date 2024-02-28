use std::net::SocketAddr;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time;

use tokio::runtime::Builder;

mod app;

fn main() {
    let api_runtime = Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();

    api_runtime.block_on(app::app_main())
}
