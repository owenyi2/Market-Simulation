use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::{StatusCode, header::HeaderMap},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod order;
pub mod account;
pub mod market;

use market_simulation::{market::Market, account::AccountId};

pub async fn app_main() {
    println!("Hello app");

    let market = Arc::new(Market::default());

    let api_route = Router::new()
        .route("/api/account/new", post(account::new_account))
        .route("/api/account", get(account::get_account))
        .route("/api/order/:id", get(order::get_order_by_id).delete(order::delete_order_by_id))
        .route("/api/order/new", post(order::new_order))
        .route("/api/order", get(order::get_all_orders))
        .with_state(market)
        .fallback(fallback);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000"
            ).await.unwrap();
    axum::serve(listener, api_route).await.unwrap();        
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

pub enum AccountIdError {
    Missing,
    Invalid,
    AccountDoesNotExist,
}

impl IntoResponse for AccountIdError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AccountIdError::Missing => (StatusCode::BAD_REQUEST, "`account-id` missing in Header"),
            AccountIdError::Invalid => (StatusCode::BAD_REQUEST, "`account-id` is invalid"),
            AccountIdError::AccountDoesNotExist => (StatusCode::FORBIDDEN, "this `account-id` does not exist")
        };
        return (status, message).into_response() 
    }
}

fn parse_account_id_from_header(market: Arc<Market>, headers: HeaderMap) -> Result<AccountId, AccountIdError> { 
    let account_id = headers.get("account-id")
        .ok_or(AccountIdError::Missing)?
        .to_str().map_err(|e| AccountIdError::Invalid)?;
    let account_id = Uuid::try_parse(account_id).map_err(|e| AccountIdError::Invalid)?;

    let accounts = market.accounts.lock().unwrap();
    let account_id = accounts.check_uuid(account_id).ok_or(AccountIdError::AccountDoesNotExist)?;
    Ok(account_id)
}
