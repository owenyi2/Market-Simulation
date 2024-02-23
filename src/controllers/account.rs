use axum::{
    debug_handler,
    extract::{Path, Query, State}, 
    response::{IntoResponse, Response}
};

pub async fn get_account() -> impl IntoResponse {
    "get_account" 
}
pub async fn post_account() -> impl IntoResponse {
    "post_account"
}
