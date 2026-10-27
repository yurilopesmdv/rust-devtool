use crate::{
    catalog::{Requirement, Template},
    config::*,
    planner::Plan,
};
use serde::Serialize;
use std::process::Stdio;

#[derive(Debug, Serialize)]
pub struct Check {
    pub command: String,
    pub purpose: String,
    pub available: bool,
    pub compatible: Option<bool>,
    pub version: String,
}

fn tool(command: &str, args: &[&str], purpose: &str) -> Requirement {
    Requirement {
        command: command.into(),
        args: args.iter().map(|s| s.to_string()).collect(),
        purpose: purpose.into(),
    }
}

pub fn requirements(template: &Template, plan: Option<&Plan>) -> Vec<Requirement> {
    let mut requirements = template.requirements.clone();
    if let Some(plan) = plan {
        let infra = &plan.configuration.infrastructure;
        if template.role == ProjectType::Backend
            && plan.configuration.backend.migrations == Some(Migrations::Dbmate)
        {
            requirements.push(tool(
                "python3",
                &["--version"],
                "Parser de URI de migrations sem bibliotecas adicionais",
            ));
            requirements.push(tool(
                "psql",
                &["--version"],
                "Migrations explícitas: cliente PostgreSQL 17+",
            ));
            requirements.push(tool(
                "curl",
                &["--version"],
                "Instalação local explícita do dbmate fixado",
            ));
            requirements.push(tool(
                "sha256sum",
                &["--version"],
                "Verificar binário e integridade das migrations",
            ));
            requirements.push(tool(
                "setsid",
                &["--version"],
                "Encerrar o grupo de processos ao cancelar migrations",
            ));
        }
        if infra.containers == Some(Containers::Compose) {
            requirements.push(tool(
                "docker",
                &["--version"],
                "Executar containers posteriormente; não é necessário para planejar",
            ));
            requirements.push(tool(
                "docker",
                &["compose", "version"],
                "Executar Compose posteriormente; não consulta o daemon",
            ));
        }
        if infra.kubernetes == Some(Kubernetes::Manifests) {
            requirements.push(tool(
                "kubectl",
                &["version", "--client=true"],
                "Usar manifests posteriormente; não consulta o cluster",
            ));
        }
    }
    requirements
}

pub fn probe(requirements: &[Requirement], node_version: &str) -> Vec<Check> {
    let runtime = crate::generator::runtime::Runtime::find_node(node_version).unwrap_or_default();
    requirements
        .iter()
        .map(|requirement| {
            // Catálogo é embutido e versionado, não carregado da receita do usuário.
            let output = runtime
                .command(&requirement.command)
                .args(&requirement.args)
                .stdin(Stdio::null())
                .output();
            let (available, version) = match output {
                Ok(output) => {
                    let message = if output.stdout.is_empty() {
                        &output.stderr
                    } else {
                        &output.stdout
                    };
                    (
                        output.status.success(),
                        String::from_utf8_lossy(message).trim().to_string(),
                    )
                }
                Err(e) => (false, e.to_string()),
            };
            Check {
                command: std::iter::once(requirement.command.as_str())
                    .chain(requirement.args.iter().map(String::as_str))
                    .collect::<Vec<_>>()
                    .join(" "),
                purpose: requirement.purpose.clone(),
                compatible: (requirement.command == "node").then(|| {
                    available && crate::generator::runtime::matches_node(node_version, &version)
                }),
                available,
                version,
            }
        })
        .collect()
}

pub fn vcs_requirements(git: bool, github: bool) -> Vec<Requirement> {
    let mut requirements = Vec::new();
    if git || github {
        requirements.push(tool("git", &["--version"], "Inicializar Git local"));
    }
    if github {
        requirements.push(tool(
            "gh",
            &["--version"],
            "Criar repositório remoto quando autorizado",
        ));
    }
    requirements
}

pub fn auth_check() -> Check {
    let available = std::process::Command::new("gh")
        .args(["auth", "status", "--hostname", "github.com"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    Check {
        command: "gh auth status --hostname github.com".into(),
        purpose: "Verificar autenticação sem exibir tokens".into(),
        available,
        compatible: None,
        version: if available {
            "autenticado"
        } else {
            "indisponível; execute gh auth login"
        }
        .into(),
    }
}
