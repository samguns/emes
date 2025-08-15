use serde::Serialize;
use sqlx::Acquire as _;
use sqlx::Row;

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
            label: file_query.get("label"),
            created_at: file_query.get("created_at"),
        })
    }

    pub async fn insert_file(&self, file_entry: FileEntry) -> Result<(), sqlx::Error> {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let insert_query = sqlx::query(
            "INSERT INTO file (name, size, path, label, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(file_entry.name)
        .bind(file_entry.size)
        .bind(file_entry.path)
        .bind(file_entry.label)
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
                    label TEXT NOT NULL,
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
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub id: Option<i64>,
    pub name: String,
    pub size: f64,
    pub path: String,
    pub label: String,
    pub created_at: f64,
}
