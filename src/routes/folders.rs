use crate::types::{AppState, Folder, Result};
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
    const TEMPLATE: &str = "folders/view.tera";
    debug!(data = path, "requested folder");

    let cached: Option<Folder> = state.db.select(("folder", &path)).await?;
    debug!(data = ?cached, "retrieved folder from db");

    let folder = if let Some(folder) = cached {
        folder
    } else {
        let folder = Folder::from(&path)?;
        let saved: Option<Folder> = state.db.create(("folder", &path)).content(&folder).await?;

        debug!(data = ?saved, "saved folder to db");
        folder
    };

    let mut ctx = Context::new();
    ctx.insert("path", &path);
    ctx.insert("folder", &folder);

    Ok(Html(state.tmpl.render(TEMPLATE, &ctx)?))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/folders/view", get(view))
}
