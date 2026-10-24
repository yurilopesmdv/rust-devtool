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
            r.project.web_dir =
                Some(line("Destino do frontend", Some(&format!("./{name}-web")))?.into());
        }
        if r.project.api_dir.is_none() {
            r.project.api_dir =
                Some(line("Destino do backend", Some(&format!("./{name}-api")))?.into());
        }
    } else if r.project.destination.is_none() {
        r.project.destination = Some(
            line(
                "Caminho de destino (diretório novo)",
                Some(&format!("./{name}")),
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
            if decision("Gerar Dockerfile, .dockerignore e Compose?", false)? {
                Containers::Compose
            } else {
                Containers::None
            },
        );
    }
    println!(
        "Banco, cache, Kubernetes, monitoria e Vault continuam apenas no planejador; new ainda não gera esses adapters."
    );
    Ok(r)
}
