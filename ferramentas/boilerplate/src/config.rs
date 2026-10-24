use crate::{Error, catalog};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

macro_rules! choice {
    ($name:ident { $($variant:ident),+ }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
        #[serde(rename_all = "kebab-case")]
        pub enum $name { $($variant),+ }
    };
}
choice!(ProjectType {
    Frontend,
    Backend,
    Fullstack
});
choice!(Layout { Monorepo, Split });
choice!(Database {
    None,
    Postgres,
    Mongodb
});
choice!(Cache { None, Redis });
choice!(Location { Local, External });
choice!(Containers { None, Compose });
choice!(Kubernetes { None, Manifests });
choice!(Monitoring {
    None,
    PrometheusGrafana
});
choice!(Secrets { Env, Vault });

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Project {
    #[serde(rename = "type")]
    pub kind: Option<ProjectType>,
    pub name: Option<String>,
    pub destination: Option<PathBuf>,
    pub layout: Option<Layout>,
    pub web_dir: Option<PathBuf>,
    pub api_dir: Option<PathBuf>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Frontend {
    pub template: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Backend {
    pub template: Option<String>,
    pub database: Option<Database>,
    pub database_location: Option<Location>,
    pub cache: Option<Cache>,
    pub cache_location: Option<Location>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Infrastructure {
    pub containers: Option<Containers>,
    pub kubernetes: Option<Kubernetes>,
    pub monitoring: Option<Monitoring>,
    pub secrets: Option<Secrets>,
    pub vault_location: Option<Location>,
    pub namespace: Option<String>,
    pub image: Option<String>,
    pub image_web: Option<String>,
    pub image_api: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Recipe {
    pub schema_version: Option<u32>,
    pub catalog_version: Option<String>,
    pub project: Project,
    pub frontend: Frontend,
    pub backend: Backend,
    pub infrastructure: Infrastructure,
}

impl Recipe {
    pub fn load(path: &std::path::Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            Error::operation(format!("Não foi possível ler {}: {e}", path.display()))
        })?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, Error> {
        let recipe: Self = toml::from_str(text)
            .map_err(|e| Error::input(format!("Receita TOML inválida: {e}")))?;
        if recipe.schema_version != Some(1)
            || recipe.catalog_version.as_deref() != Some(catalog::VERSION)
        {
            return Err(Error::input(
                "Receita exige schema_version = 1 e catalog_version = \"1\"",
            ));
        }
        Ok(recipe)
    }
}
