use axum::{
    debug_handler,
    extract::{Path, Query, State, Json}, 
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result}
};
use serde::{Deserialize, Serialize};

use market_simulation::market;

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountRequestBody {
    account_balance: f64,
    position: i32,
}

pub async fn new_account(Json(account): Json<AccountRequestBody>) -> Result<&'static str> {
    println!("{:?}", account); 
    Ok("new_account")
}

pub async fn get_account(headers: HeaderMap) -> impl IntoResponse { 
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response()
    };
    println!("{:?}", account_id);
    "get_account".into_response()
}
