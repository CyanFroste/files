use crate::{
    types::{AppState, Folder, Result},
    utils::{generate_breadcrumbs, get_thumbnail_path},
};
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
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
    const TEMPLATE: &str = "folder.tera";
    debug!(data = path, "requested folder");

    let cached: Option<Folder> = state.db.select(("folder", &path)).await?;
    debug!(data = ?cached, "retrieved folder from db");

    let folder = if let Some(folder) = cached {
        folder
    } else {
        let folder = Folder::from(&path)?;
        let saved = state
            .db
            .create(("folder", &path))
            .content(&folder)
            .await?
            .ok_or_else(|| "failed to save folder")?;

        debug!(data = ?saved, "saved folder to db");
        saved
    };

    let mut ctx = Context::new();
    ctx.insert("path", &path);
    ctx.insert("folder", &folder);
    ctx.insert("breadcrumbs", &generate_breadcrumbs(&path));

    let thumbnails: HashMap<_, _> = HashMap::from_iter(folder.contents.iter().filter_map(|x| {
        if x.is_dir {
            None
        } else {
            get_thumbnail_path(&state.config.thumbnail_dir, &x.path)
                .ok()
                .filter(|p| p.exists())
                .map(|p| (&x.path, p))
        }
    }));

    ctx.insert("thumbnails", &thumbnails);

    Ok(Html(state.tmpl.render(TEMPLATE, &ctx)?))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/folder", get(view))
}
