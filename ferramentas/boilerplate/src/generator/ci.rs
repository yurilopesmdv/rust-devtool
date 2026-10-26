use super::integrations::Files;
use crate::{Error, config::Ci, planner::Plan};
use std::path::{Path, PathBuf};

pub fn apply(plan: &Plan, root: &Path, files: &mut Files) -> Result<(), Error> {
    if plan.configuration.infrastructure.ci != Some(Ci::GithubActions) {
        return Ok(());
    }
    let Some(app) = plan
        .applications
        .iter()
        .find(|app| app.template == "express-ts" && app.destination.starts_with(root))
    else {
        return Ok(());
    };
    let relative = app
        .destination
        .strip_prefix(root)
        .map_err(|e| Error::operation(e.to_string()))?;
    let directory = if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        relative.to_string_lossy().to_string()
    };
    let workflow = include_str!("../../integrations/ci/express.yml")
        .replace("@@WORKING_DIRECTORY@@", &directory)
        .replace(
            "@@NODE_VERSION@@",
            &plan.configuration.toolchain.node_version,
        );
    let path = PathBuf::from(".github/workflows/ci.yml");
    if files.insert(path, workflow).is_some() {
        return Err(Error::operation("Workflow CI duplicado"));
    }
    files.insert(PathBuf::from("docs/CI.md"),format!("# CI da API\n\nGitHub Actions executa npm ci e npm run check em `{directory}` com Node da receita. Actions fixadas por SHA; contents:read e credenciais do checkout desativadas. Não há deploy, publicação, registry login, push ou uso de segredos.\n\nEm fullstack esta primeira opção verifica somente a API Express. Dependências externas não são provisionadas: testes que exigem DATABASE_URL são pulados sem URL de teste. Acrescente job de persistência com serviços descartáveis antes de exigir cobertura de banco. Redis/Vault/monitoria não são iniciados.\n\nPara reproduzir localmente, entre no diretório da API e execute npm ci e npm run check. Atualize versões/SHA com revisão e teste. Regras de branch/required checks são configuração separada no GitHub; gerar o arquivo não as ativa.\n"));
    Ok(())
}
