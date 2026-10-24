use crate::{
    Error,
    catalog::{Catalog, Template},
    config::*,
};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Application {
    pub template: String,
    pub language: String,
    pub destination: PathBuf,
    pub install_command: Vec<String>,
    pub planned_paths: Vec<PathBuf>,
}

#[derive(Debug, Serialize)]
pub struct Plan {
    pub catalog_version: &'static str,
    pub status: &'static str,
    pub configuration: Recipe,
    pub applications: Vec<Application>,
    pub integrations: Vec<String>,
    pub omitted: Vec<String>,
    pub infrastructure_paths: Vec<PathBuf>,
    pub notes: Vec<String>,
}

fn require(condition: bool, message: &str) -> Result<(), Error> {
    if condition {
        Ok(())
    } else {
        Err(Error::input(message))
    }
}

fn destination(path: &Path, cwd: &Path) -> Result<PathBuf, Error> {
    require(!path.as_os_str().is_empty(), "Destino não pode ser vazio")?;
    require(
        !path.components().any(|c| matches!(c, Component::ParentDir)),
        "Destino não aceita componentes '..'",
    )?;
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    match std::fs::symlink_metadata(&path) {
        Ok(_) => {
            return Err(Error::input(format!(
                "Destino já existe (inclusive symlinks): {}",
                path.display()
            )));
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(Error::operation(format!(
                "Não foi possível verificar o destino: {e}"
            )));
        }
    }
    // Resolve ancestrais existentes para detectar sobreposição via symlinks.
    let mut ancestor = path.as_path();
    let mut suffix = Vec::new();
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                suffix.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| Error::input("Destino inválido"))?
                        .to_os_string(),
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| Error::input("Destino inválido"))?;
            }
            Err(e) => {
                return Err(Error::operation(format!(
                    "Falha ao verificar ancestral: {e}"
                )));
            }
        }
    }
    let mut resolved = ancestor
        .canonicalize()
        .map_err(|e| Error::input(format!("Ancestral inacessível ou symlink inválido: {e}")))?;
    require(
        resolved.is_dir(),
        "Ancestral do destino precisa ser um diretório",
    )?;
    for part in suffix.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn template<'a>(
    catalog: &'a Catalog,
    id: &Option<String>,
    role: ProjectType,
) -> Result<&'a Template, Error> {
    let id = id
        .as_deref()
        .ok_or_else(|| Error::input(format!("Informe o template de {role:?}")))?;
    let t = catalog.get(id)?;
    require(
        t.role == role,
        "Template incompatível com o papel frontend/backend",
    )?;
    Ok(t)
}

fn location(enabled: bool, value: Option<Location>, label: &str) -> Result<(), Error> {
    if enabled && value.is_none() {
        return Err(Error::input(format!("Informe {label}: local ou external")));
    }
    if !enabled && value.is_some() {
        return Err(Error::input(format!(
            "{label} não se aplica a uma opção desativada"
        )));
    }
    Ok(())
}

