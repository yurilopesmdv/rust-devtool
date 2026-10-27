use crate::{
    Error,
    catalog::{Catalog, Status},
    cli::NewArgs,
    config::*,
};
use std::io::{self, IsTerminal, Write};

pub fn interactive(args: &NewArgs) -> bool {
    !args.plan.non_interactive
        && !args.plan.json
        && io::stdin().is_terminal()
        && io::stdout().is_terminal()
}

pub fn line(label: &str, default: Option<&str>) -> Result<String, Error> {
    loop {
        print!(
            "{label}{}: ",
            default.map(|d| format!(" [{d}]")).unwrap_or_default()
        );
        io::stdout()
            .flush()
            .map_err(|e| Error::operation(e.to_string()))?;
        let mut input = String::new();
        if io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::operation(e.to_string()))?
            == 0
        {
            return Err(Error {
                message: "Entrada encerrada; operação cancelada".into(),
                code: 130,
            });
        }
        let input = input.trim();
        if !input.is_empty() {
            return Ok(input.to_string());
        }
        if let Some(default) = default {
            return Ok(default.to_string());
        }
    }
}

pub fn decision(label: &str, default: bool) -> Result<bool, Error> {
    loop {
        let value = line(
            &format!("{label} (s/n)"),
            Some(if default { "s" } else { "n" }),
        )?;
        match value.to_lowercase().as_str() {
            "s" | "sim" | "y" | "yes" => return Ok(true),
            "n" | "nao" | "não" | "no" => return Ok(false),
            _ => println!("Responda s ou n"),
        }
    }
}

fn choose(label: &str, options: &[String]) -> Result<String, Error> {
    println!("{label}");
    for (index, value) in options.iter().enumerate() {
        println!("  {}. {value}", index + 1);
    }
    loop {
        let value = line("Escolha", None)?;
        if let Ok(index) = value.parse::<usize>()
            && let Some(value) = index.checked_sub(1).and_then(|i| options.get(i))
        {
            return Ok(value.clone());
        }
        if options.contains(&value) {
            return Ok(value);
        }
        println!("Escolha uma opção da lista");
    }
}

