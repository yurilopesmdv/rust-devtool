use rust_boilerplate::{catalog::Catalog, config::*, generator::render, planner};
use std::{collections::BTreeMap, path::PathBuf};
fn recipe(id: &str) -> Recipe {
    let mut r = Recipe::default();
    r.project.name = Some("integration-sample".into());
    r.project.kind = Some(ProjectType::Backend);
    r.backend.template = Some(id.into());
    r.infrastructure.containers = Some(Containers::Compose);
    r.infrastructure.kubernetes = Some(Kubernetes::Manifests);
    r
}
fn files(r: Recipe) -> BTreeMap<PathBuf, String> {
    let catalog = Catalog::embedded().unwrap();
    let cwd = std::env::temp_dir();
    let plan = planner::build(r, &catalog, &cwd).unwrap();
    render::render(&plan, &catalog).unwrap().remove(0).files
}
#[test]
fn backend_feature_matrix_is_composable_and_omits_unselected_drivers_and_resources() {
    for stack in ["express-ts", "nest-ts", "rust-axum"] {
        for database in [Database::None, Database::Postgres, Database::Mongodb] {
            for cache in [false, true] {
                for monitoring in [false, true] {
                    for vault in [false, true] {
                        let mut r = recipe(stack);
                        r.backend.database = Some(database);
                        if database != Database::None {
                            r.backend.database_location = Some(Location::Local);
                        }
                        r.backend.cache = Some(if cache { Cache::Redis } else { Cache::None });
                        if cache {
                            r.backend.cache_location = Some(Location::Local);
                        }
                        r.infrastructure.monitoring = Some(if monitoring {
                            Monitoring::PrometheusGrafana
                        } else {
                            Monitoring::None
                        });
                        r.infrastructure.secrets =
                            Some(if vault { Secrets::Vault } else { Secrets::Env });
                        if vault {
                            r.infrastructure.vault_location = Some(Location::Local);
                        }
                        let files = files(r);
                        let compose: serde_json::Value =
                            serde_json::from_str(&files[&PathBuf::from("compose.yaml")]).unwrap();
                        let services = compose["services"].as_object().unwrap();
                        assert_eq!(
                            services.contains_key("database"),
                            database != Database::None
                        );
                        assert_eq!(services.contains_key("cache"), cache);
                        assert_eq!(services.contains_key("prometheus"), monitoring);
                        assert_eq!(services.contains_key("grafana"), monitoring);
                        assert_eq!(services.contains_key("vault"), vault);
                        assert_eq!(services.contains_key("vault-agent"), vault);
                        assert_eq!(files.keys().any(|p| p.starts_with("infra/vault")), vault);
                        assert_eq!(
                            files.keys().any(|p| p.starts_with("infra/monitoring")),
                            monitoring
                        );
                        if stack != "rust-axum" {
                            let p: serde_json::Value =
                                serde_json::from_str(&files[&PathBuf::from("package.json")])
                                    .unwrap();
                            let lock: serde_json::Value =
                                serde_json::from_str(&files[&PathBuf::from("package-lock.json")])
                                    .unwrap();
                            for (dep, enabled) in [
                                ("pg", database == Database::Postgres),
                                ("mongodb", database == Database::Mongodb),
                                ("redis", cache),
                                ("prom-client", monitoring),
                            ] {
                                assert_eq!(
                                    p["dependencies"].get(dep).is_some(),
                                    enabled,
                                    "{stack} {dep}"
                                );
                                assert_eq!(
                                    lock["packages"]
                                        .get(format!("node_modules/{dep}"))
                                        .is_some(),
                                    enabled,
                                    "lock {stack} {dep}"
                                );
                            }
                        } else {
                            assert!(files.contains_key(&PathBuf::from("Cargo.lock")));
                            let cargo: toml::Table =
                                toml::from_str(&files[&PathBuf::from("Cargo.toml")]).unwrap();
                            for (dep, enabled) in [
                                ("sqlx", database == Database::Postgres),
                                ("mongodb", database == Database::Mongodb),
                                ("redis", cache),
                                ("prometheus", monitoring),
                            ] {
                                assert_eq!(
                                    cargo["dependencies"].get(dep).is_some(),
                                    enabled,
                                    "Rust {dep}"
                                );
                            }
                        }
                        for (path, source) in &files {
                            if path.starts_with("infra/kubernetes")
                                && path.extension().is_some_and(|e| e == "yaml")
                            {
                                let value: serde_json::Value =
                                    serde_json::from_str(source).unwrap();
                                assert_ne!(value["kind"], "Secret", "não versionar credenciais");
                                assert!(!source.contains("replace-me"));
                            }
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn external_services_do_not_generate_local_database_cache_or_vault_instances() {
    let mut r = recipe("express-ts");
    r.backend.database = Some(Database::Postgres);
    r.backend.database_location = Some(Location::External);
    r.backend.cache = Some(Cache::Redis);
    r.backend.cache_location = Some(Location::External);
    r.infrastructure.secrets = Some(Secrets::Vault);
    r.infrastructure.vault_location = Some(Location::External);
    let files = files(r);
    let compose: serde_json::Value =
        serde_json::from_str(&files[&PathBuf::from("compose.yaml")]).unwrap();
    for service in ["database", "cache", "vault"] {
        assert!(compose["services"].get(service).is_none());
    }
    assert!(!files.contains_key(&PathBuf::from("infra/vault/server.hcl")));
    assert!(!files.contains_key(&PathBuf::from("infra/kubernetes/base/vault-review.yaml")));
    assert!(files.contains_key(&PathBuf::from("infra/vault/agent.kubernetes.hcl")));
}
#[test]
fn vault_propagates_only_file_paths_to_the_application_and_protects_bootstrap_material() {
    let mut r = recipe("express-ts");
    r.backend.database = Some(Database::Postgres);
    r.backend.database_location = Some(Location::Local);
    r.infrastructure.secrets = Some(Secrets::Vault);
    r.infrastructure.vault_location = Some(Location::Local);
    let files = files(r);
    let compose: serde_json::Value =
        serde_json::from_str(&files[&PathBuf::from("compose.yaml")]).unwrap();
    let env = &compose["services"]["api"]["environment"];
    assert!(env.get("DATABASE_URL").is_none());
    assert_eq!(env["DATABASE_URL_FILE"], "/run/secrets/database-url");
    assert!(files[&PathBuf::from(".dockerignore")].contains("infra/vault/local/"));
    assert!(files[&PathBuf::from(".gitignore")].contains("infra/vault/local/*"));
    let deployment: serde_json::Value =
        serde_json::from_str(&files[&PathBuf::from("infra/kubernetes/base/api-deployment.yaml")])
            .unwrap();
    assert_eq!(
        deployment["spec"]["template"]["spec"]["initContainers"][0]["name"],
        "vault-agent"
    );
}
#[test]
fn no_database_does_not_create_repositories_or_preparation_jobs() {
    let mut r = recipe("rust-axum");
    r.backend.cache = Some(Cache::Redis);
    r.backend.cache_location = Some(Location::Local);
    let files = files(r);
    assert!(
        !files
            .keys()
            .any(|p| p.to_string_lossy().contains("repositories"))
    );
    assert!(!files.contains_key(&PathBuf::from("infra/kubernetes/jobs/prepare.yaml")));
    assert!(!files.contains_key(&PathBuf::from("examples/prepare.rs")));
}
#[test]
fn split_frontend_recipe_has_no_backend_services_and_keeps_urls_and_image_contract() {
    let mut r = recipe("express-ts");
    r.project.kind = Some(ProjectType::Fullstack);
    r.project.layout = Some(Layout::Split);
    r.project.web_dir = Some("integration-split-web".into());
    r.project.api_dir = Some("integration-split-api".into());
    r.frontend.template = Some("next-ts".into());
    r.infrastructure.image_web = Some("sample/web:1".into());
    r.infrastructure.image_api = Some("sample/api:1".into());
    r.infrastructure.frontend_origin = Some("https://web.example".into());
    r.infrastructure.api_url = Some("https://api.example".into());
    r.infrastructure.monitoring = Some(Monitoring::PrometheusGrafana);
    r.infrastructure.secrets = Some(Secrets::Vault);
    r.infrastructure.vault_location = Some(Location::Local);
    let c = Catalog::embedded().unwrap();
    let cwd = std::env::temp_dir();
    let p = planner::build(r, &c, &cwd).unwrap();
    let projects = render::render(&p, &c).unwrap();
    let frontend = projects
        .iter()
        .find(|p| p.destination.ends_with("integration-split-web"))
        .unwrap();
    let config = Recipe::parse(&frontend.files[&PathBuf::from("boilerplate.toml")]).unwrap();
    assert_eq!(config.infrastructure.secrets, Some(Secrets::Env));
    assert_eq!(config.infrastructure.monitoring, Some(Monitoring::None));
    assert_eq!(config.infrastructure.image.as_deref(), Some("sample/web:1"));
    assert!(config.infrastructure.image_api.is_none());
    assert!(planner::build(config, &c, &cwd).is_ok());
    assert!(!frontend.files.keys().any(|p| p.starts_with("infra/vault")));
}

#[test]
fn rust_custom_origin_and_public_frontend_urls_survive_generation() {
    let mut r = recipe("rust-axum");
    r.infrastructure.frontend_origin = Some("https://web.example".into());
    r.infrastructure.monitoring = Some(Monitoring::PrometheusGrafana);
    let output = files(r);
    assert!(output[&PathBuf::from("src/lib.rs")].contains("https://web.example"));
    let deployment: serde_json::Value =
        serde_json::from_str(&output[&PathBuf::from("infra/kubernetes/base/api-deployment.yaml")])
            .unwrap();
    assert!(
        deployment["spec"]["template"]["spec"]["containers"][0]["env"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["name"] == "FRONTEND_ORIGIN" && v["value"] == "https://web.example")
    );
}
#[test]
fn immutable_image_digests_and_static_metrics_without_compose_are_supported() {
    use rust_boilerplate::planner::valid_image;
    let digest = format!("example/web@sha256:{}", "a".repeat(64));
    assert!(valid_image(&digest));
    for image in [
        "example/api:latest",
        "https://registry/repo:1",
        "user@registry:tag",
        "repo:bad\"tag",
        "repo@sha256:short",
    ] {
        assert!(!valid_image(image));
    }
    let mut r = Recipe::default();
    r.project.name = Some("static-metrics-sample".into());
    r.project.kind = Some(ProjectType::Frontend);
    r.frontend.template = Some("vanilla-js".into());
    r.infrastructure.kubernetes = Some(Kubernetes::Manifests);
    r.infrastructure.image = Some(digest);
    r.infrastructure.monitoring = Some(Monitoring::PrometheusGrafana);
    let output = files(r);
    assert!(!output.contains_key(&PathBuf::from("compose.yaml")));
    assert!(output.contains_key(&PathBuf::from("infra/kubernetes/base/web-metrics.yaml")));
}

#[test]
fn external_vault_address_is_shared_by_agent_modes_and_disabled_options_reject_it() {
    let mut r = recipe("express-ts");
    r.infrastructure.secrets = Some(Secrets::Vault);
    r.infrastructure.vault_location = Some(Location::External);
    r.infrastructure.vault_address = Some("https://vault.example:8200".into());
    let output = files(r.clone());
    assert!(
        output[&PathBuf::from("infra/.env.example")]
            .contains("VAULT_ADDR=https://vault.example:8200")
    );
    assert!(
        output[&PathBuf::from("infra/vault/agent.kubernetes.hcl")]
            .contains("https://vault.example:8200")
    );
    r.infrastructure.secrets = Some(Secrets::Env);
    r.infrastructure.vault_location = None;
    let catalog = Catalog::embedded().unwrap();
    assert!(planner::build(r, &catalog, &std::env::temp_dir()).is_err());
}
#[test]
fn vault_container_contract_uses_owned_storage_and_bypasses_entrypoint_for_approle_files() {
    let mut r = recipe("express-ts");
    r.infrastructure.secrets = Some(Secrets::Vault);
    r.infrastructure.vault_location = Some(Location::Local);
    let output = files(r);
    let compose: serde_json::Value =
        serde_json::from_str(&output[&PathBuf::from("compose.yaml")]).unwrap();
    assert_eq!(
        compose["services"]["vault-agent"]["entrypoint"],
        serde_json::json!(["vault"])
    );
    assert_eq!(
        compose["services"]["vault"]["command"],
        serde_json::json!(["server"])
    );
    assert!(
        compose["services"]["vault"]["volumes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "vault-data:/vault/file")
    );
    assert!(output[&PathBuf::from("infra/vault/server.hcl")].contains("/vault/file"));
}
