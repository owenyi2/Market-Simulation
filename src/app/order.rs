use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use market_simulation::order;
use super::{parse_account_id_from_header, AppError, MarketStateHandle};

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderReqBody {
    limit: f64,
    quantity: usize,
    side: order::Side,
}

pub async fn get_order_by_id(
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> impl IntoResponse {
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response();
    }; // This should be middleware. Consider refactoring later. I think using route_layer
    println!("{:?}", account_id);

    "get_order_by_id".into_response()
}
pub async fn get_all_orders(headers: HeaderMap) -> impl IntoResponse {
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response();
    };
    println!("{:?}", account_id);

    "get_all_orders".into_response()
}
pub async fn new_order(
    headers: HeaderMap,
    State(market): State<MarketStateHandle>,
    Json(order_req_body): Json<OrderReqBody>,
) -> Result<Response, AppError> {
    let account_id = parse_account_id_from_header(headers)?;

    let mut market = market.lock().await;
    let account_id = market
        .check_uuid(account_id)
        .ok_or(AppError::AccountDoesNotExist)?;

    //TODO: Implement market.validate_order()
    let order = order::OrderBase::build(order_req_body.limit, order_req_body.quantity, order_req_body.side, account_id)
        .map_err(|e| AppError::OrderInvalid)?;

    let order_view = order.view();
    market.handle_incoming_order(order); 

    Ok(Json(order_view).into_response()) 
}
pub async fn delete_order_by_id(
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> impl IntoResponse {
    let Some(account_id) = headers.get("account-id") else {
        return (StatusCode::BAD_REQUEST, "`account-id` missing in header").into_response();
    };
    println!("{:?}", account_id);
    "delete_order_by_id".into_response()
}
