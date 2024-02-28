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
use super::{parse_account_id_from_header, AccountIdError};

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

pub async fn get_account(State(market): State<Arc<market::Market>>, headers: HeaderMap) -> Result<Response, AccountIdError> { 
    let account_id = parse_account_id_from_header(market.clone(), headers)?;

    let accounts = market.accounts.lock().unwrap();
    let account = accounts.get(&account_id);
    Ok(Json(account.view()).into_response())
}