pub fn complete(args: &NewArgs, catalog: &Catalog) -> Result<Recipe, Error> {
    let mut r = args.plan.recipe()?;
    if !interactive(args) {
        return Ok(r);
    }
    if r.project.name.is_none() {
        r.project.name = Some(line("Nome do projeto", None)?);
    }
    if r.project.kind.is_none() {
        let kind = choose(
            "Tipo de projeto",
            &["frontend".into(), "backend".into(), "fullstack".into()],
        )?;
        r.project.kind = Some(match kind.as_str() {
            "frontend" => ProjectType::Frontend,
            "backend" => ProjectType::Backend,
            _ => ProjectType::Fullstack,
        });
    }
    if r.project.kind == Some(ProjectType::Fullstack) && r.project.layout.is_none() {
        let value = choose(
            "Organização dos repositórios",
            &["monorepo".into(), "split".into()],
        )?;
        r.project.layout = Some(if value == "split" {
            Layout::Split
        } else {
            Layout::Monorepo
        });
    }
    let name = r.project.name.as_deref().unwrap();
    if r.project.layout == Some(Layout::Split) {
        if r.project.web_dir.is_none() {
            r.project.web_dir = Some(
                line(
                    "Destino do frontend",
                    Some(
                        &crate::planner::default_destination(&format!("{name}-web"))?
                            .to_string_lossy(),
                    ),
                )?
                .into(),
            );
        }
        if r.project.api_dir.is_none() {
            r.project.api_dir = Some(
                line(
                    "Destino do backend",
                    Some(
                        &crate::planner::default_destination(&format!("{name}-api"))?
                            .to_string_lossy(),
                    ),
                )?
                .into(),
            );
        }
    } else if r.project.destination.is_none() {
        r.project.destination = Some(
            line(
                "Caminho de destino (diretório novo)",
                Some(&crate::planner::default_destination(name)?.to_string_lossy()),
            )?
            .into(),
        );
    }
    for (role, selected) in [
        (ProjectType::Frontend, &mut r.frontend.template),
        (ProjectType::Backend, &mut r.backend.template),
    ] {
        if (r.project.kind == Some(role) || r.project.kind == Some(ProjectType::Fullstack))
            && selected.is_none()
        {
            let options: Vec<_> = catalog
                .templates
                .iter()
                .filter(|t| t.role == role && t.status == Status::Ready)
                .map(|t| t.id.clone())
                .collect();
            *selected = Some(choose(&format!("Stack {role:?}"), &options)?);
        }
    }
    let selected = [&r.frontend.template, &r.backend.template]
        .into_iter()
        .flatten()
        .map(|id| catalog.get(id))
        .collect::<Result<Vec<_>, _>>()?;
    let backend = selected.iter().find(|t| t.role == ProjectType::Backend);
    let supports_backend =
        |capability: &str| backend.is_some_and(|t| t.capabilities.iter().any(|c| c == capability));
    let supports_all = |capability: &str| {
        !selected.is_empty()
            && selected
                .iter()
                .all(|t| t.capabilities.iter().any(|c| c == capability))
    };
    let uses_node = [&r.frontend.template, &r.backend.template]
        .into_iter()
        .flatten()
        .any(|id| {
            catalog
                .get(id)
                .is_ok_and(|t| t.requirements.iter().any(|r| r.command == "node"))
        });
    if uses_node && args.plan.node_version.is_none() {
        r.toolchain.node_version = line(
            "Versão Node (major ou versão exata)",
            Some(&r.toolchain.node_version),
        )?;
    }
    if uses_node && args.plan.module_system.is_none() {
        let commonjs_supported = [&r.frontend.template, &r.backend.template]
            .into_iter()
            .flatten()
            .all(|id| {
                catalog.get(id).is_ok_and(|t| {
                    t.module_systems.is_empty() || t.module_systems.iter().any(|s| s == "commonjs")
                })
            });
        if commonjs_supported && args.plan.config.is_none() {
            loop {
                match line("Sistema de módulos (esm/commonjs)", Some("esm"))?.as_str() {
                    "esm" => {
                        r.toolchain.module_system = ModuleSystem::Esm;
                        break;
                    }
                    "commonjs" => {
                        r.toolchain.module_system = ModuleSystem::Commonjs;
                        break;
                    }
                    _ => println!("Escolha esm ou commonjs"),
                }
            }
        } else {
            println!(
                "Sistema de módulos: {:?} (fontes com import/export)",
                r.toolchain.module_system
            );
        }
    }
    if r.infrastructure.containers.is_none() {
        r.infrastructure.containers = Some(
            if supports_all("compose")
                && decision("Gerar Dockerfile, .dockerignore e Compose?", false)?
            {
                Containers::Compose
            } else {
                Containers::None
            },
        );
    }

    if r.infrastructure.kubernetes.is_none() {
        r.infrastructure.kubernetes = Some(
            if supports_all("kubernetes") && decision("Gerar manifests Kubernetes?", false)? {
                Kubernetes::Manifests
            } else {
                Kubernetes::None
            },
        );
    }
    let local_infra = r.infrastructure.containers == Some(Containers::Compose)
        || r.infrastructure.kubernetes == Some(Kubernetes::Manifests);
    if r.project.kind != Some(ProjectType::Frontend) {
        if r.backend.database.is_none() {
            let mut options = vec!["none".to_string()];
            options.extend(
                ["postgres", "mongodb"]
                    .into_iter()
                    .filter(|cap| supports_backend(cap))
                    .map(str::to_string),
            );
            let value = if options.len() == 1 {
                "none".to_string()
            } else {
                choose("Banco de dados", &options)?
            };
            r.backend.database = Some(match value.as_str() {
                "postgres" => Database::Postgres,
                "mongodb" => Database::Mongodb,
                _ => Database::None,
            });
        }
        if r.backend.database != Some(Database::None) && r.backend.database_location.is_none() {
            r.backend.database_location =
                Some(service_location("Localização do banco", local_infra)?);
        }
        if r.backend.database == Some(Database::Postgres)
            && r.backend.migrations.is_none()
            && supports_backend("migrations-dbmate")
        {
            r.backend.migrations = Some(
                if decision(
                    "Adicionar migrations PostgreSQL versionadas (dbmate)?",
                    false,
                )? {
                    Migrations::Dbmate
                } else {
                    Migrations::None
                },
            );
        }
        if r.backend.database == Some(Database::Mongodb)
            && r.backend.migrations.is_none()
            && supports_backend("migrations-mongodb-native")
        {
            r.backend.migrations = Some(
                if decision(
                    "Adicionar migrations MongoDB nativas com checkpoints?",
                    false,
                )? {
                    Migrations::MongodbNative
                } else {
                    Migrations::None
                },
            );
        }
        if r.backend.cache.is_none() {
            r.backend.cache = Some(
                if supports_backend("redis") && decision("Usar Redis como cache opcional?", false)?
                {
                    Cache::Redis
                } else {
                    Cache::None
                },
            );
        }
        if r.backend.cache == Some(Cache::Redis) && r.backend.cache_location.is_none() {
            r.backend.cache_location = Some(service_location("Localização do Redis", local_infra)?);
        }
        if r.infrastructure.secrets.is_none() {
            r.infrastructure.secrets = Some(
                if supports_backend("vault")
                    && decision(
                        "Integrar Vault para entrega de segredos por arquivos?",
                        false,
                    )?
                {
                    Secrets::Vault
                } else {
                    Secrets::Env
                },
            );
        }
        if r.infrastructure.secrets == Some(Secrets::Vault)
            && r.infrastructure.vault_location.is_none()
        {
            r.infrastructure.vault_location =
                Some(service_location("Localização do Vault", local_infra)?);
        }
        if r.infrastructure.secrets == Some(Secrets::Vault)
            && r.infrastructure.vault_address.is_none()
            && r.infrastructure.vault_location == Some(Location::External)
        {
            r.infrastructure.vault_address = Some(line(
                "URL do Vault acessível aos Agents (sem credenciais)",
                None,
            )?);
        }
        if r.infrastructure.frontend_origin.is_none() {
            let port = if r.frontend.template.as_deref() == Some("next-ts") {
                3001
            } else {
                5173
            };
            r.infrastructure.frontend_origin = Some(line(
                "Origem permitida do frontend (CORS)",
                Some(&format!("http://localhost:{port}")),
            )?);
        }
    }
    if r.frontend.template.is_some() && r.infrastructure.api_url.is_none() {
        r.infrastructure.api_url = Some(line(
            "URL da API acessível ao navegador",
            Some("http://localhost:3000"),
        )?);
    }
    if r.infrastructure.monitoring.is_none() {
        let supported = if backend.is_some() {
            supports_backend("prometheus-grafana")
        } else {
            !selected.is_empty()
                && selected.iter().all(|t| {
                    t.capabilities.iter().any(|c| {
                        c == "prometheus-grafana"
                            || (local_infra && c == "prometheus-grafana-container")
                    })
                })
        };
        r.infrastructure.monitoring = Some(
            if supported && decision("Adicionar Prometheus e Grafana?", false)? {
                Monitoring::PrometheusGrafana
            } else {
                Monitoring::None
            },
        );
    }
    if r.infrastructure.kubernetes == Some(Kubernetes::Manifests) {
        if r.infrastructure.namespace.is_none() {
            r.infrastructure.namespace =
                Some(line("Namespace Kubernetes", r.project.name.as_deref())?);
        }
        if r.infrastructure.containers != Some(Containers::Compose) {
            if r.project.kind == Some(ProjectType::Fullstack) {
                if r.infrastructure.image_web.is_none() {
                    r.infrastructure.image_web = Some(line("Imagem frontend com tag", None)?);
                }
                if r.infrastructure.image_api.is_none() {
                    r.infrastructure.image_api = Some(line("Imagem backend com tag", None)?);
                }
            } else if r.infrastructure.image.is_none() {
                r.infrastructure.image = Some(line("Imagem da aplicação com tag", None)?);
            }
        }
    }
    if r.infrastructure.ci.is_none() && supports_all("github-actions") {
        r.infrastructure.ci = Some(
            if decision("Gerar CI GitHub Actions dos projetos (sem deploy)?", false)? {
                Ci::GithubActions
            } else {
                Ci::None
            },
        );
    }
    Ok(r)
}
fn service_location(label: &str, local: bool) -> Result<Location, Error> {
    loop {
        match line(
            &format!("{label} (local/external)"),
            Some(if local { "local" } else { "external" }),
        )?
        .as_str()
        {
            "local" => return Ok(Location::Local),
            "external" => return Ok(Location::External),
            _ => println!("Escolha local ou external"),
        }
    }
}
