use crate::types::BreadCrumb;
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
