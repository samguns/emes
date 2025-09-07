use crate::api::utils::{FailureResponse, SuccessResponse};
use crate::ws2812::SetLedStripStatusEvent;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::dao::player_led_dao;

pub async fn get_led_strip_status(
    state: State<Arc<AppState>>,
) -> Result<SuccessResponse<player_led_dao::PlayerLedEntry>, LedStripError> {
    let player_led_dao = player_led_dao::PlayerLedDao::new(&state.db_state).await;
    let led_strip = player_led_dao.get_led_strip_status().await;
    if led_strip.is_err() {
        return Err(LedStripError::DatabaseError);
    }

    let led_strip = led_strip.unwrap();
    Ok(SuccessResponse::new(led_strip, "Success"))
}

pub async fn set_led_strip_status(
    state: State<Arc<AppState>>,
    Json(req): Json<player_led_dao::PlayerLedEntry>,
) -> Result<SuccessResponse<()>, LedStripError> {
    let player_led_dao = player_led_dao::PlayerLedDao::new(&state.db_state).await;
    let led_strip = player_led_dao.set_led_strip_status(req).await;
    if led_strip.is_err() {
        return Err(LedStripError::DatabaseError);
    }

    let event_chan_sender = state.led_strip_state.get_event_chan_sender();
    let event_str = json!(SetLedStripStatusEvent {
        enable: true,
        status: Some(req.clone()),
    })
    .to_string();
    let _ = event_chan_sender.send(event_str);

    Ok(SuccessResponse::new((), "Success"))
}

pub enum LedStripError {
    DatabaseError,
}

impl IntoResponse for LedStripError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            LedStripError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
        };

        let res = FailureResponse::new(error_msg);
        let body = Json(json!(res));
        (status, body).into_response()
    }
}
