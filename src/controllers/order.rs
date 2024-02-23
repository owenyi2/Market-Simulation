use axum::{
    debug_handler,
    extract::{Path, Query, State}, 
    response::{IntoResponse, Response}
};

pub async fn get_order() -> impl IntoResponse {
    "get_order" 
}
pub async fn post_order() -> impl IntoResponse {
    "post_order"
}
pub async fn delete_order() -> impl IntoResponse {
    "delete_order"
}
