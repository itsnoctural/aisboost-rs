use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use headers::{Cookie, HeaderMapExt};
use libsql::{de, params, Connection};

use super::{routes::users::User, AppState};

pub async fn authorization(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let cookies = req
        .headers()
        .typed_get::<Cookie>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let session_id = cookies
        .get("aisboost.auth")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let connection = state.db.connect().unwrap();
    let user = get_user(&connection, session_id).await;

    if let Some(user) = user {
        req.extensions_mut().insert(user);
        return Ok(next.run(req).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}

async fn get_user(connection: &Connection, session_id: &str) -> Option<User> {
    let row = connection.query("SELECT session.id as session_id, session.expires_at, user.id, user.email, user.created_at FROM session JOIN user ON session.user_id = user.id WHERE session_id = ?1", params![session_id]).await.unwrap().next().await.unwrap();
    if let Some(row) = row {
        let expires_at: i64 = row.get(1).unwrap();

        if Utc::now().timestamp() >= expires_at {
            let _ = connection
                .execute("DELETE FROM session WHERE id = ?1", params![session_id])
                .await;
            return None;
        }

        if let Ok(user) = de::from_row::<User>(&row) {
            return Some(user);
        }
    }

    None
}
