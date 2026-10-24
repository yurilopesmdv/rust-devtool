use crate::Error;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn checked(command: &mut Command, label: &str) -> Result<(), Error> {
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(std::io::stderr()))
        .status()
        .map_err(|e| Error::operation(format!("{label}: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::operation(format!(
            "{label} falhou ({status}); arquivos do projeto foram preservados"
        )))
    }
}

pub fn exists(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn valid_node_version(version: &str) -> bool {
    let parts: Vec<_> = version.split('.').collect();
    !parts.is_empty()
        && parts.len() <= 3
        && parts.iter().all(|p| {
            !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit()) && p.parse::<u16>().is_ok()
        })
        && parts[0].parse::<u16>().is_ok_and(|v| v >= 22)
        && !(parts[0] == "22"
            && parts
                .get(1)
                .is_some_and(|v| v.parse::<u16>().unwrap_or(0) < 12))
}

pub fn matches_node(requested: &str, actual: &str) -> bool {
    let actual = actual.trim().trim_start_matches('v');
    valid_node_version(actual)
        && actual
            .split('.')
            .take(requested.split('.').count())
            .eq(requested.split('.'))
}

pub fn nvm_script() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("NVM_DIR") {
        return Some(PathBuf::from(dir).join("nvm.sh"));
    }
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        let script = PathBuf::from(dir).join("nvm/nvm.sh");
        if script.is_file() {
            return Some(script);
        }
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".nvm/nvm.sh"))
}

#[derive(Debug, Default)]
pub struct Runtime {
    pub node_bin: Option<PathBuf>,
}
impl Runtime {
    pub fn command(&self, program: &str) -> Command {
        let mut command = Command::new(if let Some(dir) = &self.node_bin {
            if program == "npm" || program == "node" {
                dir.join(program)
            } else {
                PathBuf::from(program)
            }
        } else {
            PathBuf::from(program)
        });
        if let Some(dir) = &self.node_bin {
            let mut paths = vec![dir.clone()];
            if let Some(path) = std::env::var_os("PATH") {
                paths.extend(std::env::split_paths(&path));
            }
            if let Ok(path) = std::env::join_paths(paths) {
                command.env("PATH", path);
            }
        }
        command
    }

    pub fn find_node(version: &str) -> Option<Self> {
        let result = Command::new("node")
            .arg("--version")
            .stdin(Stdio::null())
            .output();
        if let Ok(output) = result
            && output.status.success()
            && matches_node(version, &String::from_utf8_lossy(&output.stdout))
            && exists("npm")
        {
            return Some(Self::default());
        }
        let script = nvm_script()?;
        if !script.is_file() {
            return None;
        }
        let output = Command::new("bash")
            .args([
                "-c",
                "set -e; . \"$1\" --no-use; nvm which \"$2\"",
                "boilerplate",
            ])
            .arg(&script)
            .arg(version)
            .stdin(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let node = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        let dir = node.parent()?.to_path_buf();
        if !node.is_file() || !dir.join("npm").exists() {
            return None;
        }
        let runtime = Self {
            node_bin: Some(dir),
        };
        let output = runtime
            .command("node")
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .ok()?;
        (output.status.success() && matches_node(version, &String::from_utf8_lossy(&output.stdout)))
            .then_some(runtime)
    }

    pub fn install_node(version: &str) -> Result<Self, Error> {
        let script = nvm_script().ok_or_else(|| {
            Error::operation("HOME/NVM_DIR não definido; configure nvm manualmente")
        })?;
        if !script.is_file() {
            let destination = script.parent().unwrap();
            if destination.exists() {
                return Err(Error::operation(
                    "Diretório nvm já existe sem nvm.sh; corrija manualmente, nada será sobrescrito",
                ));
            }
            if !exists("git") {
                return Err(Error::operation("Instale git antes de instalar nvm"));
            }
            checked(
                Command::new("git")
                    .args([
                        "clone",
                        "--depth",
                        "1",
                        "--branch",
                        "v0.40.8",
                        "https://github.com/nvm-sh/nvm.git",
                    ])
                    .arg(destination),
                "Instalar nvm",
            )?;
        }
        // Fixed shell source; version and paths are arguments, never interpolated as shell code.
        checked(
            Command::new("bash")
                .args([
                    "-c",
                    "set -e; . \"$1\" --no-use; nvm install \"$2\"",
                    "boilerplate",
                ])
                .arg(&script)
                .arg(version),
            "Instalar Node via nvm",
        )?;
        Self::find_node(version).ok_or_else(|| {
            Error::operation("Node instalado não foi localizado; carregue nvm e tente novamente")
        })
    }
}

pub fn install(
    app: &crate::planner::Application,
    runtime: &Runtime,
    template: &crate::catalog::Template,
) -> Result<(), Error> {
    let Some((program, args)) = template.install.split_first() else {
        return Ok(());
    };
    let mut args = args.to_vec();
    // A fresh Rust project has no lockfile yet. Subsequent invocations can use --locked.
    if program == "cargo" && !app.destination.join("Cargo.lock").exists() {
        args.retain(|a| a != "--locked");
    }
    if program == "cargo"
        && args.first().is_some_and(|arg| arg == "fetch")
        && !args.iter().any(|arg| arg == "--target")
    {
        let output = Command::new("rustc")
            .arg("-vV")
            .stdin(Stdio::null())
            .output()
            .map_err(|e| Error::operation(format!("Verificar toolchain Rust: {e}")))?;
        let info = String::from_utf8_lossy(&output.stdout);
        let host = info
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .filter(|host| host.contains("-linux-"))
            .ok_or_else(|| {
                Error::operation("A instalação exige uma toolchain Rust Linux dentro do WSL")
            })?;
        args.extend(["--target".to_string(), host.to_string()]);
    }
    checked(
        runtime
            .command(program)
            .args(args)
            .current_dir(&app.destination),
        &format!("Instalar dependências em {}", app.destination.display()),
    )
}

pub fn check_git_identity(path: &Path) -> Result<(), Error> {
    for key in ["user.name", "user.email"] {
        let output = Command::new("git")
            .args(["config", "--get", key])
            .current_dir(path)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| Error::operation(e.to_string()))?;
        if !output.status.success() || output.stdout.iter().all(u8::is_ascii_whitespace) {
            return Err(Error::operation(format!(
                "Configure git {key} antes de fazer commit/push; nenhuma configuração global foi alterada"
            )));
        }
    }
    Ok(())
}
