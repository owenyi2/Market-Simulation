use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{parse_account_id_from_header, AppError, MarketStateHandle};
use market_simulation::{account::AccountId, market};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountReqBody {
    account_balance: f64,
    position: i32,
}

#[debug_handler]
pub async fn new_account(
    State(market): State<MarketStateHandle>,
    Json(account_req_body): Json<AccountReqBody>,
) -> impl IntoResponse {
    let Ok(account_id) = market
        .lock()
        .await
        .new_account(account_req_body.account_balance, account_req_body.position)
    else {
        return (
            StatusCode::BAD_REQUEST,
            "field `account_balance` in Body is invalid",
        )
            .into_response();
    };
    account_id.as_uuid().to_string().into_response()
}

pub async fn get_account(
    State(market): State<MarketStateHandle>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let account_id = parse_account_id_from_header(headers)?;

    let market = market.lock().await;
    let account_id = market
        .check_uuid(account_id)
        .ok_or(AppError::AccountDoesNotExist)?;

    let account = market.get_account(&account_id);
    Ok(Json(account.view()).into_response())
}
