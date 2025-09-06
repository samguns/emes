use serde::Deserialize;
use serde::Serialize;
use sqlx::Acquire as _;
use sqlx::Row;

use crate::api::utils::PaginationRequest;
use crate::dao::db_state::DBClientState;

pub struct FileDao {
    db_client_state: DBClientState,
}

impl FileDao {
    pub async fn new(db_client_state: &DBClientState) -> Self {
        let file_dao = FileDao {
            db_client_state: db_client_state.clone(),
        };

        file_dao.init().await;

        file_dao
    }

    pub async fn get_file_by_name(&self, name: &str) -> Option<FileEntry> {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let file_query = sqlx::query("SELECT * FROM file WHERE name = ?")
            .bind(name)
            .fetch_one(&mut *tx)
            .await;
        if let Err(e) = file_query {
            // tracing::error!("Failed to query file by name: {}", e);
            return None;
        }

        let file_query = file_query.unwrap();

        Some(FileEntry {
            id: file_query.get("id"),
            name: file_query.get("name"),
            size: file_query.get("size"),
            path: file_query.get("path"),
            class: file_query.get("class"),
            is_training_data: file_query.get("is_training_data"),
            created_at: file_query.get("created_at"),
        })
    }

    pub async fn insert_file(&self, file_entry: FileEntry) -> Result<(), sqlx::Error> {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let insert_query = sqlx::query(
            "INSERT INTO file (name, size, path, class, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(file_entry.name)
        .bind(file_entry.size)
        .bind(file_entry.path)
        .bind(file_entry.class)
        .bind(file_entry.created_at);

        let insert_query = insert_query.execute(&mut *tx).await;
        if let Err(e) = insert_query {
            tracing::error!("Failed to insert file: {}", e);
            return Err(e);
        }

        if let Err(e) = tx.commit().await {
            tracing::error!("Failed to commit transaction: {}", e);
            return Err(e);
        }

        Ok(())
    }

    pub async fn get_files(
        &self,
        request: &PaginationRequest<FileEntryFilter>,
    ) -> Result<(Vec<FileEntry>, i64), sqlx::Error> {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let mut query_str = String::from("SELECT * FROM file");
        let mut query_count_str = String::from("SELECT COUNT(*) FROM file");
        let mut conditions = Vec::new();
        match &request.condition {
            Some(filter) => {
                if let Some(name) = &filter.name {
                    conditions.push(format!("name = '{}'", name));
                }

                if let Some(class) = &filter.class {
                    conditions.push(format!("class = '{}'", class));
                }

                if let Some(is_training_data) = &filter.is_training_data {
                    conditions.push(format!("is_training_data = {}", is_training_data));
                }

                if !conditions.is_empty() {
                    query_str += " WHERE ";
                    query_str += &conditions.join(" AND ");
                    query_count_str += " WHERE ";
                    query_count_str += &conditions.join(" AND ");
                }
            }
            None => {}
        };

        query_count_str += " ORDER BY id DESC";
        let count_query = sqlx::query(&query_count_str).fetch_one(&mut *tx).await?;
        let count = count_query.get::<i64, _>(0);

        query_str += &format!(
            " ORDER BY id DESC LIMIT {} OFFSET {}",
            request.page_size,
            request.page * request.page_size
        );

        let paged_query = sqlx::query(&query_str);
        let files_row = paged_query.fetch_all(&mut *tx).await?;
        let files: Vec<FileEntry> = files_row
            .into_iter()
            .map(|row| FileEntry {
                id: row.get("id"),
                name: row.get("name"),
                size: row.get("size"),
                path: row.get("path"),
                class: row.get("class"),
                is_training_data: row.get("is_training_data"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok((files, count))
    }

    async fn init(&self) {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let table_query: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='file'")
                .fetch_one(&mut *tx)
                .await
                .expect("Failed to check if file table exists");

        if table_query.0 == 0 {
            sqlx::query(
                "CREATE TABLE file (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT KEY NOT NULL,
                    size REAL NOT NULL,
                    path TEXT NOT NULL,
                    class INTEGER NOT NULL,
                    is_training_data BOOLEAN NOT NULL DEFAULT 0,
                    created_at REAL NOT NULL,
                    UNIQUE (name)
                )",
            )
            .execute(&mut *tx)
            .await
            .expect("Failed to create file table");
        }

        tx.commit().await.expect("Failed to commit transaction");
    }

    pub async fn update_class(&self, req: &UpdateClassRequest) -> Result<(), sqlx::Error> {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let update_query = sqlx::query("UPDATE file SET class = ? WHERE id = ?")
            .bind(req.class)
            .bind(req.id);
        let update_query = update_query.execute(&mut *tx).await;
        if let Err(e) = update_query {
            tracing::error!("Failed to update class: {}", e);
            return Err(e);
        }

        if let Err(e) = tx.commit().await {
            tracing::error!("Failed to commit transaction: {}", e);
            return Err(e);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: Option<i64>,
    pub name: String,
    pub size: f64,
    pub path: String,
    pub class: i32,
    pub is_training_data: Option<bool>,
    pub created_at: f64,
}

#[derive(Debug, Deserialize)]
pub struct FileEntryFilter {
    pub name: Option<String>,
    pub class: Option<i32>,
    pub is_training_data: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClassRequest {
    pub id: i64,
    pub class: i32,
}
