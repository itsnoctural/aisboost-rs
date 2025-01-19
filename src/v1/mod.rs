mod middleware;
mod routes;
mod services;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Router,
};
use libsql::{Builder, Database};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

pub async fn root() -> Router {
    let db = Arc::new(
        Builder::new_remote(String::from("http://127.0.0.1:8080"), String::new())
            .build()
            .await
            .unwrap(),
    );
    let app_state = AppState { db };

    let users = Router::new().route("/@me", get(routes::users::me));
    let applications = Router::new()
        .route("/", get(routes::applications::index))
        .route("/", post(routes::applications::create))
        .route("/{id}", get(routes::applications::get_by_id))
        .route("/{id}", patch(routes::applications::patch_by_id))
        .route("/{id}", delete(routes::applications::delete_by_id));
    let templates = Router::new()
        .route("/{application_id}", get(routes::templates::index))
        .route("/{application_id}", post(routes::templates::create))
        .route("/{application_id}/{id}", get(routes::templates::get_by_id))
        .route(
            "/{application_id}/{id}",
            patch(routes::templates::patch_by_id),
        )
        .route(
            "/{application_id}/{id}",
            delete(routes::templates::delete_by_id),
        );

    let with_auth = Router::new()
        .nest("/users", users)
        .nest("/applications", applications)
        .nest("/templates", templates)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::authorization,
        ));

    Router::new().merge(with_auth).with_state(app_state)
}
