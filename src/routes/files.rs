use crate::{
    types::{AppState, File, Result, Tag},
    utils::{generate_breadcrumbs, generate_thumbnail, get_thumbnail_path, is_previewable},
};
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::Form;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
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
    const TEMPLATE: &str = "file.tera";
    debug!(data = path, "requested file");

    let cached: Option<File> = state.db.select(("file", &path)).await?;
    debug!(data = ?cached, "retrieved file from db");

    let file = if let Some(file) = cached {
        file
    } else {
        let file = File::from(&path)?;
        let saved = state
            .db
            .create(("file", &path))
            .content(&file)
            .await?
            .ok_or_else(|| "failed to save file")?;

        debug!(data = ?saved, "saved file to db");
        saved
    };

    let sql = format!(
        "SELECT * FROM tag WHERE id NOT IN [{}]",
        file.tags
            .iter()
            .filter_map(|x| x.id.as_ref().map(|x| x.to_raw()))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut res = state.db.query(sql).await?;
    let tags: Vec<Tag> = res.take(0)?;

    let mut ctx = Context::new();
    ctx.insert("file", &file);
    ctx.insert("tags", &tags);
    ctx.insert("breadcrumbs", &generate_breadcrumbs(&path));
    ctx.insert(
        "previewable",
        &is_previewable(&file.r#type.unwrap_or_default()),
    );
    ctx.insert(
        "thumbnail",
        &get_thumbnail_path(&state.config.thumbnail_dir, &file.path)
            .ok()
            .filter(|p| p.exists()),
    );

    Ok(Html(state.tmpl.render(TEMPLATE, &ctx)?))
}

#[derive(Debug, Deserialize)]
struct EditTagsForm {
    removed: Option<Vec<String>>,
    added: Option<Vec<String>>,
    file_path: String,
    file_id: String,
}

#[derive(Debug, Serialize)]
struct EditTagsMerge {
    tags: HashSet<Tag>,
}

async fn edit_tags(
    state: State<Arc<AppState>>,
    Form(form): Form<EditTagsForm>,
) -> Result<impl IntoResponse> {
    if form.removed.is_none() && form.added.is_none() {
        return Ok(Redirect::to(&format!("/file?path={}", form.file_path)));
    }

    let file: Option<File> = state.db.select(("file", &form.file_id)).await?;
    let file = file.ok_or_else(|| "file not found")?;

    let mut tags = HashSet::from_iter(file.tags.into_iter());

    if let Some(removed) = form.removed {
        tags.retain(|x| {
            if let Some(id) = x.id.as_ref() {
                !removed.contains(&id.to_raw())
            } else {
                false
            }
        });
    }

    if let Some(added) = form.added {
        let sql = format!("SELECT * FROM [{}]", added.join(", "));
        let mut res = state.db.query(sql).await?;
        let added: Vec<Tag> = res.take(0)?;

        tags.extend(added.into_iter());
    }

    let _: Option<File> = state
        .db
        .update(("file", &form.file_id))
        .merge(EditTagsMerge { tags })
        .await?;

    Ok(Redirect::to(&format!("/file?path={}", form.file_path)))
}

#[derive(Debug, Deserialize)]
struct CreateThumbnailForm {
    // file_id: String,
    file_path: String,
}

async fn create_thumbnail(
    state: State<Arc<AppState>>,
    Form(form): Form<CreateThumbnailForm>,
) -> Result<impl IntoResponse> {
    const TEMPLATE: &str = "created_file_thumbnail.tera";

    let thumbnail_path = generate_thumbnail(&state.config.thumbnail_dir, &form.file_path).await?;

    let mut ctx = Context::new();
    ctx.insert("thumbnail", &thumbnail_path);

    Ok(Html(state.tmpl.render(TEMPLATE, &ctx)?))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/file", get(view))
        .route("/file/tags", post(edit_tags))
        .route("/file/thumbnail", post(create_thumbnail))
}
