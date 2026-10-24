use clap::Parser;
use rust_boilerplate::{
    catalog::Catalog,
    cli::{Cli, Command, PlanArgs},
    config::*,
    doctor, planner,
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "boilerplate-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn recipe() -> Recipe {
    let mut r = Recipe::default();
    r.project.name = Some("minha-api".into());
    r.project.destination = Some("minha-api".into());
    r.project.kind = Some(ProjectType::Backend);
    r.backend.template = Some("express-ts".into());
    r
}
fn plan(r: Recipe, root: &Path) -> Result<planner::Plan, rust_boilerplate::Error> {
    planner::build(r, &Catalog::embedded().unwrap(), root)
}

#[test]
fn minimal_plan_has_no_optional_services_or_writes() {
    let temp = Temp::new();
    let p = plan(recipe(), &temp.0).unwrap();
    assert!(p.integrations.is_empty());
    assert!(p.infrastructure_paths.is_empty());
    assert_eq!(p.status, "planning-only");
    let paths = format!("{:?}", p.applications[0].planned_paths);
    for fragment in [
        "repositories",
        "database",
        "cache",
        "secrets",
        "observability",
        "Dockerfile",
    ] {
        assert!(!paths.contains(fragment), "{fragment} inesperado");
    }
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
}

#[test]
fn all_nine_fullstack_pairs_are_plannable() {
    let temp = Temp::new();
    for frontend in ["vanilla-js", "react-vite-ts", "next-ts"] {
        for backend in ["express-ts", "nest-ts", "rust-axum"] {
            let mut r = recipe();
            r.project.kind = Some(ProjectType::Fullstack);
            r.project.layout = Some(Layout::Monorepo);
            r.frontend.template = Some(frontend.into());
            r.backend.template = Some(backend.into());
            let p = plan(r, &temp.0).unwrap();
            assert_eq!(p.applications.len(), 2);
            assert!(p.applications[0].destination.ends_with("apps/web"));
            assert!(p.applications[1].destination.ends_with("apps/api"));
        }
    }
}

#[test]
fn missing_layout_or_wrong_template_role_is_rejected() {
    let temp = Temp::new();
    let mut r = recipe();
    r.backend.template = Some("react-vite-ts".into());
    assert_eq!(plan(r, &temp.0).unwrap_err().code, 2);
    let mut r = recipe();
    r.project.kind = Some(ProjectType::Fullstack);
    assert!(plan(r, &temp.0).unwrap_err().message.contains("--layout"));
}

#[test]
fn frontend_rejects_backend_integrations() {
    let temp = Temp::new();
    let mut r = recipe();
    r.project.kind = Some(ProjectType::Frontend);
    r.backend.template = None;
    r.frontend.template = Some("react-vite-ts".into());
    r.backend.database = Some(Database::Postgres);
    assert!(plan(r, &temp.0).is_err());
}

#[test]
fn service_location_is_required_and_cannot_be_silently_enabled() {
    let temp = Temp::new();
    let mut r = recipe();
    r.backend.database = Some(Database::Postgres);
    assert!(plan(r.clone(), &temp.0).is_err());
    r.backend.database_location = Some(Location::Local);
    assert!(plan(r.clone(), &temp.0).is_err());
    r.backend.database_location = Some(Location::External);
    let p = plan(r, &temp.0).unwrap();
    assert_eq!(p.integrations, ["postgres"]);
    assert!(p.infrastructure_paths.is_empty());
}

#[test]
fn complete_configuration_adds_only_selected_integration_paths() {
    let temp = Temp::new();
    let mut r = recipe();
    r.backend.database = Some(Database::Mongodb);
    r.backend.database_location = Some(Location::Local);
    r.backend.cache = Some(Cache::Redis);
    r.backend.cache_location = Some(Location::Local);
    r.infrastructure.containers = Some(Containers::Compose);
    r.infrastructure.kubernetes = Some(Kubernetes::Manifests);
    r.infrastructure.monitoring = Some(Monitoring::PrometheusGrafana);
    r.infrastructure.secrets = Some(Secrets::Vault);
    r.infrastructure.vault_location = Some(Location::Local);
    let p = plan(r, &temp.0).unwrap();
    assert!(p.omitted.contains(&"postgres".into()));
    assert!(
        p.applications[0]
            .planned_paths
            .iter()
            .any(|p| p.ends_with("repositories"))
    );
    assert_eq!(p.infrastructure_paths.len(), 4);
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
}

