use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use libsql::{de, params};
use serde::{Deserialize, Serialize};

use super::users::User;
use crate::v1::{services, AppState};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Type {
    Linkvertise,
    Lootlabs,
    Workink,
    Shrtfly,
}

impl Type {
    fn to_string(self) -> String {
        match self {
            Type::Linkvertise => String::from("linkvertise"),
            Type::Lootlabs => String::from("lootlabs"),
            Type::Workink => String::from("workink"),
            Type::Shrtfly => String::from("shrtfly"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Template {
    id: i64,
    r#type: String,
    api_key: String,
    api_url: String,
}

#[derive(Deserialize)]
pub struct TemplatePayload {
    r#type: Type,
    api_key: String,
    api_url: String,
}

pub async fn index(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(application_id): Path<i64>,
) -> Result<Json<Vec<Template>>, StatusCode> {
    let connection = state.db.connect().unwrap();

    if let Err(err) =
        services::applications::get_by_id(&connection, &user.id, &application_id).await
    {
        return Err(err);
    }

    let mut rows = connection
        .query(
            "SELECT id, type, api_key, api_url FROM template WHERE application_id = ?1",
            params![application_id],
        )
        .await
        .unwrap();
    let mut templates: Vec<Template> = Vec::new();

    while let Some(row) = rows.next().await.unwrap() {
        if let Ok(template) = de::from_row::<Template>(&row) {
            templates.push(template);
        }
    }

    Ok(Json(templates))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(application_id): Path<i64>,
    Json(payload): Json<TemplatePayload>,
) -> StatusCode {
    let connection = state.db.connect().unwrap();

    if let Err(err) =
        services::applications::get_by_id(&connection, &user.id, &application_id).await
    {
        return err;
    }

    let rows_changed = connection
        .execute(
            "INSERT INTO template (type, api_key, api_url, application_id) VALUES (?1, ?2, ?3, ?4)",
            params![
                payload.r#type.to_string(),
                payload.api_key,
                payload.api_url,
                application_id
            ],
        )
        .await
        .unwrap();

    if rows_changed > 0 {
        return StatusCode::CREATED;
    }

    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((application_id, id)): Path<(i64, i64)>,
) -> Result<Json<Template>, StatusCode> {
    let connection = state.db.connect().unwrap();

    let row = connection.query(
        "SELECT template.id, type, api_key, api_url FROM template JOIN application ON template.application_id = application.id WHERE template.id = ?1 AND application.id = ?2 AND application.user_id = ?3",
        params![id, application_id, user.id],
    ).await.unwrap().next().await.unwrap();

    if let Some(row) = row {
        if let Ok(template) = de::from_row(&row) {
            return Ok(Json(template));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

pub async fn patch_by_id(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((application_id, id)): Path<(i64, i64)>,
    Json(payload): Json<TemplatePayload>,
) -> StatusCode {
    let connection = state.db.connect().unwrap();

    if let Err(err) =
        services::applications::get_by_id(&connection, &user.id, &application_id).await
    {
        return err;
    }

    let rows_changed = connection.execute(
        "UPDATE template SET type = ?1, api_key = ?2, api_url = ?3 WHERE id = ?4 AND application_id = ?5",
        params![
            payload.r#type.to_string(),
            payload.api_key,
            payload.api_url,
            id,
            application_id
        ],
    ).await.unwrap();

    if rows_changed > 0 {
        return StatusCode::OK;
    }

    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn delete_by_id(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((application_id, id)): Path<(i64, i64)>,
) -> StatusCode {
    let connection = state.db.connect().unwrap();

    if let Err(err) =
        services::applications::get_by_id(&connection, &user.id, &application_id).await
    {
        return err;
    }

    let rows_changed = connection
        .execute(
            "DELETE FROM template WHERE id = ?1 AND application_id = ?2",
            params![id, application_id],
        )
        .await
        .unwrap();

    if rows_changed > 0 {
        return StatusCode::NO_CONTENT;
    }

    StatusCode::NOT_FOUND
}
