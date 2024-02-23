use std::net::SocketAddr;
use std::sync::Arc;
use std::process;
use std::thread;
use std::time;

use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router
};
use serde::{Deserialize, Serialize};
use tokio::{sync::{Mutex, RwLock}, runtime::Builder};

mod controllers;
use controllers::{account,market,order};


fn main() {
    let logic_handler = thread::spawn(|| {
        logic_main()
    }); 

    let api_runtime = Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();
        
    api_runtime.block_on(
        app_main()
    )
}

fn logic_main() {
    loop {
        let ten_secs = time::Duration::from_secs(10); 
        println!("sleeping");
        thread::sleep(ten_secs);
    }
}

async fn app_main() {
    println!("Hello app");

    let api_route = Router::new()
        .route("/api/account", get(account::get_account).post(account::post_account))
        .route("/api/market/", get(market::get_market)) 
        .route("/api/order", get(order::get_order).post(order::post_order).delete(order::delete_order))
        .fallback(fallback);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000"
            ).await.unwrap();
    axum::serve(listener, api_route).await.unwrap();
    
        
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

