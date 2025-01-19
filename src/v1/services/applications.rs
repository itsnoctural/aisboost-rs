use axum::http::StatusCode;
use libsql::{de, params, Connection};

use crate::v1::routes::applications::Application;

pub async fn get_by_id(
    connection: &Connection,
    user_id: &i64,
    id: &i64,
) -> Result<Application, StatusCode> {
    let row = connection.query(
        "SELECT id, name, duration, checkpoints, prefix, length, webhook, webhook_content, user_id FROM application WHERE id = ?1 AND user_id = ?2", 
        params![id, user_id],
    ).await.unwrap().next().await.unwrap();

    if let Some(row) = row {
        if let Ok(application) = de::from_row(&row) {
            return Ok(application);
        }
    }

    Err(StatusCode::NOT_FOUND)
}
