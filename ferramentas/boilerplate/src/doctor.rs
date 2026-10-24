use crate::{
    catalog::{Requirement, Template},
    config::*,
    planner::Plan,
};
use serde::Serialize;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize)]
pub struct Check {
    pub command: String,
    pub purpose: String,
    pub available: bool,
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

pub fn probe(requirements: &[Requirement]) -> Vec<Check> {
    requirements
        .iter()
        .map(|requirement| {
            // Catálogo é embutido e versionado, não carregado da receita do usuário.
            let output = Command::new(&requirement.command)
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
                available,
                version,
            }
        })
        .collect()
}
