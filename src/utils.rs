use crate::types::{BreadCrumb, Result};
use md5::{Digest, Md5};
use std::path::{Path, PathBuf};

pub mod templates {
    use std::collections::HashMap;
    use surrealdb::sql::Thing;
    use tera::{from_value, Error, Result, Value};

    pub fn stringify_db_id(args: &HashMap<String, Value>) -> Result<Value> {
        let value = args.get("id").ok_or_else(|| Error::msg("missing id"))?;
        let id = from_value::<Thing>(value.clone())?;

        Ok(tera::Value::String(id.id.to_raw()))
    }
}

pub fn generate_breadcrumbs(path: impl AsRef<Path>) -> Vec<BreadCrumb> {
    use std::path::{Component, MAIN_SEPARATOR_STR};

    let path = path.as_ref();
    let mut breadcrumbs = Vec::new();
    let mut acc = PathBuf::new();

    for c in path.components() {
        match c {
            Component::RootDir => continue,
            c => {
                acc.push(c);

                if matches!(c, Component::Prefix(_)) {
                    acc.push(MAIN_SEPARATOR_STR);
                }

                breadcrumbs.push(BreadCrumb {
                    path: acc.to_string_lossy().to_string(),
                    name: c.as_os_str().to_string_lossy().to_string(),
                    is_dir: acc.is_dir(),
                });
            }
        }
    }

    breadcrumbs
}

pub async fn generate_thumbnail(
    base_path: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<PathBuf> {
    let base_path = PathBuf::from(base_path.as_ref());
    let path = PathBuf::from(path.as_ref());

    tokio::task::spawn_blocking(move || -> Result<PathBuf> {
        let img = image::open(&path)?;
        let thumbnail = img.thumbnail(512, 512);

        let encoder = webp::Encoder::from_image(&thumbnail)?;
        let encoded = encoder.encode(80.);
        let thumbnail_path = get_thumbnail_path(base_path, path)?;

        std::fs::write(&thumbnail_path, &*encoded)?;
        Ok(thumbnail_path)
    })
    .await?
}

pub fn get_thumbnail_path(base_path: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<PathBuf> {
    let base_path = base_path.as_ref();
    let path = path.as_ref();

    let mut hasher = Md5::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let thumbnail_path = base_path.join(&hash).with_extension("webp");
    Ok(thumbnail_path)
}

pub fn is_previewable(ext: &str) -> bool {
    matches!(ext, "jpg" | "jpeg" | "png" | "webp" | "gif")
}

pub fn get_mime_type(ext: &str) -> String {
    match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
    .into()
}
