use crate::{Error, config::*};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "boilerplate",
    version,
    about = "Planejamento de projetos web; geração ainda em desenvolvimento"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Consulta as stacks planejadas (não são templates geráveis ainda).
    Templates {
        #[command(subcommand)]
        command: Templates,
    },
    /// Valida escolhas e imprime um plano, sem criar arquivos ou executar ferramentas.
    Plan(Box<PlanArgs>),
    /// Verifica executáveis no PATH, sem consultar daemons ou instalar ferramentas.
    Doctor {
        #[arg(long)]
        template: String,
        /// Também verifica ferramentas de infraestrutura selecionadas na receita.
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum Templates {
    List {
        #[arg(long)]
        json: bool,
    },
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Args, Default)]
pub struct PlanArgs {
    pub name: Option<String>,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long = "type", value_enum)]
    pub kind: Option<ProjectType>,
    #[arg(long)]
    pub destination: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub layout: Option<Layout>,
    #[arg(long)]
    pub web_dir: Option<PathBuf>,
    #[arg(long)]
    pub api_dir: Option<PathBuf>,
    #[arg(long)]
    pub frontend: Option<String>,
    #[arg(long)]
    pub backend: Option<String>,
    #[arg(long, value_enum)]
    pub database: Option<Database>,
    #[arg(long, value_enum)]
    pub database_location: Option<Location>,
    #[arg(long, value_enum)]
    pub cache: Option<Cache>,
    #[arg(long, value_enum)]
    pub cache_location: Option<Location>,
    #[arg(long, value_enum)]
    pub containers: Option<Containers>,
    #[arg(long, value_enum)]
    pub kubernetes: Option<Kubernetes>,
    #[arg(long, value_enum)]
    pub monitoring: Option<Monitoring>,
    #[arg(long, value_enum)]
    pub secrets: Option<Secrets>,
    #[arg(long, value_enum)]
    pub vault_location: Option<Location>,
    #[arg(long)]
    pub namespace: Option<String>,
    #[arg(long)]
    pub image: Option<String>,
    #[arg(long)]
    pub image_web: Option<String>,
    #[arg(long)]
    pub image_api: Option<String>,
    /// O planejador sempre é não interativo; flag aceita para scripts.
    #[arg(long)]
    pub non_interactive: bool,
    #[arg(long)]
    pub json: bool,
}

impl PlanArgs {
    pub fn recipe(&self) -> Result<Recipe, Error> {
        let mut r = if let Some(path) = &self.config {
            Recipe::load(path)?
        } else {
            Recipe::default()
        };
        macro_rules! apply {
            ($target:expr, $value:expr) => {
                if let Some(value) = &$value {
                    $target = Some(value.clone());
                }
            };
        }
        apply!(r.project.name, self.name);
        apply!(r.project.kind, self.kind);
        apply!(r.project.destination, self.destination);
        apply!(r.project.layout, self.layout);
        apply!(r.project.web_dir, self.web_dir);
        apply!(r.project.api_dir, self.api_dir);
        apply!(r.frontend.template, self.frontend);
        apply!(r.backend.template, self.backend);
        apply!(r.backend.database, self.database);
        apply!(r.backend.database_location, self.database_location);
        apply!(r.backend.cache, self.cache);
        apply!(r.backend.cache_location, self.cache_location);
        apply!(r.infrastructure.containers, self.containers);
        apply!(r.infrastructure.kubernetes, self.kubernetes);
        apply!(r.infrastructure.monitoring, self.monitoring);
        apply!(r.infrastructure.secrets, self.secrets);
        apply!(r.infrastructure.vault_location, self.vault_location);
        apply!(r.infrastructure.namespace, self.namespace);
        apply!(r.infrastructure.image, self.image);
        apply!(r.infrastructure.image_web, self.image_web);
        apply!(r.infrastructure.image_api, self.image_api);
        // Desativar uma opção por flag também remove a localização herdada.
        if self.database == Some(Database::None) && self.database_location.is_none() {
            r.backend.database_location = None;
        }
        if self.cache == Some(Cache::None) && self.cache_location.is_none() {
            r.backend.cache_location = None;
        }
        if self.secrets == Some(Secrets::Env) && self.vault_location.is_none() {
            r.infrastructure.vault_location = None;
        }
        r.schema_version = Some(1);
        r.catalog_version = Some(crate::catalog::VERSION.to_string());
        Ok(r)
    }
}
