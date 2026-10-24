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
pub(super) fn source_mut<'a>(
    files: &'a mut Files,
    prefix: &Path,
    path: &str,
) -> Result<&'a mut String, Error> {
    files
        .get_mut(&prefix.join(path))
        .ok_or_else(|| Error::operation(format!("Adapter sem fonte {path}")))
}

pub(super) fn anchor_position(source: &str, path: &str, anchor: &str) -> Result<usize, Error> {
    if anchor.is_empty() {
        return Err(Error::operation(format!("Âncora vazia em {path}")));
    }
    let mut matches = source
        .char_indices()
        .filter_map(|(position, _)| source[position..].starts_with(anchor).then_some(position));
    let first = matches.next();
    let duplicate = matches.next();
    match (first, duplicate) {
        (Some(position), None) => Ok(position),
        _ => {
            let count =
                usize::from(first.is_some()) + usize::from(duplicate.is_some()) + matches.count();
            Err(Error::operation(format!(
                "Contrato de extensão mudou em {path}: âncora {anchor:?} esperada exatamente uma vez; encontradas {count}"
            )))
        }
    }
}

pub(super) fn replace_source(
    source: &mut String,
    path: &str,
    from: &str,
    to: &str,
) -> Result<(), Error> {
    let position = anchor_position(source, path, from)?;
    source.replace_range(position..position + from.len(), to);
    Ok(())
}

pub fn replace(
    files: &mut Files,
    prefix: &Path,
    path: &str,
    from: &str,
    to: &str,
) -> Result<(), Error> {
    replace_source(source_mut(files, prefix, path)?, path, from, to)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_only_an_exact_unique_anchor() {
        let mut files =
            Files::from([(PathBuf::from("src/app.ts"), "before (anchor) after".into())]);
        replace(&mut files, Path::new(""), "src/app.ts", "(anchor)", "new").unwrap();
        assert_eq!(files[&PathBuf::from("src/app.ts")], "before new after");
    }

    #[test]
    fn invalid_anchors_report_context_without_modifying_sources() {
        for (source, anchor, detail) in [
            ("before", "absent", "encontradas 0"),
            ("anchor anchor", "anchor", "encontradas 2"),
            ("ababa", "aba", "encontradas 2"),
            ("before", "", "Âncora vazia"),
        ] {
            let mut files = Files::from([(PathBuf::from("src/app.ts"), source.into())]);
            let error =
                replace(&mut files, Path::new(""), "src/app.ts", anchor, "new").unwrap_err();
            assert_eq!(error.code, 1);
            assert!(error.message.contains("src/app.ts"));
            assert!(error.message.contains(detail));
            assert!(error.message.contains(anchor));
            assert_eq!(files[&PathBuf::from("src/app.ts")], source);
        }
    }

    #[test]
    fn missing_source_is_an_operational_error() {
        let error = replace(
            &mut Files::new(),
            Path::new("apps/api"),
            "src/app.ts",
            "anchor",
            "new",
        )
        .unwrap_err();
        assert_eq!(error.code, 1);
        assert!(error.message.contains("src/app.ts"));
    }
}
