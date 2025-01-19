use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    email: String,

    created_at: i64,
}

pub async fn me(Extension(user): Extension<User>) -> Json<User> {
    Json(user)
}