pub fn build(mut r: Recipe, catalog: &Catalog, cwd: &Path) -> Result<Plan, Error> {
    require(
        crate::generator::runtime::valid_node_version(&r.toolchain.node_version),
        "Versão Node deve ser numérica (22.12+ ou major 22/24+); aliases e comandos não são aceitos",
    )?;
    let kind = r
        .project
        .kind
        .ok_or_else(|| Error::input("Informe --type frontend, backend ou fullstack"))?;
    let name = r
        .project
        .name
        .as_deref()
        .ok_or_else(|| Error::input("Informe o nome do projeto"))?;
    require(
        !name.is_empty()
            && name.len() <= 63
            && name.as_bytes()[0].is_ascii_lowercase()
            && name
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            && !name.ends_with('-'),
        "Nome deve ter até 63 caracteres: começar com letra minúscula, conter a-z/0-9/hífen e não terminar com hífen",
    )?;
    let full = kind == ProjectType::Fullstack;
    let has_web = kind != ProjectType::Backend;
    let has_api = kind != ProjectType::Frontend;
    require(
        full || r.project.layout.is_none(),
        "--layout só se aplica a fullstack",
    )?;
    require(
        !full || r.project.layout.is_some(),
        "Fullstack exige --layout monorepo ou split",
    )?;
    let split = r.project.layout == Some(Layout::Split);
    require(
        split || (r.project.web_dir.is_none() && r.project.api_dir.is_none()),
        "--web-dir e --api-dir exigem layout split",
    )?;
    require(
        !split || r.project.destination.is_none(),
        "Layout split usa --web-dir/--api-dir, não --destination",
    )?;
    require(
        has_web || r.frontend.template.is_none(),
        "Backend não aceita --frontend",
    )?;
    require(
        has_api || r.backend.template.is_none(),
        "Frontend não aceita --backend",
    )?;

    let database = r.backend.database.unwrap_or(Database::None);
    let cache = r.backend.cache.unwrap_or(Cache::None);
    let containers = r.infrastructure.containers.unwrap_or(Containers::None);
    let kubernetes = r.infrastructure.kubernetes.unwrap_or(Kubernetes::None);
    let monitoring = r.infrastructure.monitoring.unwrap_or(Monitoring::None);
    let secrets = r.infrastructure.secrets.unwrap_or(Secrets::Env);
    require(
        has_api || (database == Database::None && cache == Cache::None && secrets == Secrets::Env),
        "Banco, cache e Vault exigem backend; não são configurações do navegador",
    )?;
    location(
        database != Database::None,
        r.backend.database_location,
        "--database-location",
    )?;
    location(
        cache != Cache::None,
        r.backend.cache_location,
        "--cache-location",
    )?;
    location(
        secrets == Secrets::Vault,
        r.infrastructure.vault_location,
        "--vault-location",
    )?;
    let infrastructure_local =
        containers == Containers::Compose || kubernetes == Kubernetes::Manifests;
    for (enabled, loc, label) in [
        (
            database != Database::None,
            r.backend.database_location,
            "banco",
        ),
        (cache != Cache::None, r.backend.cache_location, "cache"),
        (
            secrets == Secrets::Vault,
            r.infrastructure.vault_location,
            "Vault",
        ),
    ] {
        require(
            !(enabled && loc == Some(Location::Local)) || infrastructure_local,
            &format!(
                "{label} local exige Compose ou manifests; use external para uma instância já instalada"
            ),
        )?;
    }
    let images = [
        &r.infrastructure.image,
        &r.infrastructure.image_web,
        &r.infrastructure.image_api,
    ];
    require(
        kubernetes == Kubernetes::Manifests
            || (images.iter().all(|v| v.is_none()) && r.infrastructure.namespace.is_none()),
        "Imagens e namespace exigem --kubernetes manifests",
    )?;
    if kubernetes == Kubernetes::Manifests {
        require(
            !full || r.infrastructure.image.is_none(),
            "Fullstack usa --image-web e --image-api",
        )?;
        require(
            full || (r.infrastructure.image_web.is_none() && r.infrastructure.image_api.is_none()),
            "Projeto individual usa --image",
        )?;
        for image in images.into_iter().flatten() {
            require(
                !image.is_empty()
                    && !image.contains(char::is_whitespace)
                    && !image.contains("://")
                    && !image.contains('@')
                    && image.rsplit('/').next().is_some_and(|last| {
                        last.contains(':') && !last.ends_with(":latest") && !last.ends_with(':')
                    }),
                "Imagem deve ter uma tag explícita diferente de latest, sem URL, credenciais ou espaços",
            )?;
        }
        let explicit = if full {
            r.infrastructure.image_web.is_some() && r.infrastructure.image_api.is_some()
        } else {
            r.infrastructure.image.is_some()
        };
        require(
            containers == Containers::Compose || explicit,
            "Kubernetes sem Dockerfile exige referência de imagem para cada aplicação",
        )?;
        let namespace = r
            .infrastructure
            .namespace
            .get_or_insert_with(|| name.to_string());
        require(
            !namespace.is_empty()
                && namespace.len() <= 63
                && namespace
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
                && !namespace.starts_with('-')
                && !namespace.ends_with('-'),
            "Namespace Kubernetes inválido",
        )?;
    }
    r.backend.database = Some(database);
    r.backend.cache = Some(cache);
    r.infrastructure.containers = Some(containers);
    r.infrastructure.kubernetes = Some(kubernetes);
    r.infrastructure.monitoring = Some(monitoring);
    r.infrastructure.secrets = Some(secrets);
    r.schema_version = Some(1);
    r.catalog_version = Some(crate::catalog::VERSION.to_string());

    let root = if split {
        None
    } else {
        Some(destination(
            r.project
                .destination
                .as_deref()
                .unwrap_or_else(|| Path::new(name)),
            cwd,
        )?)
    };
    let mut selected = Vec::new();
    if has_web {
        let dir = if split {
            destination(
                r.project
                    .web_dir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("")),
                cwd,
            )?
        } else if full {
            root.as_ref().unwrap().join("apps/web")
        } else {
            root.clone().unwrap()
        };
        selected.push((
            template(catalog, &r.frontend.template, ProjectType::Frontend)?,
            dir,
        ));
    }
    if has_api {
        let dir = if split {
            destination(
                r.project
                    .api_dir
                    .as_deref()
                    .unwrap_or_else(|| Path::new("")),
                cwd,
            )?
        } else if full {
            root.as_ref().unwrap().join("apps/api")
        } else {
            root.clone().unwrap()
        };
        selected.push((
            template(catalog, &r.backend.template, ProjectType::Backend)?,
            dir,
        ));
    }
    for (t, _) in &selected {
        let system = if r.toolchain.module_system == ModuleSystem::Esm {
            "esm"
        } else {
            "commonjs"
        };
        require(
            t.module_systems.is_empty() || t.module_systems.iter().any(|s| s == system),
            &format!("{} não suporta módulos {system}", t.id),
        )?;
    }
    if split {
        require(
            !selected[0].1.starts_with(&selected[1].1)
                && !selected[1].1.starts_with(&selected[0].1),
            "Destinos web/api devem ser independentes e não podem se sobrepor",
        )?;
    }
    let mut integrations = Vec::new();
    let mut omitted = Vec::new();
    for (key, enabled) in [
        ("postgres", database == Database::Postgres),
        ("mongodb", database == Database::Mongodb),
        ("redis", cache == Cache::Redis),
        ("compose", containers == Containers::Compose),
        ("kubernetes", kubernetes == Kubernetes::Manifests),
        (
            "prometheus-grafana",
            monitoring == Monitoring::PrometheusGrafana,
        ),
        ("vault", secrets == Secrets::Vault),
    ] {
        if enabled {
            integrations.push(key.to_string());
        } else {
            omitted.push(key.to_string());
        }
    }
    for capability in &integrations {
        let targets: Vec<_> = match capability.as_str() {
            "postgres" | "mongodb" | "redis" | "vault" => selected
                .iter()
                .filter(|(t, _)| t.role == ProjectType::Backend)
                .collect(),
            "prometheus-grafana" => selected
                .iter()
                .filter(|(t, _)| t.role == ProjectType::Backend || !has_api)
                .collect(),
            _ => selected.iter().collect(),
        };
        for (t, _) in targets {
            // Frontend estático só recebe monitoria via servidor de container.
            let static_metrics = capability == "prometheus-grafana"
                && t.role == ProjectType::Frontend
                && containers == Containers::Compose
                && t.capabilities
                    .iter()
                    .any(|c| c == "prometheus-grafana-container");
            require(
                t.capabilities.contains(capability) || static_metrics,
                &format!("{} não declara capacidade planejada {capability}", t.id),
            )?;
        }
    }
    let mut infrastructure_paths = Vec::new();
    let infra_roots: Vec<_> = if let Some(root) = &root {
        vec![root.clone()]
    } else {
        selected.iter().map(|(_, p)| p.clone()).collect()
    };
    for dir in infra_roots {
        if containers == Containers::Compose {
            infrastructure_paths.push(dir.join("compose.yaml"));
        }
        if kubernetes == Kubernetes::Manifests {
            infrastructure_paths.push(dir.join("infra/kubernetes/"));
        }
        if monitoring == Monitoring::PrometheusGrafana {
            infrastructure_paths.push(dir.join("infra/monitoring/"));
        }
        if secrets == Secrets::Vault
            && (!split
                || selected
                    .iter()
                    .any(|(t, p)| p == &dir && t.role == ProjectType::Backend))
        {
            infrastructure_paths.push(dir.join("infra/vault/"));
        }
    }
    let applications = selected
        .into_iter()
        .map(|(t, dir)| {
            let mut paths: Vec<_> = t.paths.iter().map(|p| dir.join(p)).collect();
            for integration in &integrations {
                if let Some(extra) = t.conditional_paths.get(integration) {
                    paths.extend(extra.iter().map(|p| dir.join(p)));
                }
            }
            paths.sort();
            paths.dedup();
            Application {
                template: t.id.clone(),
                language: t.language.clone(),
                destination: dir,
                install_command: t.install.clone(),
                planned_paths: paths,
            }
        })
        .collect();
    Ok(Plan {
        catalog_version: crate::catalog::VERSION,
        status: "planning-only",
        configuration: r,
        applications,
        integrations,
        omitted,
        infrastructure_paths,
        notes: vec![
            "Este comando apenas planeja; new gera bases e Compose, e rejeita adapters avançados ainda não implementados.".into(),
            "Versões de dependências das bases estão nos templates; drivers e serviços avançados ainda dependem de adapters.".into(),
            "new usa API local na porta 3000, frontend em 5173 (Next em 3001) e CORS correspondente.".into(),
            "Nenhuma instalação, container, daemon ou cluster foi executado/consultado pelo planejador.".into(),
        ],
    })
}
