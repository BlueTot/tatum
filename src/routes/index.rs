use std::path::PathBuf;

use axum::{extract::Query, response::Html};
use resolve_path::PathResolveExt;
use serde::Deserialize;
use tracing::info;

use axum::extract::Extension;
use std::sync::Arc;

use crate::render::render_doc;
use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct IndexParams {
    path: PathBuf,
}

pub async fn index(
    Query(IndexParams { path }): Query<IndexParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> Html<String> {
    info!("Rendering document {}", path.to_string_lossy());

    Html(render_doc(path.resolve(), true, state.template_path.clone()).await.unwrap())
}
