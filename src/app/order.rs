use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{parse_account_id_from_header, AppError, MarketStateHandle};
use market_simulation::order;

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderReqBody {
    limit: f64,
    quantity: usize,
    side: order::Side,
}

pub async fn get_order_by_id(
    headers: HeaderMap,
    State(market): State<MarketStateHandle>,
    Path(order_id): Path<String>,
) -> Result<Response, AppError> {
    let order_id = Uuid::try_parse(&order_id).map_err(|_| AppError::OrderIdInvalid)?;

    let account_id = parse_account_id_from_header(headers)?;

    let mut market = market.lock().await;
    let account_id = market
        .check_account_uuid(account_id)
        .ok_or(AppError::AccountDoesNotExist)?;

    let order = market
        .get_order_by_id(order_id)
        .ok_or(AppError::OrderDoesNotExist)?;

    Ok(Json(order.view()).into_response())
}
pub async fn get_all_orders(
    headers: HeaderMap,
    State(market): State<MarketStateHandle>,
) -> Result<Response, AppError> {
    let account_id = parse_account_id_from_header(headers)?;

    let mut market = market.lock().await;
    let account_id = market
        .check_account_uuid(account_id)
        .ok_or(AppError::AccountDoesNotExist)?;

    let orders = market.get_orders_by_account(account_id);

    Ok(Json(orders.map(|order| order.view()).collect::<Vec<_>>()).into_response())
}
pub async fn new_order(
    headers: HeaderMap,
    State(market): State<MarketStateHandle>,
    Json(order_req_body): Json<OrderReqBody>,
) -> Result<Response, AppError> {
    let account_id = parse_account_id_from_header(headers)?;

    let mut market = market.lock().await;
    let account_id = market
        .check_account_uuid(account_id)
        .ok_or(AppError::AccountDoesNotExist)?;

    let order = order::OrderBase::build(
        order_req_body.limit,
        order_req_body.quantity,
        order_req_body.side,
        account_id,
    )
    .map_err(|_| AppError::OrderBodyIncorrect)?;

    market.validate_order(&order, account_id).map_err(|e| AppError::OrderInvalid(e))?;

    let order_view = order.view();
    market.handle_incoming_order(order);

    Ok(Json(order_view).into_response())
}

pub async fn delete_order_by_id(
    headers: HeaderMap,
    State(market): State<MarketStateHandle>,
    Path(order_id): Path<String>,
) -> Result<Response, AppError> {
    let order_id = Uuid::try_parse(&order_id).map_err(|_| AppError::OrderIdInvalid)?;

    let account_id = parse_account_id_from_header(headers)?;

    let mut market = market.lock().await;
    let account_id = market
        .check_account_uuid(account_id)
        .ok_or(AppError::AccountDoesNotExist)?;

    market
        .delete_order_by_id(order_id)
        .ok_or(AppError::OrderCannotBeCancelled)?;

    Ok("".into_response())
}
