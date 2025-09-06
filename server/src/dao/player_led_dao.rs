use serde::Deserialize;
use serde::Serialize;
use sqlx::Acquire as _;
use sqlx::Row;

use crate::dao::db_state::DBClientState;

pub struct PlayerLedDao {
    db_client_state: DBClientState,
}

impl PlayerLedDao {
    pub async fn new(db_client_state: &DBClientState) -> Self {
        let player_led_dao = PlayerLedDao {
            db_client_state: db_client_state.clone(),
        };

        player_led_dao.init().await;

        player_led_dao
    }

    pub async fn get_led_strip_status(&self) -> Result<PlayerLedEntry, sqlx::Error> {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let led_strip_query = sqlx::query("SELECT * FROM player_led")
            .fetch_one(&mut *tx)
            .await;
        if let Err(e) = led_strip_query {
            tracing::error!("Failed to query led strip: {}", e);
            return Err(e);
        }

        if let Err(e) = tx.commit().await {
            tracing::error!("Failed to commit transaction: {}", e);
            return Err(e);
        }

        let led_strip_query = led_strip_query.unwrap();
        let led_strip_entry = PlayerLedEntry {
            id: led_strip_query.get("id"),
            frequency: led_strip_query.get("frequency"),
            scale: led_strip_query.get("scale"),
            red: led_strip_query.get("red"),
            green: led_strip_query.get("green"),
            blue: led_strip_query.get("blue"),
        };

        Ok(led_strip_entry)
    }

    pub async fn set_led_strip_status(&self, req: PlayerLedEntry) -> Result<(), sqlx::Error> {
        let pool = self.db_client_state.get_pool();
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        let check_query = sqlx::query("SELECT COUNT(*) FROM player_led WHERE id = ?")
            .bind(req.id)
            .fetch_one(&mut *tx)
            .await;
        if let Err(e) = check_query {
            tracing::error!("Failed to check if led strip exists: {}", e);
            return Err(e);
        }

        // Properly extract the count from the row using get::<type, &str>("column_name")
        let count: i64 = check_query.unwrap().get::<i64, _>("COUNT(*)");
        // If the led strip does not exist, create it
        if count == 0 {
            tracing::error!("Led strip does not exist, creating it");
            let insert_query = sqlx::query("INSERT INTO player_led (id, frequency, scale, red, green, blue) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(req.id)
                .bind(req.frequency)
                .bind(req.scale)
                .bind(req.red)
                .bind(req.green)
                .bind(req.blue);
            let insert_query = insert_query.execute(&mut *tx).await;
            if let Err(e) = insert_query {
                tracing::error!("Failed to insert led strip: {}", e);
                return Err(e);
            }

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction: {}", e);
                return Err(e);
            }

            return Ok(());
        }

        tracing::info!("Led strip exists, updating it");

        let update_query = sqlx::query("UPDATE player_led SET frequency = ?, scale = ?, red = ?, green = ?, blue = ? WHERE id = ?")
            .bind(req.frequency)
            .bind(req.scale)
            .bind(req.red)
            .bind(req.green)
            .bind(req.blue)
            .bind(req.id);
        let update_query = update_query.execute(&mut *tx).await;
        if let Err(e) = update_query {
            tracing::error!("Failed to update led strip: {}", e);
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

        let table_query: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='player_led'",
        )
        .fetch_one(&mut *tx)
        .await
        .expect("Failed to check if player_led table exists");

        if table_query.0 == 0 {
            sqlx::query(
                "CREATE TABLE player_led (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    frequency REAL NOT NULL,
                    scale REAL NOT NULL,
                    red INTEGER NOT NULL,
                    green INTEGER NOT NULL,
                    blue INTEGER NOT NULL,
                    UNIQUE (id)
                )",
            )
            .execute(&mut *tx)
            .await
            .expect("Failed to create player_led table");
        }

        tx.commit().await.expect("Failed to commit transaction");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerLedEntry {
    pub id: i64,
    pub frequency: f64,
    pub scale: f64,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