#[test]
fn kubernetes_requires_image_or_dockerfile_and_checks_namespace() {
    let temp = Temp::new();
    let mut r = recipe();
    r.infrastructure.kubernetes = Some(Kubernetes::Manifests);
    assert!(plan(r.clone(), &temp.0).is_err());
    r.infrastructure.image = Some("example/api:latest".into());
    assert!(plan(r.clone(), &temp.0).is_err());
    r.infrastructure.image = Some("example/api:1.0".into());
    assert!(plan(r.clone(), &temp.0).is_ok());
    r.infrastructure.namespace = Some("INVALID".into());
    assert!(plan(r, &temp.0).is_err());
}

#[test]
fn existing_and_overlapping_destinations_are_rejected_without_touching_content() {
    let temp = Temp::new();
    std::fs::create_dir(temp.0.join("minha-api")).unwrap();
    std::fs::write(temp.0.join("minha-api/keep"), "preservar").unwrap();
    assert!(plan(recipe(), &temp.0).is_err());
    assert_eq!(
        std::fs::read_to_string(temp.0.join("minha-api/keep")).unwrap(),
        "preservar"
    );
    let mut r = recipe();
    r.project.kind = Some(ProjectType::Fullstack);
    r.project.layout = Some(Layout::Split);
    r.project.destination = None;
    r.project.web_dir = Some("novo".into());
    r.project.api_dir = Some("novo/api".into());
    r.frontend.template = Some("react-vite-ts".into());
    assert!(plan(r, &temp.0).unwrap_err().message.contains("sobrepor"));
}

#[cfg(unix)]
#[test]
fn dangling_destination_symlinks_and_overlapping_ancestor_symlinks_are_rejected() {
    use std::os::unix::fs::symlink;
    let temp = Temp::new();
    symlink(temp.0.join("missing"), temp.0.join("minha-api")).unwrap();
    assert!(plan(recipe(), &temp.0).is_err());
    std::fs::create_dir(temp.0.join("actual")).unwrap();
    symlink(temp.0.join("actual"), temp.0.join("alias")).unwrap();
    let mut r = recipe();
    r.project.kind = Some(ProjectType::Fullstack);
    r.project.layout = Some(Layout::Split);
    r.project.destination = None;
    r.project.web_dir = Some("actual/new".into());
    r.project.api_dir = Some("alias/new/api".into());
    r.frontend.template = Some("react-vite-ts".into());
    assert!(plan(r, &temp.0).unwrap_err().message.contains("sobrepor"));
}

#[test]
fn split_destinations_can_be_independent() {
    let temp = Temp::new();
    let mut r = recipe();
    r.project.kind = Some(ProjectType::Fullstack);
    r.project.layout = Some(Layout::Split);
    r.project.destination = None;
    r.project.web_dir = Some("web".into());
    r.project.api_dir = Some("api".into());
    r.frontend.template = Some("react-vite-ts".into());
    assert_eq!(plan(r, &temp.0).unwrap().applications.len(), 2);
}

#[test]
fn invalid_names_and_parent_traversal_are_rejected() {
    let temp = Temp::new();
    for name in ["", "../api", "API", "api space", "api-", "$(touch hacked)"] {
        let mut r = recipe();
        r.project.name = Some(name.into());
        assert!(plan(r, &temp.0).is_err(), "{name}");
    }
    let mut r = recipe();
    r.project.destination = Some("../escape".into());
    assert!(plan(r, &temp.0).is_err());
}

#[test]
fn recipes_reject_unknown_fields_invalid_values_and_versions() {
    for text in [
        "schema_version = 2\ncatalog_version = '1'",
        "schema_version = 1\ncatalog_version = 'future'",
        "schema_version = 1\ncatalog_version = '1'\npassword = 'x'",
        "schema_version = 1\ncatalog_version = '1'\n[backend]\ndatabase = 'mysql'",
        "schema_version = 1\ncatalog_version = '1'\n[project]\nnamme = 'typo'",
    ] {
        assert!(Recipe::parse(text).is_err());
    }
}

