use crate::types::{AppState, File, Result};
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tera::Context;
use tracing::debug;

#[derive(Deserialize)]
struct ViewQueryParams {
    path: String,
}

async fn view(
    state: State<Arc<AppState>>,
    Query(ViewQueryParams { path }): Query<ViewQueryParams>,
) -> Result<impl IntoResponse> {
    const TEMPLATE: &str = "api/files/view.tera";
    debug!(data = path, "requested file");

    let cached: Option<File> = state.db.select(("file", &path)).await?;
    debug!(data = ?cached, "retrieved file from db");

    let file = if let Some(file) = cached {
        file
    } else {
        let file = File::from(&path)?;
        let saved: Option<File> = state.db.create(("file", &path)).content(&file).await?;

        debug!(data = ?saved, "saved file to db");
        file
    };

    let mut ctx = Context::new();
    ctx.insert("file", &file);

    Ok(Html(state.tmpl.render(TEMPLATE, &ctx)?))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/files/view", get(view))
}
