use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::json;
use std::{fmt, str::FromStr};

#[derive(Debug, Serialize)]
pub struct SuccessResponse<T> {
    code: i8,
    data: T,
    message: String,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T, message: &str) -> Self {
        Self {
            code: 0,
            data,
            message: message.to_string(),
        }
    }

    pub fn new_with_code(code: i8, data: T, message: &str) -> Self {
        Self {
            code,
            data,
            message: message.to_string(),
        }
    }
}

impl<T> IntoResponse for SuccessResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let body = Json(json!(self));
        (StatusCode::OK, body).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct FailureResponse {
    code: i8,
    message: String,
}

impl FailureResponse {
    pub fn new(message: &str) -> Self {
        Self {
            code: -1,
            message: message.to_string(),
        }
    }
}

pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationRequest<T> {
    pub page: i32,
    pub page_size: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetEntryResponse<T> {
    pub entries: Vec<T>,
    pub entries_per_page: i32,
    pub total_entries: i32,
}
