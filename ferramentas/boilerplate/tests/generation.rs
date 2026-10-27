use rust_boilerplate::{
    catalog::Catalog,
    config::*,
    generator::{render, runtime, vcs, writer},
    planner,
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static COUNT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "boilerplate-generation-{}-{}-{}",
            std::process::id(),
            COUNT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn recipe(id: &str, role: ProjectType) -> Recipe {
    let mut r = Recipe::default();
    r.project.name = Some("meu-projeto".into());
    r.project.destination = Some("meu-projeto".into());
    r.project.kind = Some(role);
    if role == ProjectType::Frontend {
        r.frontend.template = Some(id.into());
    } else {
        r.backend.template = Some(id.into());
    }
    r
}
fn create(id: &str, role: ProjectType, runtime_compose: bool) -> (Temp, Vec<writer::ProjectFiles>) {
    let temp = Temp::new();
    let c = Catalog::embedded().unwrap();
    let mut r = recipe(id, role);
    if runtime_compose {
        r.infrastructure.containers = Some(Containers::Compose);
    }
    let p = planner::build(r, &c, &temp.0).unwrap();
    let projects = render::render(&p, &c).unwrap();
    (temp, projects)
}
#[test]
fn every_base_generates_sources_ignore_rules_agent_instructions_and_recipe() {
    for (id, role) in [
        ("go-http", ProjectType::Backend),
        ("express-ts", ProjectType::Backend),
        ("nest-ts", ProjectType::Backend),
        ("rust-axum", ProjectType::Backend),
        ("vanilla-js", ProjectType::Frontend),
        ("react-vite-ts", ProjectType::Frontend),
        ("next-ts", ProjectType::Frontend),
    ] {
        let (_temp, projects) = create(id, role, false);
        writer::publish(&projects).unwrap();
        let root = &projects[0].destination;
        for file in ["README.md", "AGENTS.md", ".gitignore", "boilerplate.toml"] {
            assert!(root.join(file).is_file(), "{id}: {file}");
        }
        assert!(!root.join(".dockerignore").exists());
        assert!(!root.join("Dockerfile").exists());
        assert!(!root.join("compose.yaml").exists());
        let recipe = Recipe::load(&root.join("boilerplate.toml")).unwrap();
        assert_eq!(recipe.project.name.as_deref(), Some("meu-projeto"));
        let ignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(ignore.contains(".env\n"));
        assert!(ignore.contains("!.env.example"));
    }
}
#[test]
fn esm_is_default_and_commonjs_is_explicitly_supported_by_express() {
    let temp = Temp::new();
    let c = Catalog::embedded().unwrap();
    for (system, expected) in [
        (ModuleSystem::Esm, "module"),
        (ModuleSystem::Commonjs, "commonjs"),
    ] {
        let mut r = recipe("express-ts", ProjectType::Backend);
        r.toolchain.module_system = system;
        let p = planner::build(r, &c, &temp.0).unwrap();
        let projects = render::render(&p, &c).unwrap();
        let package: serde_json::Value =
            serde_json::from_str(&projects[0].files[&PathBuf::from("package.json")]).unwrap();
        assert_eq!(package["type"], expected);
        assert_eq!(projects[0].files[&PathBuf::from(".nvmrc")], "24\n");
        assert!(projects[0].files[&PathBuf::from("src/app.ts")].contains("import express"));
    }
    let mut r = recipe("next-ts", ProjectType::Frontend);
    r.toolchain.module_system = ModuleSystem::Commonjs;
    assert!(planner::build(r, &c, &temp.0).is_err());
}
#[test]
fn compose_selection_controls_dockerignore_and_dockerfile_for_every_base() {
    for (id, role) in [
        ("express-ts", ProjectType::Backend),
        ("nest-ts", ProjectType::Backend),
        ("rust-axum", ProjectType::Backend),
        ("react-vite-ts", ProjectType::Frontend),
        ("next-ts", ProjectType::Frontend),
        ("vanilla-js", ProjectType::Frontend),
    ] {
        let (_temp, projects) = create(id, role, true);
        let f = &projects[0].files;
        assert!(f.contains_key(&PathBuf::from("Dockerfile")), "{id}");
        assert!(f[&PathBuf::from(".dockerignore")].contains(".env"));
        assert!(f[&PathBuf::from("compose.yaml")].contains("127.0.0.1:"));
    }
}
#[test]
fn all_pairs_generate_in_monorepo_and_split_without_absolute_recipe_paths() {
    let temp = Temp::new();
    let c = Catalog::embedded().unwrap();
    for front in ["vanilla-js", "react-vite-ts", "next-ts"] {
        for back in ["express-ts", "nest-ts", "rust-axum", "python-fastapi"] {
            for layout in [Layout::Monorepo, Layout::Split] {
                let mut r = recipe(back, ProjectType::Fullstack);
                r.frontend.template = Some(front.into());
                r.project.layout = Some(layout);
                if layout == Layout::Split {
                    r.project.destination = None;
                    r.project.web_dir = Some("web".into());
                    r.project.api_dir = Some("api".into());
                }
                let p = planner::build(r, &c, &temp.0).unwrap();
                let projects = render::render(&p, &c).unwrap();
                assert_eq!(projects.len(), if layout == Layout::Split { 2 } else { 1 });
                for project in projects {
                    assert!(
                        !project.files[&PathBuf::from("boilerplate.toml")]
                            .contains(temp.0.to_str().unwrap())
                    );
                }
            }
        }
    }
}
#[test]
fn integrations_are_rendered_before_writing() {
    let temp = Temp::new();
    let c = Catalog::embedded().unwrap();
    let mut r = recipe("express-ts", ProjectType::Backend);
    r.backend.database = Some(Database::Postgres);
    r.backend.database_location = Some(Location::External);
    let p = planner::build(r, &c, &temp.0).unwrap();
    let projects = render::render(&p, &c).unwrap();
    assert!(
        projects[0]
            .files
            .contains_key(&PathBuf::from("src/infrastructure/database/client.ts"))
    );
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
}
#[test]
fn writer_preserves_existing_destinations_and_does_not_publish_first_of_invalid_pair() {
    let temp = Temp::new();
    let first = temp.0.join("one");
    let second = temp.0.join("two");
    std::fs::create_dir(&second).unwrap();
    std::fs::write(second.join("keep"), "original").unwrap();
    let f = BTreeMap::from([(PathBuf::from("source"), String::from("new"))]);
    let projects = vec![
        writer::ProjectFiles {
            destination: first.clone(),
            files: f.clone(),
        },
        writer::ProjectFiles {
            destination: second.clone(),
            files: f,
        },
    ];
    assert!(writer::publish(&projects).is_err());
    assert!(!first.exists());
    assert_eq!(
        std::fs::read_to_string(second.join("keep")).unwrap(),
        "original"
    );
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1);
}
#[test]
fn writer_rejects_parent_paths_without_escaping_staging() {
    let temp = Temp::new();
    let project = writer::ProjectFiles {
        destination: temp.0.join("app"),
        files: BTreeMap::from([(PathBuf::from("../escape"), "bad".into())]),
    };
    assert!(writer::publish(&[project]).is_err());
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
}
#[test]
fn node_versions_and_github_names_reject_shell_injection_and_incompatible_versions() {
    for v in ["24", "24.15.0", "22.12.0"] {
        assert!(runtime::valid_node_version(v));
    }
    for v in [
        "20",
        "22.0.0",
        "lts/*",
        "24; touch hacked",
        "../node",
        "24.1.0.2",
    ] {
        assert!(!runtime::valid_node_version(v));
    }
    assert!(runtime::matches_node("24", "v24.15.0"));
    assert!(!runtime::matches_node("24.15.0", "v24.16.0"));
    for v in ["owner/repo", "repo-name"] {
        assert!(vcs::valid_remote(v));
    }
    for v in [
        "--public",
        "owner/../repo",
        "a/b/c",
        "$(touch hacked)",
        "https://github.com/x/y",
    ] {
        assert!(!vcs::valid_remote(v));
    }
}
fn cli(temp: &Temp) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_boilerplate"));
    c.current_dir(&temp.0);
    c
}
#[test]
fn unattended_generation_requires_install_decision_and_never_prompts() {
    let temp = Temp::new();
    let output = cli(&temp)
        .args([
            "new",
            "api",
            "--destination",
            "api",
            "--type",
            "backend",
            "--backend",
            "express-ts",
            "--non-interactive",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!temp.0.join("api").exists());
}
#[test]
fn json_generation_is_usable_without_any_global_tools_if_install_and_git_are_disabled() {
    let temp = Temp::new();
    let output = cli(&temp)
        .env("PATH", "")
        .args([
            "new",
            "api",
            "--destination",
            "api",
            "--type",
            "backend",
            "--backend",
            "express-ts",
            "--install",
            "no",
            "--git",
            "no",
            "--github",
            "no",
            "--non-interactive",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["installed"], false);
    assert!(temp.0.join("api/src/server.ts").is_file());
    assert!(!temp.0.join("api/.git").exists());
}
#[test]
fn missing_node_preserves_generated_files_and_does_not_install_without_authorization() {
    let temp = Temp::new();
    let home = temp.0.join("isolated-home");
    std::fs::create_dir(&home).unwrap();
    let output = cli(&temp)
        .env("PATH", "")
        .env("NVM_DIR", home.join("nvm"))
        .args([
            "new",
            "api",
            "--destination",
            "api",
            "--type",
            "backend",
            "--backend",
            "express-ts",
            "--install",
            "yes",
            "--setup-node",
            "no",
            "--git",
            "no",
            "--github",
            "no",
            "--non-interactive",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(temp.0.join("api/package.json").is_file());
    assert!(!home.join("nvm").exists());
}
#[cfg(unix)]
#[test]
fn github_is_opt_in_checks_auth_and_keeps_push_separate_using_fake_tools() {
    use std::os::unix::fs::PermissionsExt;
    let temp = Temp::new();
    let bin = temp.0.join("bin");
    std::fs::create_dir(&bin).unwrap();
    let log = temp.0.join("commands");
    for tool in ["git", "gh"] {
        let file = bin.join(tool);
        std::fs::write(
            &file,
            format!("#!/bin/sh\nprintf '%s\\n' '{tool}' \"$*\" >> \"$COMMAND_LOG\"\nexit 0\n"),
        )
        .unwrap();
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = cli(&temp)
        .env("PATH", &bin)
        .env("COMMAND_LOG", &log)
        .args([
            "new",
            "api",
            "--destination",
            "api",
            "--type",
            "backend",
            "--backend",
            "express-ts",
            "--install",
            "no",
            "--github",
            "yes",
            "--github-repo",
            "org/api",
            "--push",
            "no",
            "--non-interactive",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["git"], true);
    let calls = std::fs::read_to_string(log).unwrap();
    assert!(calls.contains("auth status --hostname github.com"));
    assert!(calls.contains("repo create org/api --private --source . --remote origin"));
    assert!(!calls.contains("--push"));
    assert!(!calls.contains("commit"));
}
#[test]
fn push_without_github_is_rejected_before_publication() {
    let temp = Temp::new();
    let output = cli(&temp)
        .args([
            "new",
            "api",
            "--destination",
            "api",
            "--type",
            "backend",
            "--backend",
            "express-ts",
            "--install",
            "no",
            "--github",
            "no",
            "--push",
            "yes",
            "--non-interactive",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!temp.0.join("api").exists());
}

#[test]
fn git_is_initialized_on_main_without_a_commit_or_remote_by_default() {
    if !runtime::exists("git") {
        return;
    }
    let temp = Temp::new();
    let output = cli(&temp)
        .args([
            "new",
            "api",
            "--destination",
            "api",
            "--type",
            "backend",
            "--backend",
            "express-ts",
            "--install",
            "no",
            "--github",
            "no",
            "--non-interactive",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let root = temp.0.join("api");
    let branch = Command::new("git")
        .args(["symbolic-ref", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&branch.stdout).trim(),
        "refs/heads/main"
    );
    let remote = Command::new("git")
        .arg("remote")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(remote.stdout.is_empty());
}

#[cfg(unix)]
#[test]
fn github_auth_failure_preserves_sources_and_never_creates_a_remote() {
    use std::os::unix::fs::PermissionsExt;
    let temp = Temp::new();
    let bin = temp.0.join("bin");
    std::fs::create_dir(&bin).unwrap();
    let log = temp.0.join("commands");
    for (name, code) in [
        ("git", "exit 0"),
        ("gh", "if [ \"$1\" = auth ]; then exit 1; fi\nexit 0"),
    ] {
        let file = bin.join(name);
        std::fs::write(
            &file,
            format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$COMMAND_LOG\"\n{code}\n"),
        )
        .unwrap();
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = cli(&temp)
        .env("PATH", &bin)
        .env("COMMAND_LOG", &log)
        .args([
            "new",
            "api",
            "--destination",
            "api",
            "--type",
            "backend",
            "--backend",
            "express-ts",
            "--install",
            "no",
            "--github",
            "yes",
            "--push",
            "no",
            "--non-interactive",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(temp.0.join("api/package.json").exists());
    assert!(
        !std::fs::read_to_string(log)
            .unwrap()
            .contains("repo create")
    );
}

#[test]
fn declared_empty_architecture_folders_are_preserved_for_future_features() {
    let (_temp, projects) = create("react-vite-ts", ProjectType::Frontend, false);
    for folder in [
        "src/components/.gitkeep",
        "src/features/.gitkeep",
        "docs/.gitkeep",
    ] {
        assert!(
            projects[0].files.contains_key(&PathBuf::from(folder)),
            "{folder}"
        );
    }
}

#[test]
fn default_generation_is_outside_working_directory_and_split_uses_same_base() {
    let temp = Temp::new();
    let home = temp.0.join("home");
    let cwd = temp.0.join("rust-devtool");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir(&cwd).unwrap();
    let output = cli(&temp)
        .current_dir(&cwd)
        .env("HOME", &home)
        .args([
            "new",
            "teste",
            "--type",
            "frontend",
            "--frontend",
            "vanilla-js",
            "--install",
            "no",
            "--git",
            "no",
            "--github",
            "no",
            "--non-interactive",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(home.join("projects/teste/index.html").is_file());
    assert_eq!(std::fs::read_dir(&cwd).unwrap().count(), 0);
    let output = cli(&temp)
        .current_dir(&cwd)
        .env("HOME", &home)
        .args([
            "plan",
            "plataforma",
            "--type",
            "fullstack",
            "--layout",
            "split",
            "--frontend",
            "vanilla-js",
            "--backend",
            "express-ts",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        plan["applications"][0]["destination"],
        home.join("projects/plataforma-web").to_str().unwrap()
    );
    assert_eq!(
        plan["applications"][1]["destination"],
        home.join("projects/plataforma-api").to_str().unwrap()
    );
}

#[test]
fn go_generates_with_every_frontend_in_both_layouts() {
    let temp = Temp::new();
    let catalog = Catalog::embedded().unwrap();
    for frontend in ["vanilla-js", "react-vite-ts", "next-ts"] {
        for layout in [Layout::Monorepo, Layout::Split] {
            let mut r = recipe("go-http", ProjectType::Fullstack);
            r.backend.template = Some("go-http".into());
            r.frontend.template = Some(frontend.into());
            r.project.layout = Some(layout);
            if layout == Layout::Split {
                r.project.destination = None;
                r.project.web_dir = Some(temp.0.join("web"));
                r.project.api_dir = Some(temp.0.join("api"));
            }
            let plan = planner::build(r, &catalog, &temp.0).unwrap();
            let projects = render::render(&plan, &catalog).unwrap();
            assert_eq!(projects.len(), if layout == Layout::Split { 2 } else { 1 });
            assert!(projects.iter().any(|project| {
                project
                    .files
                    .keys()
                    .any(|path| path.ends_with("cmd/api/main.go"))
            }));
            assert!(
                projects
                    .iter()
                    .all(|project| !project.files.keys().any(|path| path.ends_with("go.sum")))
            );
        }
    }
}

#[test]
fn go_postgres_is_optional_and_uses_repository_pool_and_preparation() {
    let (_temp, minimal) = create("go-http", ProjectType::Backend, false);
    assert!(!minimal[0].files.contains_key(&PathBuf::from("go.sum")));
    assert!(
        !minimal[0]
            .files
            .keys()
            .any(|path| path.to_string_lossy().contains("repositories"))
    );
    let temp = Temp::new();
    let catalog = Catalog::embedded().unwrap();
    let mut r = recipe("go-http", ProjectType::Backend);
    r.backend.database = Some(Database::Postgres);
    r.backend.database_location = Some(Location::Local);
    r.infrastructure.containers = Some(Containers::Compose);
    r.infrastructure.kubernetes = Some(Kubernetes::Manifests);
    let plan = planner::build(r, &catalog, &temp.0).unwrap();
    let projects = render::render(&plan, &catalog).unwrap();
    let files = &projects[0].files;
    assert!(files[&PathBuf::from("go.mod")].contains("pgx/v5 v5.11.0"));
    assert!(files.contains_key(&PathBuf::from("go.sum")));
    assert!(
        files[&PathBuf::from("internal/modules/example/repositories/repository.go")]
            .contains("SELECT message")
    );
    assert!(!files[&PathBuf::from("internal/config/config.go")].contains("SELECT"));
    assert!(files[&PathBuf::from("cmd/api/main.go")].contains("defer pool.Close()"));
    assert!(files[&PathBuf::from("Dockerfile")].contains("/app/prepare"));
    assert!(files[&PathBuf::from("infra/kubernetes/jobs/prepare.yaml")].contains("/app/prepare"));
}

#[test]
fn generated_ci_is_opt_in_and_belongs_to_the_api_repository() {
    let temp = Temp::new();
    let catalog = Catalog::embedded().unwrap();
    for enabled in [false, true] {
        for layout in [None, Some(Layout::Monorepo), Some(Layout::Split)] {
            let mut r = recipe("express-ts", ProjectType::Backend);
            r.infrastructure.ci = Some(if enabled { Ci::GithubActions } else { Ci::None });
            if let Some(layout) = layout {
                r.project.kind = Some(ProjectType::Fullstack);
                r.project.layout = Some(layout);
                r.frontend.template = Some("react-vite-ts".into());
                if layout == Layout::Split {
                    r.project.destination = None;
                    r.project.web_dir = Some(temp.0.join("web"));
                    r.project.api_dir = Some(temp.0.join("api"));
                }
            }
            let plan = planner::build(r, &catalog, &temp.0).unwrap();
            let projects = render::render(&plan, &catalog).unwrap();
            let workflows: Vec<_> = projects
                .iter()
                .filter_map(|project| {
                    project
                        .files
                        .get(&PathBuf::from(".github/workflows/ci.yml"))
                })
                .collect();
            assert_eq!(workflows.len(), usize::from(enabled));
            if enabled {
                let workflow = workflows[0];
                assert!(workflow.contains("npm run check"));
                assert!(workflow.contains("contents: read"));
                assert!(workflow.contains(if layout == Some(Layout::Monorepo) {
                    "working-directory: 'apps/api'"
                } else {
                    "working-directory: '.'"
                }));
            }
            for project in projects {
                let recipe =
                    Recipe::parse(&project.files[&PathBuf::from("boilerplate.toml")]).unwrap();
                if recipe.project.kind == Some(ProjectType::Frontend) {
                    assert_eq!(recipe.infrastructure.ci, Some(Ci::None));
                }
            }
        }
    }
    let mut unsupported = recipe("nest-ts", ProjectType::Backend);
    unsupported.infrastructure.ci = Some(Ci::GithubActions);
    assert!(planner::build(unsupported, &catalog, &temp.0).is_err());
}

#[test]
fn versioned_postgres_migrations_are_explicit_and_omitted_when_disabled() {
    let temp = Temp::new();
    let catalog = Catalog::embedded().unwrap();
    for stack in [
        "express-ts",
        "nest-ts",
        "rust-axum",
        "python-fastapi",
        "go-http",
    ] {
        for enabled in [false, true] {
            let mut r = recipe(stack, ProjectType::Backend);
            r.backend.database = Some(Database::Postgres);
            r.backend.database_location = Some(Location::External);
            r.backend.migrations = Some(if enabled {
                Migrations::Dbmate
            } else {
                Migrations::None
            });
            let plan = planner::build(r, &catalog, &temp.0).unwrap();
            let generated = render::render(&plan, &catalog).unwrap();
            let files = &generated[0].files;
            assert_eq!(
                files.contains_key(&PathBuf::from("scripts/migrate.sh")),
                enabled
            );
            assert_eq!(
                files.contains_key(&PathBuf::from("scripts/_pg_client.py")),
                enabled
            );
            assert_eq!(
                files.contains_key(&PathBuf::from("docs/MIGRATIONS.md")),
                enabled
            );
            assert_eq!(
                files.keys().any(|path| path.starts_with("migrations/sql")),
                enabled
            );
            if enabled {
                assert!(files[&PathBuf::from("scripts/migrate.sh")].contains("--migration-lock"));
                assert!(files[&PathBuf::from("scripts/migrate.sh")].contains("f855e99a0f0a8132"));
            }
        }
    }
    let mut invalid = recipe("express-ts", ProjectType::Backend);
    invalid.backend.migrations = Some(Migrations::Dbmate);
    assert!(planner::build(invalid, &catalog, &temp.0).is_err());
}
