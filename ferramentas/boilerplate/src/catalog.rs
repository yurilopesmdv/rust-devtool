use crate::{Error, config::ProjectType};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

include!(concat!(env!("OUT_DIR"), "/catalog.rs"));
pub const VERSION: &str = "1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub command: String,
    pub args: Vec<String>,
    pub purpose: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Planned,
    Ready,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub id: String,
    pub role: ProjectType,
    pub language: String,
    pub framework: String,
    pub integration_adapter: Option<String>,
    #[serde(default)]
    pub port: u16,
    pub container_port: Option<u16>,
    pub architecture: String,
    pub status: Status,
    pub package_manager: String,
    pub paths: Vec<String>,
    pub capabilities: Vec<String>,
    pub install: Vec<String>,
    #[serde(default)]
    pub module_systems: Vec<String>,
    #[serde(default)]
    pub requirements: Vec<Requirement>,
    #[serde(default)]
    pub conditional_paths: BTreeMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct Catalog {
    pub templates: Vec<Template>,
}

impl Catalog {
    pub fn embedded() -> Result<Self, Error> {
        Self::parse(SOURCES)
    }

    pub fn parse(sources: &[&str]) -> Result<Self, Error> {
        let mut templates = Vec::new();
        let mut ids = BTreeSet::new();
        for source in sources {
            let template: Template = toml::from_str(source)
                .map_err(|e| Error::operation(format!("Metadados de template inválidos: {e}")))?;
            if template.id.is_empty()
                || !template
                    .id
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
                || !ids.insert(template.id.clone())
            {
                return Err(Error::operation("ID de template vazio ou duplicado"));
            }
            if template.role == ProjectType::Fullstack {
                return Err(Error::operation(
                    "Template deve ter papel frontend ou backend",
                ));
            }
            for path in template
                .paths
                .iter()
                .chain(template.conditional_paths.values().flatten())
            {
                if path.is_empty()
                    || path.starts_with('/')
                    || path.contains('\\')
                    || path.split('/').any(|part| part == "..")
                {
                    return Err(Error::operation(format!(
                        "Caminho de catálogo inválido: {path}"
                    )));
                }
            }
            if template.requirements.iter().any(|r| {
                r.command.is_empty() || r.command.contains('/') || r.command.contains('\\')
            }) {
                return Err(Error::operation(
                    "Requisito deve nomear um executável no PATH",
                ));
            }
            templates.push(template);
        }
        templates.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self { templates })
    }

    pub fn get(&self, id: &str) -> Result<&Template, Error> {
        self.templates.iter().find(|t| t.id == id).ok_or_else(|| {
            Error::input(format!(
                "Template desconhecido: {id}. Consulte boilerplate templates list"
            ))
        })
    }
}
