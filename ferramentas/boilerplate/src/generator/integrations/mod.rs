pub mod docs;
pub mod infra;
pub mod node;
pub mod python;
pub mod rust;
use crate::{Error, catalog::Template, config::*};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
pub type Files = BTreeMap<PathBuf, String>;

pub fn insert(files: &mut Files, prefix: &Path, path: &str, content: impl Into<String>) {
    files.insert(prefix.join(path), content.into());
}
pub fn replace(
    files: &mut Files,
    prefix: &Path,
    path: &str,
    from: &str,
    to: &str,
) -> Result<(), Error> {
    let source = files
        .get_mut(&prefix.join(path))
        .ok_or_else(|| Error::operation(format!("Adapter sem fonte {path}")))?;
    if !source.contains(from) {
        return Err(Error::operation(format!(
            "Contrato de extensão mudou: {path}"
        )));
    }
    *source = source.replacen(from, to, 1);
    Ok(())
}
pub fn backend(r: &Recipe, t: &Template, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    if t.role != ProjectType::Backend {
        return Ok(());
    }
    if r.backend.database == Some(Database::None)
        && r.backend.cache == Some(Cache::None)
        && r.infrastructure.monitoring == Some(Monitoring::None)
        && r.infrastructure.secrets == Some(Secrets::Env)
    {
        return Ok(());
    }
    match t.integration_adapter.as_deref() {
        Some("node-express" | "node-nest") => node::apply(r, t, files, prefix),
        Some("rust-axum") => rust::apply(r, files, prefix),
        Some("python-fastapi") => python::apply(r, files, prefix),
        _ => Err(Error::input(format!(
            "{} não possui adapter de integrações",
            t.id
        ))),
    }
}

pub fn frontend(r: &Recipe, t: &Template, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    if t.role == ProjectType::Frontend
        && r.backend.template.is_none()
        && r.infrastructure.monitoring == Some(Monitoring::PrometheusGrafana)
        && t.integration_adapter.as_deref() == Some("node-next")
    {
        node::next_metrics(files, prefix)?;
    }
    Ok(())
}
