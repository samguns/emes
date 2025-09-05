use crate::api::utils::{FailureResponse, SuccessResponse};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::player::{PlayerStatus, Track};

#[derive(Debug, Deserialize)]
pub struct PlayRequest {
    pub playlist: Vec<Track>,
    pub selected_index: usize,
}

pub async fn play(
    state: State<Arc<AppState>>,
    Json(req): Json<PlayRequest>,
) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();

    if let Err(e) = player.play(&req.playlist, req.selected_index) {
        tracing::error!("Failed to play track: {}", e);
        return Err(PlayError::InternalError);
    }

    Ok(SuccessResponse::new((), "Success"))
}

pub async fn stop(state: State<Arc<AppState>>) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();
    if let Err(e) = player.stop() {
        tracing::error!("Failed to stop track: {}", e);
        return Err(PlayError::InternalError);
    }

    Ok(SuccessResponse::new((), "Success"))
}

pub async fn toggle(state: State<Arc<AppState>>) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();
    if let Err(e) = player.toggle() {
        tracing::error!("Failed to toggle track: {}", e);
        return Err(PlayError::InternalError);
    }

    Ok(SuccessResponse::new((), "Success"))
}

pub async fn status(
    state: State<Arc<AppState>>,
) -> Result<SuccessResponse<PlayerStatus>, PlayError> {
    let player = state.player_state.get_music_player();
    match player.status() {
        Ok(status) => Ok(SuccessResponse::new(status, "Success")),
        Err(e) => {
            tracing::error!("Failed to get status: {}", e);
            Err(PlayError::InternalError)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetVolumeRequest {
    pub volume: f32,
}

pub async fn set_volume(
    state: State<Arc<AppState>>,
    Json(req): Json<SetVolumeRequest>,
) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();
    if let Err(e) = player.set_volume(req.volume) {
        tracing::error!("Failed to set volume: {}", e);
        return Err(PlayError::InternalError);
    }
    Ok(SuccessResponse::new((), "Success"))
}

#[derive(Debug, Deserialize)]
pub struct SeekRequest {
    pub delta: f32,
}

pub async fn seek(
    state: State<Arc<AppState>>,
    Json(req): Json<SeekRequest>,
) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();
    if let Err(e) = player.seek(req.delta) {
        tracing::error!("Failed to seek: {}", e);
        return Err(PlayError::InternalError);
    }
    Ok(SuccessResponse::new((), "Success"))
}

#[derive(Debug, Deserialize)]
pub struct SeekToRequest {
    pub seconds: f32,
}

pub async fn seek_to(
    state: State<Arc<AppState>>,
    Json(req): Json<SeekToRequest>,
) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();
    if let Err(e) = player.seek_to(req.seconds) {
        tracing::error!("Failed to seek to: {}", e);
        return Err(PlayError::InternalError);
    }
    Ok(SuccessResponse::new((), "Success"))
}

pub async fn next(state: State<Arc<AppState>>) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();
    if let Err(e) = player.next() {
        tracing::error!("Failed to next: {}", e);
        return Err(PlayError::InternalError);
    }
    Ok(SuccessResponse::new((), "Success"))
}

pub async fn prev(state: State<Arc<AppState>>) -> Result<SuccessResponse<()>, PlayError> {
    let player = state.player_state.get_music_player();
    if let Err(e) = player.prev() {
        tracing::error!("Failed to prev: {}", e);
        return Err(PlayError::InternalError);
    }
    Ok(SuccessResponse::new((), "Success"))
}

pub enum PlayError {
    InternalError,
    DatabaseError,
}

impl IntoResponse for PlayError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            PlayError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to play track"),
            PlayError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        let res = FailureResponse::new(error_msg);
        let body = Json(json!(res));
        (status, body).into_response()
    }
}
