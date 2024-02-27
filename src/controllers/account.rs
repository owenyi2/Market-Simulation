use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Path, Query, State, Json}, 
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result}
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use market_simulation::{market, account::AccountId};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountReqBody {
    account_balance: f64,
    position: i32,
}

#[debug_handler]
pub async fn new_account(State(market): State<Arc<market::Market>>, Json(account_req_body): Json<AccountReqBody>) -> impl IntoResponse {
    let Ok(account_id) = market.new_account(account_req_body.account_balance, account_req_body.position) else {
        return (StatusCode::BAD_REQUEST, "field `account_balance` in Body is invalid").into_response()
    };
    account_id.as_uuid().to_string().into_response()
}

pub async fn get_account(State(market): State<Arc<market::Market>>, headers: HeaderMap) -> impl IntoResponse { 
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in Header").into_response()
    };
    let Ok(account_id) = account_id.to_str() else {
        return (StatusCode::BAD_REQUEST, "`account-id` is invalid").into_response()
    };
    let Ok(account_id) = Uuid::try_parse(account_id) else {
        return (StatusCode::BAD_REQUEST, "`account-id` is invalid").into_response()
    };
    
    let accounts = market.accounts.lock().unwrap();
    let Some(account_id) = accounts.check_uuid(account_id) else {
        return (StatusCode::FORBIDDEN, "this `account-id` doesn't exist").into_response()
    };

    let account = accounts.get(&account_id);
    Json(account.view()).into_response() 
}
