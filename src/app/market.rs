use axum::{
    debug_handler,
    extract::{Path, Query, State}, 
    response::{IntoResponse, Response}
};

pub async fn get_market() -> impl IntoResponse {
    "get_market" 
}
