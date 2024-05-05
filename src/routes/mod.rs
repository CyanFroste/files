mod files;
mod folders;

use crate::{
    types::{AppState, Result},
    utils::get_mime_type,
};
use axum::{
    body::Body,
    extract::{Query, State},
    http::header::CONTENT_TYPE,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tera::Context;
use tokio_util::io::ReaderStream;
use tower_http::services::ServeDir;

// async fn search(state: State<Arc<AppState>>) -> Result<impl IntoResponse> {
//     // let r = db
//     //     .query("SELECT * FROM file WHERE tags[WHERE name = 'tag_two']")
//     //     .await?;

//     Ok(())
// }

async fn home(state: State<Arc<AppState>>) -> Result<impl IntoResponse> {
    const TEMPLATE: &str = "home.tera";

    let mut ctx = Context::new();
    ctx.insert("allowed_paths", &state.config.allowed_paths);

    return Ok(Html(state.tmpl.render(TEMPLATE, &ctx)?));
}

#[derive(Deserialize)]
struct MediaQueryParams {
    path: String,
}

async fn media(
    Query(MediaQueryParams { path }): Query<MediaQueryParams>,
) -> Result<impl IntoResponse> {
    let path = std::path::Path::new(&path);
    let file = tokio::fs::File::open(&path).await?;
    let stream = ReaderStream::new(file);

    let ext = path
        .extension()
        .ok_or_else(|| "missing extension")?
        .to_string_lossy();

    let headers = [(CONTENT_TYPE, get_mime_type(&ext))];

    Ok((headers, Body::from_stream(stream)))
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/media", get(media))
        .merge(folders::router())
        .merge(files::router())
        .with_state(state)
        .fallback_service(ServeDir::new("public"))
}
