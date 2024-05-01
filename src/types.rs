use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use surrealdb::{engine::local::Db, sql::Thing, Surreal};
use tera::Tera;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: Option<Thing>,
    pub name: String,
    pub path: String,
    pub r#type: Option<String>,
    pub favorite: bool,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<Thing>,
    pub name: String,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub id: Option<Thing>,
    pub contents: Vec<FolderContent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderContent {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

impl File {
    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let name = path
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .ok_or_else(|| -> Error { "invalid file name".into() })?;

        let r#type = path.extension().map(|x| x.to_string_lossy().to_string());

        let path = path.to_string_lossy().to_string();

        Ok(File {
            id: None,
            name,
            path,
            r#type,
            favorite: false,
            tags: vec![],
        })
    }
}

impl Folder {
    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_dir(path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let is_dir = entry.file_type().ok()?.is_dir();

                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path().to_string_lossy().to_string();

                Some(FolderContent { name, path, is_dir })
            })
            .collect();

        Ok(Self { id: None, contents })
    }
}

#[derive(Debug)]
pub struct AppState {
    pub db: Surreal<Db>,
    pub tmpl: Tera,
    pub config: Config,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub allowed_paths: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error(String);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<surrealdb::Error> for Error {
    fn from(e: surrealdb::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<tera::Error> for Error {
    fn from(e: tera::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<dotenvy::Error> for Error {
    fn from(e: dotenvy::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self(e.to_string())
    }
}
