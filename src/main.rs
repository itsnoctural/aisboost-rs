mod v1;

use axum::{http::StatusCode, routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("/v1", v1::root().await)
        .route("/ping", get(|| async { StatusCode::OK }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
