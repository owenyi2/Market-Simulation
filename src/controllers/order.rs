use axum::{
    debug_handler,
    extract::{Path, Query, State, Json}, 
    http::{header::HeaderMap, StatusCode}, 
    response::{IntoResponse, Response}
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderRequestBody {

}

pub async fn get_order_by_id(headers: HeaderMap, Path(order_id): Path<String>) -> impl IntoResponse { 
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response()
    }; // This should be middleware. Consider refactoring later. I think using route_layer
    println!("{:?}", account_id);

    "get_order_by_id".into_response()
}
pub async fn get_all_orders(headers: HeaderMap) -> impl IntoResponse {
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response()
    };
    println!("{:?}", account_id);
    
    "get_all_orders".into_response()
}
pub async fn new_order(headers: HeaderMap, Json(order): Json<OrderRequestBody>) -> impl IntoResponse {
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response()
    };
    println!("{:?}", account_id);

    "post_order".into_response()
}
pub async fn delete_order_by_id(headers: HeaderMap, Path(order_id): Path<String>) -> impl IntoResponse { 
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response()
    };
    println!("{:?}", account_id);
    "delete_order_by_id".into_response()
}
