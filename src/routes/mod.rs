use axum::{routing::get, Router};
use axum::extract::Extension;
use std::sync::Arc;
use tower_http::services::ServeDir;

mod index;
mod watch;
use index::index;
use watch::watch;

#[derive(Clone)]
struct AppState {
    template_path: String,
}

pub fn construct_router(template_path: String) -> Router {

    let serve_path = template_path.clone();

    let app_state = AppState {
        template_path
    };

    Router::new()
        .route("/", get(index))
        .route("/watch", get(watch))
        .layer(Extension(Arc::new(app_state)))
        .nest_service("/static", ServeDir::new(serve_path))
}
