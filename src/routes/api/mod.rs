mod files;

use crate::types::AppState;
use axum::Router;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().merge(files::router())
}