#[test]
fn flags_override_recipes_and_none_clears_inherited_location() {
    let temp = Temp::new();
    let file = temp.0.join("recipe.toml");
    std::fs::write(&file, "schema_version = 1\ncatalog_version = '1'\n[project]\ntype = 'backend'\nname = 'old'\n[backend]\ntemplate = 'express-ts'\ndatabase = 'postgres'\ndatabase_location = 'external'").unwrap();
    let args = PlanArgs {
        name: Some("new".into()),
        config: Some(file),
        database: Some(Database::None),
        ..Default::default()
    };
    let r = args.recipe().unwrap();
    assert_eq!(r.project.name.as_deref(), Some("new"));
    assert_eq!(r.backend.database_location, None);
    assert!(plan(r, &temp.0).unwrap().integrations.is_empty());
}

#[test]
fn new_language_uses_metadata_and_its_own_integration_paths_without_cli_changes() {
    let temp = Temp::new();
    let source = r#"
id = "python-fastapi"
role = "backend"
language = "Python"
framework = "FastAPI"
architecture = "layered"
status = "planned"
package_manager = "uv"
install = ["uv", "sync", "--locked"]
paths = ["app/controllers/", "app/services/"]
capabilities = ["postgres"]
[conditional_paths]
postgres = ["app/adapters/postgres/"]
"#;
    let catalog = Catalog::parse(&[source]).unwrap();
    let mut r = recipe();
    r.backend.template = Some("python-fastapi".into());
    r.backend.database = Some(Database::Postgres);
    r.backend.database_location = Some(Location::External);
    let p = planner::build(r, &catalog, &temp.0).unwrap();
    assert_eq!(p.applications[0].language, "Python");
    assert!(
        p.applications[0]
            .planned_paths
            .iter()
            .any(|p| p.ends_with("app/adapters/postgres"))
    );
    assert!(
        p.applications[0]
            .planned_paths
            .iter()
            .all(|p| !p.ends_with("src/infrastructure/database"))
    );
}

#[test]
fn catalog_rejects_duplicate_ids_and_escaping_conditional_paths() {
    let source = include_str!("../catalog/express-ts.toml");
    assert!(Catalog::parse(&[source, source]).is_err());
    let bad = source.replace("src/infrastructure/database/", "../escape/");
    assert!(Catalog::parse(&[&bad]).is_err());
}

#[test]
fn doctor_requirements_are_selected_without_daemon_probes() {
    let temp = Temp::new();
    let c = Catalog::embedded().unwrap();
    let t = c.get("express-ts").unwrap();
    assert_eq!(doctor::requirements(t, None).len(), 2);
    let mut r = recipe();
    r.infrastructure.containers = Some(Containers::Compose);
    r.infrastructure.kubernetes = Some(Kubernetes::Manifests);
    let p = plan(r, &temp.0).unwrap();
    let req = doctor::requirements(t, Some(&p));
    assert!(
        req.iter()
            .any(|r| r.command == "docker" && r.args == ["compose", "version"])
    );
    assert!(
        req.iter()
            .any(|r| r.command == "kubectl" && r.args == ["version", "--client=true"])
    );
    assert!(!req.iter().any(|r| {
        r.args
            .iter()
            .any(|a| ["info", "up", "apply"].contains(&a.as_str()))
    }));
}

#[test]
fn cli_rejects_typos_and_accepts_new() {
    assert!(Cli::try_parse_from(["boilerplate", "new", "api"]).is_ok());
    assert!(Cli::try_parse_from(["boilerplate", "plan", "api", "--databse", "none"]).is_err());
    assert!(matches!(
        Cli::try_parse_from(["boilerplate", "plan", "--non-interactive"])
            .unwrap()
            .command,
        Command::Plan(_)
    ));
}

#[test]
fn cli_plan_is_machine_readable_and_independent_of_external_tools() {
    let temp = Temp::new();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_boilerplate"))
        .current_dir(&temp.0)
        .env("PATH", "")
        .args([
            "plan",
            "api",
            "--type",
            "backend",
            "--backend",
            "rust-axum",
            "--json",
            "--non-interactive",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let p: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(p["status"], "planning-only");
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
}

#[test]
fn doctor_missing_tools_and_invalid_input_have_distinct_exit_codes() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_boilerplate"))
        .env("PATH", "")
        .args(["doctor", "--template", "express-ts", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let checks: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(checks[0]["available"], false);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_boilerplate"))
        .args(["plan", "api"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}
