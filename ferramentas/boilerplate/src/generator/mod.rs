pub mod architecture;
pub mod format;
pub mod integrations;
pub mod render;
pub mod runtime;
pub mod vcs;
pub mod wizard;
pub mod writer;

use crate::{
    Error,
    catalog::Catalog,
    cli::{Decision, NewArgs},
    config::{Layout, ProjectType},
    planner,
};

pub fn run(args: NewArgs, catalog: &Catalog) -> Result<(), Error> {
    let interactive = wizard::interactive(&args);
    if args.plan.json && !args.plan.non_interactive {
        return Err(Error::input("new --json exige --non-interactive"));
    }
    if !interactive && args.install.is_none() {
        return Err(Error::input(
            "Sem TTY, informe --install yes ou --install no; nenhum arquivo foi criado",
        ));
    }
    let recipe = wizard::complete(&args, catalog)?;
    let split = recipe.project.layout == Some(Layout::Split);
    vcs::validate(&args, split)?;
    let cwd = std::env::current_dir().map_err(|e| Error::operation(e.to_string()))?;
    let plan = planner::build(recipe, catalog, &cwd)?;
    let projects = render::render(&plan, catalog)?;
    if interactive {
        for project in &projects {
            println!(
                "{}: {} arquivos",
                project.destination.display(),
                project.files.len()
            );
        }
        println!("Git local: {:?}; nenhum serviço será iniciado.", args.git);
        if !wizard::decision("Criar estes projetos?", false)? {
            return Err(Error {
                message: "Cancelado; nenhum projeto criado".into(),
                code: 130,
            });
        }
    }
    if plan.configuration.infrastructure.containers == Some(crate::config::Containers::Compose)
        && !runtime::exists("docker")
    {
        eprintln!(
            "Docker não encontrado no PATH. Somente os arquivos serão gerados; instale/configure Docker no WSL antes de executar Compose."
        );
    }
    let roots = writer::publish(&projects)?;
    if args.git == Decision::Yes {
        vcs::init(&roots)?;
    }
    if !args.plan.json {
        for root in &roots {
            println!("Criado: {}", root.display());
        }
    }
    let install = match args.install {
        Some(d) => d == Decision::Yes,
        None if interactive => wizard::decision(
            "Instalar agora as dependências locais das aplicações?",
            true,
        )?,
        _ => false,
    };
    if install {
        let needs_node = plan.applications.iter().any(|app| {
            catalog
                .get(&app.template)
                .is_ok_and(|t| t.requirements.iter().any(|r| r.command == "node"))
        });
        let runtime = if needs_node {
            let version = &plan.configuration.toolchain.node_version;
            if let Some(runtime) = runtime::Runtime::find_node(version) {
                runtime
            } else {
                let setup = match args.setup_node {
                    Some(d) => d == Decision::Yes,
                    None if interactive => wizard::decision(
                        &format!(
                            "Node {version} não está disponível. Instalar via nvm (sem editar o perfil do shell)?"
                        ),
                        false,
                    )?,
                    _ => false,
                };
                if !setup {
                    return Err(Error::operation(format!(
                        "Node {version} ausente/incompatível. Projeto preservado. Instale/carregue essa versão via nvm e rode npm install na aplicação, ou execute novamente em um destino novo com --setup-node yes."
                    )));
                }
                runtime::Runtime::install_node(version)?
            }
        } else {
            runtime::Runtime::default()
        };
        for app in &plan.applications {
            runtime::install(app, &runtime, catalog.get(&app.template)?)?;
        }
    }
    let name = plan.configuration.project.name.as_deref().unwrap();
    let names: Vec<_> = roots
        .iter()
        .map(|root| {
            if split {
                let app = plan
                    .applications
                    .iter()
                    .find(|app| &app.destination == root)
                    .unwrap();
                if catalog.get(&app.template).unwrap().role == ProjectType::Frontend {
                    args.github_web
                        .clone()
                        .unwrap_or_else(|| format!("{name}-web"))
                } else {
                    args.github_api
                        .clone()
                        .unwrap_or_else(|| format!("{name}-api"))
                }
            } else {
                args.github_repo.clone().unwrap_or_else(|| name.to_string())
            }
        })
        .collect();
    vcs::github(&args, &roots, &names)?;
    if args.plan.json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"created": roots, "installed": install, "git": args.git == Decision::Yes})).map_err(|e| Error::operation(e.to_string()))?);
    } else {
        println!("Consulte README.md e AGENTS.md nas aplicações. Nenhum serviço foi iniciado.");
    }
    Ok(())
}
