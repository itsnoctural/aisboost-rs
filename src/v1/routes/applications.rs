use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use libsql::{de, params};
use serde::{Deserialize, Serialize};

use super::users::User;
use crate::v1::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    id: i64,
    name: String,

    duration: u8,
    checkpoints: u8,
    prefix: String,
    length: u8,
}

#[derive(Deserialize)]
pub struct ApplicationPayload {
    name: String,
    duration: u8,
    checkpoints: u8,
    prefix: String,
    length: u8,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Json<Vec<Application>> {
    let connection = state.db.connect().unwrap();

    let mut rows = connection
        .query(
            "SELECT id, name, duration, checkpoints, prefix, length, user_id FROM application WHERE user_id = ?1",
            params![user.id],
        )
        .await
        .unwrap();
    let mut applications: Vec<Application> = Vec::new();

    while let Some(row) = rows.next().await.unwrap() {
        if let Ok(application) = de::from_row::<Application>(&row) {
            applications.push(application);
        }
    }

    Json(applications)
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<ApplicationPayload>,
) -> StatusCode {
    // TODO: refactor validation
    if (payload.name.len() > 32 || payload.name.len() < 2)
        || payload.duration < 1
        || (payload.checkpoints > 5 || payload.checkpoints < 1)
        || (payload.prefix.len() > 6 || payload.prefix.len() < 1)
        || (payload.length > 24 || payload.length < 8)
    {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    let connection = state.db.connect().unwrap();

    let rows_changed = connection.execute(
        "INSERT INTO application (user_id, name, duration, checkpoints, prefix, length) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
        params![user.id, payload.name, payload.duration, payload.checkpoints, payload.prefix, payload.length],
    ).await.unwrap();

    if rows_changed > 0 {
        return StatusCode::CREATED;
    }

    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
) -> Result<Json<Application>, StatusCode> {
    let connection = state.db.connect().unwrap();

    let row = connection.query(
        "SELECT id, name, duration, checkpoints, prefix, length, webhook, webhook_content, user_id FROM application WHERE id = ?1 AND user_id = ?2", 
        params![id, user.id],
    ).await.unwrap().next().await.unwrap();

    if let Some(row) = row {
        if let Ok(application) = de::from_row(&row) {
            return Ok(Json(application));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

pub async fn patch_by_id(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(payload): Json<ApplicationPayload>,
) -> StatusCode {
    let connection = state.db.connect().unwrap();

    let rows_changed = connection.execute(
        "UPDATE application SET name = ?1, duration = ?2, checkpoints = ?3, prefix = ?4, length = ?5 WHERE id = ?6 AND user_id = ?7",
        params![payload.name, payload.duration, payload.checkpoints, payload.prefix, payload.length, id, user.id]
    ).await.unwrap();

    if rows_changed > 0 {
        return StatusCode::OK;
    }

    StatusCode::NOT_FOUND
}

pub async fn delete_by_id(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
) -> StatusCode {
    let connection = state.db.connect().unwrap();

    let rows_changed = connection
        .execute(
            "DELETE FROM application WHERE id = ?1 AND user_id = ?2",
            params![id, user.id],
        )
        .await
        .unwrap();

    if rows_changed > 0 {
        return StatusCode::NO_CONTENT;
    }

    StatusCode::NOT_FOUND
}
