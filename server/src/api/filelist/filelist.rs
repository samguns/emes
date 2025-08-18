use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde_json::json;
use std::sync::Arc;

use crate::api::utils::{FailureResponse, GetEntryResponse, PaginationRequest, SuccessResponse};
use crate::app_state::AppState;
use crate::dao::file_dao;

pub async fn get_file_list(
    state: State<Arc<AppState>>,
    Json(req): Json<PaginationRequest<file_dao::FileEntryFilter>>,
) -> Result<SuccessResponse<GetEntryResponse<file_dao::FileEntry>>, GetFileListError> {
    let file_dao = file_dao::FileDao::new(&state.db_state).await;
    let get_result = file_dao.get_files(&req).await;
    if get_result.is_err() {
        return Err(GetFileListError::DatabaseError);
    }

    let (files, count) = get_result.unwrap();
    Ok(SuccessResponse::new(
        GetEntryResponse {
            entries: files,
            entries_per_page: req.page_size,
            total_entries: count as i32,
        },
        "Success",
    ))
}

pub enum GetFileListError {
    DatabaseError,
}

impl IntoResponse for GetFileListError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            GetFileListError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query file list from database",
            ),
        };

        let res = FailureResponse::new(error_msg);
        let body = Json(json!(res));
        (status, body).into_response()
    }
}
