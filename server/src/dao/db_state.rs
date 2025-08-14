use std::sync::{Arc, Mutex};

pub type DBPool = sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct DBClientState {
    pub inner: Arc<Mutex<DBPool>>,
}

impl DBClientState {
    pub async fn new() -> Self {
        let db_uri = "data.db";
        let db_file_path = std::path::Path::new(db_uri);
        if !db_file_path.exists() {
            std::fs::File::create(db_file_path).expect("Failed to create SQLite database file");
        }
        let conn = sqlx::sqlite::SqlitePool::connect(db_uri)
            .await
            .expect("Failed to connect to SQLite database");
        let inner = Arc::new(Mutex::new(conn));
        Self { inner }
    }

    pub fn get_pool(&self) -> DBPool {
        self.inner.lock().unwrap().clone()
    }
}
