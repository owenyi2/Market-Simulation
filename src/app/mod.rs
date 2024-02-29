use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

pub mod account;
pub mod market;
pub mod order;

use market_simulation::{account::AccountId, market::Market};

type MarketStateHandle = Arc<Mutex<Market>>;

pub async fn app_main() {
    println!("Hello app");

    let market = MarketStateHandle::default();

    let api_route = Router::new()
        .route("/api/account/new", post(account::new_account))
        .route("/api/account", get(account::get_account))
        .route(
            "/api/order/:id",
            get(order::get_order_by_id).delete(order::delete_order_by_id),
        )
        .route("/api/order/new", post(order::new_order))
        .route("/api/order", get(order::get_all_orders))
        .with_state(market)
        .fallback(fallback);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, api_route).await.unwrap();
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

pub enum AppError {
    AccountIdMissing,
    AccountIdInvalid,
    AccountDoesNotExist,
    OrderBodyIncorrect,
    OrderInvalid(&'static str),
    OrderIdInvalid,
    OrderDoesNotExist,
    OrderCannotBeCancelled,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::AccountIdMissing => {
                (StatusCode::BAD_REQUEST, "`account-id` missing in Header")
            }
            AppError::AccountIdInvalid => (StatusCode::BAD_REQUEST, "`account-id` is invalid"),
            AppError::AccountDoesNotExist => {
                (StatusCode::FORBIDDEN, "this `account-id` does not exist")
            }
            AppError::OrderBodyIncorrect => {
                (StatusCode::BAD_REQUEST, "submitted order Body is incorrect")
            }
            AppError::OrderIdInvalid => (StatusCode::NOT_FOUND, "the order `id` is invalid"),
            AppError::OrderInvalid(e) => 
                (StatusCode::BAD_REQUEST, e),
            AppError::OrderDoesNotExist => (
                StatusCode::NOT_FOUND,
                "this order `id` does not exist or no longer exists",
            ),
            AppError::OrderCannotBeCancelled => {
                (StatusCode::GONE, "this order can no longer be cancelled")
            }
        };
        return (status, message).into_response();
    }
}

fn parse_account_id_from_header(headers: HeaderMap) -> Result<Uuid, AppError> {
    let account_id = headers
        .get("account-id")
        .ok_or(AppError::AccountIdMissing)?
        .to_str()
        .map_err(|_| AppError::AccountIdInvalid)?;
    let account_id = Uuid::try_parse(account_id).map_err(|_| AppError::AccountIdInvalid)?;

    Ok(account_id)
}
