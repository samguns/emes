use axum::BoxError;
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use chrono::{Datelike, Timelike};
use futures::{Stream, TryStreamExt};
use std::io;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

use crate::api::utils::SuccessResponse;
use crate::app_state::AppState;
use crate::dao::file_dao;

pub async fn upload_file(
    state: State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<SuccessResponse<u64>, UploadError> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let filename_opt = field.file_name();
        if filename_opt.is_none() {
            continue;
        }
        let filename = filename_opt.unwrap().trim().to_string();
        if filename.is_empty() {
            continue;
        }

        let file_dao = file_dao::FileDao::new(&state.db_state).await;
        let file_entry = file_dao.get_file_by_name(&filename).await;
        if file_entry.is_some() {
            return Err(UploadError::FileAlreadyExists);
        }

        if let Ok(size) = process_upload_stream(&file_dao, &filename, field).await {
            return Ok(SuccessResponse::new(size, "Uploaded"));
        }

        return Err(UploadError::UploadFailed);
    }

    Ok(SuccessResponse::new(0, "Uploaded"))
}

async fn process_upload_stream<S, E>(
    file_dao: &file_dao::FileDao,
    filename: &str,
    field: S,
) -> Result<u64, std::io::Error>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let now = chrono::Utc::now();
    let timestamp = now.timestamp_millis();

    let year = now.year().to_string();
    let month = now.month().to_string();
    let day = now.day().to_string();
    let hour = now.hour().to_string();

    let processor = async {
        let body_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        let file_dir = format!("{}/{}/{}/{}", year, month, day, hour);
        let file_path = std::path::Path::new(&file_dir);
        if !file_path.exists() {
            tokio::fs::create_dir_all(file_path).await.unwrap();
        }

        let file_path = file_path.join(filename);
        let file_path_str = file_path.to_string_lossy().to_string();
        let mut file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)
                .await
                .unwrap(),
        );

        let copied = tokio::io::copy(&mut body_reader, &mut file).await;
        let res = match copied {
            Ok(n) => {
                let file_entry = file_dao::FileEntry {
                    id: None,
                    name: filename.to_string(),
                    size: n as f64,
                    path: file_path_str,
                    label: "".to_string(),
                    created_at: timestamp as f64,
                };
                if let Err(e) = file_dao.insert_file(file_entry).await {
                    tracing::error!("Failed to insert file: {}", e);
                    return Err(io::Error::new(io::ErrorKind::Other, e));
                }

                Ok::<u64, io::Error>(n)
            }
            Err(e) => {
                tracing::error!("Failed to copy file: {}", e);
                return Err(e.into());
            }
        };

        res
    };

    match processor.await {
        Ok(copied) => Ok(copied),
        Err(e) => Err(e),
    }
}

pub enum UploadError {
    UploadFailed,
    FileAlreadyExists,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            UploadError::UploadFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Upload failed".to_string(),
            ),
            UploadError::FileAlreadyExists => {
                (StatusCode::CONFLICT, "File already exists".to_string())
            }
        };
        (status, error_msg).into_response()
    }
}
