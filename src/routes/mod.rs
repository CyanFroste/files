mod files;
mod folders;

use crate::types::{AppState, Result};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::sync::Arc;
use tera::Context;
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

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(home))
        .merge(folders::router())
        .merge(files::router())
        .with_state(state)
        .fallback_service(ServeDir::new("public"))
}
