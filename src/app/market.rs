use axum::{
    debug_handler,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json
};

use super::{AppError, MarketStateHandle};

pub async fn bars() -> Result<Response, AppError> {
    todo!()
}

pub async fn quote(
    State(market): State<MarketStateHandle>
        ) -> impl IntoResponse {

    let market = market.lock().await;
    let (ask, bid) = market.quote();

    let ask = match ask {
        Some(ask) => Some(ask.view()),
        None => None
    }; 
    let bid = match bid {
        Some(bid) => Some(bid.view()),
        None => None
    }; 

    Json((ask, bid)).into_response()
}
