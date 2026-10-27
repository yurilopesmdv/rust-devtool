use super::integrations::Files;
use crate::{
    Error,
    catalog::Catalog,
    config::{Ci, ProjectType},
    planner::Plan,
};
use std::path::{Path, PathBuf};

fn runtime(manager: &str) -> Result<(&'static str, &'static str), Error> {
    match manager {
        "npm" => Ok((
            include_str!("../../integrations/ci/npm-job.yml"),
            include_str!("../../integrations/ci/npm-check.sh"),
        )),
        "uv" => Ok((
            include_str!("../../integrations/ci/uv-job.yml"),
            include_str!("../../integrations/ci/uv-check.sh"),
        )),
        "cargo" => Ok((
            include_str!("../../integrations/ci/cargo-job.yml"),
            include_str!("../../integrations/ci/cargo-check.sh"),
        )),
        "go" => Ok((
            include_str!("../../integrations/ci/go-job.yml"),
            include_str!("../../integrations/ci/go-check.sh"),
        )),
        _ => Err(Error::operation(format!(
            "CI não implementada para gerenciador {manager}"
        ))),
    }
}

pub fn apply(plan: &Plan, catalog: &Catalog, root: &Path, files: &mut Files) -> Result<(), Error> {
    if plan.configuration.infrastructure.ci != Some(Ci::GithubActions) {
        return Ok(());
    }
    let mut workflow = include_str!("../../integrations/ci/header.yml").to_string();
    let mut guide = String::from("# CI dos projetos\n\n");
    let mut count = 0;
    for app in plan
        .applications
        .iter()
        .filter(|app| app.destination.starts_with(root))
    {
        let template = catalog.get(&app.template)?;
        let prefix = app
            .destination
            .strip_prefix(root)
            .map_err(|e| Error::operation(e.to_string()))?;
        let directory = if prefix.as_os_str().is_empty() {
            ".".to_string()
        } else {
            prefix.to_string_lossy().to_string()
        };
        let role = if template.role == ProjectType::Frontend {
            "web"
        } else {
            "api"
        };
        let (job, script) = runtime(&template.package_manager)?;
        let job = job.replace("@@WORKING_DIRECTORY@@", &directory).replace(
            "@@NODE_VERSION@@",
            &plan.configuration.toolchain.node_version,
        );
        if job.contains("@@") {
            return Err(Error::operation("CI com slot não resolvido"));
        }
        workflow.push_str(&format!(
            "  check-{role}:\n    name: Quality ({})\n",
            template.id
        ));
        for line in job.lines() {
            if !line.is_empty() {
                workflow.push_str("    ");
                workflow.push_str(line);
            }
            workflow.push('\n');
        }
        if files
            .insert(prefix.join(".ci/check.sh"), script.to_string())
            .is_some()
        {
            return Err(Error::operation("Script CI duplicado"));
        }
        guide.push_str(&format!(
            "- {} em `{directory}`: execute `sh .ci/check.sh` nesta pasta.\n",
            template.id
        ));
        let link = if prefix.as_os_str().is_empty() {
            "docs/CI.md"
        } else {
            "../../docs/CI.md"
        };
        if let Some(readme) = files.get_mut(&prefix.join("README.md")) {
            readme.push_str(&format!("\n## CI opcional\n\nO workflow chama o mesmo comando local: `sh .ci/check.sh`. Veja [CI]({link}) na raiz do repositório.\n"));
        }
        count += 1;
    }
    if count == 0 {
        return Err(Error::operation("CI selecionada sem aplicação nesta raiz"));
    }
    guide.push_str("\nActions fixadas por SHA; contents:read, checkout sem credenciais persistentes e timeout por job. A instalação é local ao ambiente e usa os locks; não há deploy, publicação, registry login, push, criação de clusters ou uso de segredos.\n\nNo monorepo, cada aplicação tem seu job; no split, cada repositório tem seu próprio workflow. Os jobs executam somente checks da aplicação. Banco/cache/Vault/monitoria não são provisionados: testes condicionados a URLs podem ser pulados. Acrescente serviços descartáveis e preparação/migrations explícitas antes de exigir cobertura de persistência. Nenhuma migration é aplicada automaticamente por este workflow.\n\nPara reproduzir, prepare o runtime indicado no README e execute o script dentro da aplicação. Ele não instala runtimes ou ferramentas globais. Node usa a versão da receita, Python usa uv 0.13.5/Python 3.13, Rust stable com fmt/Clippy e Go 1.27.3. O build Go de checks desativa VCS stamping para funcionar também sem Git próprio.\n\nRequired checks e regras de branch precisam ser configurados separadamente no GitHub após observar o primeiro run. Atualizações de runtime/SHA precisam de revisão e checks locais/remotos.\n");
    if files
        .insert(PathBuf::from(".github/workflows/ci.yml"), workflow)
        .is_some()
    {
        return Err(Error::operation("Workflow CI duplicado"));
    }
    files.insert(PathBuf::from("docs/CI.md"), guide);
    Ok(())
}
