mod routes;
mod types;
mod utils;

use crate::types::{AppState, Config, Result};
use std::{net::SocketAddr, sync::Arc};
use surrealdb::{engine::local::RocksDb, Surreal};
use tera::Tera;
use tokio::net::TcpListener;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from(
            dotenvy::var("LOGGING").unwrap_or("simp=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = dotenvy::var("PORT")?.parse().unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    let db = Surreal::new::<RocksDb>("db").await?;
    db.use_ns("files").use_db("files").await?;

    let tmpl = Tera::new("templates/**/*.tera")?;
    let config = Config::from_file("config.json")?;

    let state = Arc::new(AppState { db, tmpl, config });
    let app = routes::router(state).into_make_service();

    debug!("listening on http://localhost:{}", port);

    Ok(axum::serve(listener, app).await?)
}
