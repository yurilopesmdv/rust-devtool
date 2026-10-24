use super::{
    runtime::{check_git_identity, checked, exists},
    wizard,
};
use crate::{
    Error,
    cli::{Decision, NewArgs},
};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

pub fn valid_remote(value: &str) -> bool {
    let parts: Vec<_> = value.split('/').collect();
    (parts.len() == 1 || parts.len() == 2)
        && parts.iter().all(|p| {
            !p.is_empty()
                && !p.starts_with('-')
                && *p != "."
                && *p != ".."
                && p.len() <= 100
                && p.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
}

pub fn validate(args: &NewArgs, split: bool) -> Result<(), Error> {
    if args.git == Decision::No
        && (args.github == Some(Decision::Yes) || args.push == Some(Decision::Yes))
    {
        return Err(Error::input("GitHub/push exigem --git yes"));
    }
    if args.push == Some(Decision::Yes) && args.github == Some(Decision::No) {
        return Err(Error::input("--push yes exige GitHub selecionado"));
    }
    if args.push == Some(Decision::Yes)
        && !wizard::interactive(args)
        && args.github != Some(Decision::Yes)
    {
        return Err(Error::input("Em automação, --push yes exige --github yes"));
    }
    if split && args.github_repo.is_some() {
        return Err(Error::input("Split usa --github-web e --github-api"));
    }
    if !split && (args.github_web.is_some() || args.github_api.is_some()) {
        return Err(Error::input("--github-web/--github-api exigem split"));
    }
    for name in [&args.github_repo, &args.github_web, &args.github_api]
        .into_iter()
        .flatten()
    {
        if !valid_remote(name) {
            return Err(Error::input(
                "Nome remoto deve ser repo ou owner/repo, sem espaços, URLs ou opções",
            ));
        }
    }
    if args.git == Decision::Yes && !exists("git") {
        return Err(Error::operation(
            "Git não está instalado; instale-o ou use --git no. Nenhum arquivo foi criado.",
        ));
    }
    Ok(())
}

pub fn init(roots: &[PathBuf]) -> Result<(), Error> {
    for root in roots {
        checked(
            Command::new("git")
                .args(["init", "--initial-branch=main"])
                .current_dir(root),
            "Inicializar Git local",
        )?;
    }
    Ok(())
}

pub fn github(args: &NewArgs, roots: &[PathBuf], names: &[String]) -> Result<(), Error> {
    if args.git == Decision::No {
        return Ok(());
    }
    let interactive = wizard::interactive(args);
    let selected = match args.github {
        Some(d) => d == Decision::Yes,
        None if interactive => wizard::decision(
            "Criar repositório(s) privado(s) no GitHub para os fontes gerados?",
            false,
        )?,
        _ => false,
    };
    if !selected {
        return Ok(());
    }
    if !exists("gh") {
        return Err(Error::operation(
            "gh não instalado. Instale GitHub CLI no WSL; os projetos locais estão preservados.",
        ));
    }
    let authenticated = Command::new("gh")
        .args(["auth", "status", "--hostname", "github.com"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !authenticated {
        return Err(Error::operation(
            "gh não está autenticado em github.com. Execute gh auth login; os projetos locais estão preservados.",
        ));
    }
    let push = match args.push {
        Some(d) => d == Decision::Yes,
        None if interactive => {
            wizard::decision("Fazer commit inicial e push dos arquivos gerados?", false)?
        }
        _ => false,
    };
    // All identities are checked before creating the first remote repository.
    if push {
        for root in roots {
            check_git_identity(root)?;
        }
    }
    for (root, name) in roots.iter().zip(names) {
        let name = if interactive {
            wizard::line(
                "Nome GitHub (repo ou owner/repo; visibilidade privada)",
                Some(name),
            )?
        } else {
            name.clone()
        };
        if !valid_remote(&name) {
            return Err(Error::input(
                "Nome de repositório GitHub inválido; fontes preservados",
            ));
        }
        if push {
            checked(
                Command::new("git").args(["add", "--all"]).current_dir(root),
                "Preparar commit inicial",
            )?;
            checked(
                Command::new("git")
                    .args(["commit", "-m", "chore: initialize project"])
                    .current_dir(root),
                "Criar commit inicial",
            )?;
        }
        let mut command = Command::new("gh");
        command
            .args([
                "repo",
                "create",
                &name,
                "--private",
                "--source",
                ".",
                "--remote",
                "origin",
            ])
            .current_dir(root);
        if push {
            command.arg("--push");
        }
        checked(&mut command, &format!("Criar repositório privado {name}"))?;
    }
    Ok(())
}
