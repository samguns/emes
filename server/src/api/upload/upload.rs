use axum::body::Bytes;
use axum::extract::Multipart;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use chrono::{Datelike, Timelike};
use std::io;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

use crate::api::utils::{FailureResponse, SuccessResponse};
use crate::app_state::AppState;
use crate::dao::file_dao;

pub async fn upload_file(
    state: State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<SuccessResponse<u64>, UploadError> {
    let mut class = None;
    let mut file_name = None;
    let mut file_bytes = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");

        match name {
            "file" => {
                let filename = field.file_name().unwrap_or("");
                file_name = Some(filename.to_string());
                let data = field.bytes().await;
                if let Ok(bytes) = data {
                    file_bytes = Some(bytes);
                }
            }
            "class" => {
                let data = field.text().await;
                if let Ok(text) = data {
                    class = Some(text);
                }
            }
            _ => {}
        }
    }

    // 检查是否已获取到标签和文件内容
    if let (Some(class_val), Some(file_name_val), Some(file_val)) =
        (&class, &file_name, &file_bytes)
    {
        // tracing::info!("class: {}, file_name: {}", class_val, file_name_val);
        let file_dao = file_dao::FileDao::new(&state.db_state).await;
        let file_entry = file_dao.get_file_by_name(file_name_val).await;
        if file_entry.is_some() {
            return Err(UploadError::FileAlreadyExists);
        }
        if let Ok(size) =
            process_upload_stream(&file_dao, &class_val, &file_name_val, &file_val).await
        {
            return Ok(SuccessResponse::new(size, "Uploaded"));
        }
    }

    Err(UploadError::UploadFailed)
}

async fn process_upload_stream(
    file_dao: &file_dao::FileDao,
    class: &str,
    filename: &str,
    file_bytes: &[u8],
) -> Result<u64, std::io::Error> {
    let now = chrono::Utc::now();
    let timestamp = now.timestamp_millis();

    let year = now.year().to_string();
    let month = now.month().to_string();
    let day = now.day().to_string();
    let hour = now.hour().to_string();

    let processor = async {
        let body_reader = StreamReader::new(futures::stream::once(async move {
            Ok::<_, io::Error>(Bytes::copy_from_slice(file_bytes))
        }));
        futures::pin_mut!(body_reader);

        let file_dir = std::path::PathBuf::from(year)
            .join(month)
            .join(day)
            .join(hour);
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
                    class: class.parse().unwrap(),
                    is_training_data: Some(false),
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

        let failure_response = FailureResponse::new(&error_msg);
        (status, axum::Json(failure_response)).into_response()
    }
}
