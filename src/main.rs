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
    let api_runtime = Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();
        
    api_runtime.block_on(
        app_main()
    )
}

async fn app_main() {
    println!("Hello app");


    let api_route = Router::new()
        .route("/api/account/new", post(account::new_account))
        .route("/api/account", get(account::get_account))
        .route("/api/order/:id", get(order::get_order_by_id).delete(order::delete_order_by_id))
        .route("/api/order/new", post(order::new_order))
        .route("/api/order", get(order::get_all_orders))
        .fallback(fallback);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000"
            ).await.unwrap();
    axum::serve(listener, api_route).await.unwrap();        
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

