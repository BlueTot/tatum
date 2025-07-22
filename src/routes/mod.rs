use axum::{routing::get, Router};
use axum::extract::Extension;
use std::sync::Arc;

mod index;
mod watch;
use index::index;
use watch::watch;

#[derive(Clone)]
struct AppState {
    template_path: String,
}

pub fn construct_router(template_path: String) -> Router {

    let app_state = AppState {
        template_path
    };

    Router::new()
        .route("/", get(index))
        .route("/watch", get(watch))
        .layer(Extension(Arc::new(app_state)))
}
