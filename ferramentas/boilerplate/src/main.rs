use clap::Parser;
use rust_boilerplate::{
    Error,
    catalog::Catalog,
    cli::{Cli, Command, Templates},
    config::Recipe,
    doctor, generator, planner,
};
use std::process::ExitCode;

fn json(value: &impl serde::Serialize) -> Result<(), Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| Error::operation(e.to_string()))?
    );
    Ok(())
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let catalog = Catalog::embedded()?;
    match cli.command {
        Command::Templates {
            command: Templates::List { json: as_json },
        } => {
            if as_json {
                json(&catalog.templates)?;
            } else {
                println!("Catálogo 1 — bases web e integrações opcionais disponíveis.");
                for t in &catalog.templates {
                    println!(
                        "{} | {:?} | {} | {} | {:?}",
                        t.id, t.role, t.language, t.framework, t.status
                    );
                }
            }
        }
        Command::Templates {
            command: Templates::Show { id, json: as_json },
        } => {
            let t = catalog.get(&id)?;
            if as_json {
                json(t)?;
            } else {
                println!(
                    "{} — {} ({})\nPapel: {:?}\nStatus: {:?}\nArquitetura: {}\nCapacidades: {}\nGerenciador: {}",
                    t.id,
                    t.framework,
                    t.language,
                    t.role,
                    t.status,
                    t.architecture,
                    t.capabilities.join(", "),
                    t.package_manager
                );
                println!("Consulte new --help; combinações são validadas antes de criar arquivos.");
                for requirement in &t.requirements {
                    println!(
                        "Requisito: {} — {}",
                        requirement.command, requirement.purpose
                    );
                }
            }
        }
        Command::New(args) => generator::run(*args, &catalog)?,
        Command::Plan(args) => {
            let cwd = std::env::current_dir().map_err(|e| Error::operation(e.to_string()))?;
            let plan = planner::build(args.recipe()?, &catalog, &cwd)?;
            if args.json {
                json(&plan)?;
            } else {
                println!("Plano de projeto — não cria arquivos nem executa ferramentas");
                for app in &plan.applications {
                    println!(
                        "\n{} ({}) → {}",
                        app.template,
                        app.language,
                        app.destination.display()
                    );
                    for path in &app.planned_paths {
                        println!("  {}", path.display());
                    }
                    if !app.install_command.is_empty() {
                        println!("  Instalação futura: {}", app.install_command.join(" "));
                    }
                }
                println!(
                    "\nIntegrações: {}\nOmitidas: {}",
                    plan.integrations.join(", "),
                    plan.omitted.join(", ")
                );
                for path in &plan.infrastructure_paths {
                    println!("Infra: {}", path.display());
                }
                for note in &plan.notes {
                    println!("• {note}");
                }
            }
        }
        Command::Doctor {
            template,
            config,
            node_version,
            git,
            github,
            json: as_json,
        } => {
            let t = catalog.get(&template)?;
            let plan = if let Some(path) = config {
                let r = Recipe::load(&path)?;
                let cwd = std::env::current_dir().map_err(|e| Error::operation(e.to_string()))?;
                let p = planner::build(r, &catalog, &cwd)?;
                if !p.applications.iter().any(|app| app.template == template) {
                    return Err(Error::input("--template precisa estar presente na receita"));
                }
                Some(p)
            } else {
                None
            };
            let version = node_version
                .as_deref()
                .or_else(|| {
                    plan.as_ref()
                        .map(|p| p.configuration.toolchain.node_version.as_str())
                })
                .unwrap_or("24");
            if !generator::runtime::valid_node_version(version) {
                return Err(Error::input("Versão Node inválida"));
            }
            let mut requirements = doctor::requirements(t, plan.as_ref());
            requirements.extend(doctor::vcs_requirements(git, github));
            let mut checks = doctor::probe(&requirements, version);
            if github {
                checks.push(doctor::auth_check());
            }
            if as_json {
                json(&checks)?;
            } else {
                println!(
                    "Requisitos para instalação/execução em WSL/Linux; planejar não exige essas ferramentas."
                );
                println!(
                    "Node é verificado contra a versão solicitada, usando nvm existente quando disponível. Outros requisitos verificam presença/versão."
                );
                for check in &checks {
                    println!(
                        "{} {}: {}",
                        if check.available && check.compatible != Some(false) {
                            "OK"
                        } else {
                            "AUSENTE/FALHOU"
                        },
                        check.command,
                        check.version
                    );
                }
                if checks.is_empty() {
                    println!(
                        "Nenhum runtime de pacotes é obrigatório para esta base. Use um servidor HTTP local para executar."
                    );
                }
            }
            if checks
                .iter()
                .any(|check| !check.available || check.compatible == Some(false))
            {
                return Err(Error::operation(
                    "Há ferramentas ausentes ou que não responderam ao comando de versão",
                ));
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Erro: {error}");
            ExitCode::from(error.code)
        }
    }
}
