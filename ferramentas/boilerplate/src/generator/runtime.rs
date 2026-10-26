use crate::Error;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

// Serializes external operations so a termination signal belongs to one active group.
static PROCESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static SIGNAL: std::sync::OnceLock<std::sync::Arc<std::sync::atomic::AtomicU8>> =
    std::sync::OnceLock::new();

fn signals() -> Result<&'static std::sync::Arc<std::sync::atomic::AtomicU8>, Error> {
    use std::sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    };
    if let Some(signal) = SIGNAL.get() {
        return Ok(signal);
    }
    let state = Arc::new(AtomicU8::new(255));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        let state = state.clone();
        // SAFETY: callback performs only an atomic store and async-signal-safe default handling.
        unsafe {
            signal_hook::low_level::register(signal, move || {
                if state.load(Ordering::SeqCst) == 255 {
                    let _ = signal_hook::low_level::emulate_default_handler(signal);
                } else {
                    state.store(signal as u8, Ordering::SeqCst);
                }
            })
        }
        .map_err(|e| Error::operation(format!("Registrar sinais: {e}")))?;
    }
    let _ = SIGNAL.set(state);
    Ok(SIGNAL.get().unwrap())
}

pub fn checked(command: &mut Command, label: &str) -> Result<(), Error> {
    let seconds = std::env::var("BOILERPLATE_COMMAND_TIMEOUT_SECONDS")
        .ok()
        .map(|s| s.parse::<u64>().ok().filter(|n| *n > 0))
        .unwrap_or(Some(900))
        .ok_or_else(|| {
            Error::input("BOILERPLATE_COMMAND_TIMEOUT_SECONDS deve ser inteiro positivo")
        })?;
    checked_timeout(command, label, std::time::Duration::from_secs(seconds))
}

pub fn checked_timeout(
    command: &mut Command,
    label: &str,
    timeout: std::time::Duration,
) -> Result<(), Error> {
    use rustix::process::{Pid, Signal, kill_process_group};
    use std::{
        os::unix::process::CommandExt,
        sync::atomic::Ordering,
        time::{Duration, Instant},
    };
    let _lock = PROCESS_LOCK
        .lock()
        .map_err(|_| Error::operation("Executor indisponível"))?;
    let signal = signals()?;
    signal.store(0, Ordering::SeqCst);
    struct Reset<'a>(&'a std::sync::atomic::AtomicU8);
    impl Drop for Reset<'_> {
        fn drop(&mut self) {
            self.0.store(255, Ordering::SeqCst);
        }
    }
    let _reset = Reset(signal);
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(std::io::stderr()))
        .process_group(0)
        .spawn()
        .map_err(|e| Error::operation(format!("{label}: {e}; arquivos preservados")))?;
    let pid = Pid::from_raw(child.id() as i32).unwrap();
    let started = Instant::now();
    loop {
        let interrupted = signal.load(Ordering::SeqCst);
        if interrupted != 0 || started.elapsed() >= timeout {
            let _ = kill_process_group(pid, Signal::TERM);
            let deadline = Instant::now() + Duration::from_millis(500);
            while Instant::now() < deadline {
                let _ = child.try_wait();
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = kill_process_group(pid, Signal::KILL);
            let _ = child.wait();
            return Err(Error {
                code: if interrupted == 0 {
                    124
                } else {
                    128 + interrupted
                },
                message: format!(
                    "{label}: {}; arquivos preservados. Retome o comando de instalação indicado no README da aplicação, dentro do diretório do projeto.",
                    if interrupted == 0 {
                        "timeout"
                    } else {
                        "interrompido"
                    }
                ),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                return Err(Error::operation(format!(
                    "{label} falhou ({status}); arquivos preservados; retome pelo README da aplicação"
                )));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => {
                let _ = kill_process_group(pid, Signal::KILL);
                let _ = child.wait();
                return Err(Error::operation(format!(
                    "{label}: {e}; arquivos preservados"
                )));
            }
        }
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
    if program == "uv" && !exists("uv") {
        return Err(Error::operation(
            "uv não está instalado. Instale no WSL seguindo https://docs.astral.sh/uv/getting-started/installation/ e execute uv sync --locked na aplicação; os arquivos foram preservados",
        ));
    }
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

#[cfg(test)]
mod process_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    #[ignore = "subprocess fixture; invoked by bounded_processes"]
    fn probe() {
        let path = std::env::var("BOILERPLATE_PROBE_PID").unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 60 & echo $! > \"$1\"; wait", "probe", &path]);
        let timeout = if std::env::var_os("BOILERPLATE_PROBE_SIGNAL").is_some() {
            60000
        } else {
            200
        };
        let error =
            checked_timeout(&mut command, "probe", Duration::from_millis(timeout)).unwrap_err();
        std::process::exit(error.code.into());
    }

    #[test]
    fn bounded_processes() {
        use rustix::process::{Pid, Signal, kill_process};
        for (signal, expected) in [
            (None, 124),
            (Some(Signal::INT), 130),
            (Some(Signal::TERM), 143),
        ] {
            let path = std::env::temp_dir().join(format!(
                "boilerplate-process-{}-{expected}",
                std::process::id()
            ));
            let _ = std::fs::remove_file(&path);
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "generator::runtime::process_tests::probe",
                    "--ignored",
                ])
                .env("BOILERPLATE_PROBE_PID", &path);
            if signal.is_some() {
                command.env("BOILERPLATE_PROBE_SIGNAL", "yes");
            }
            let mut child = command.spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !path.exists() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(path.exists(), "child did not start");
            let pid: u32 = std::fs::read_to_string(&path)
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            if let Some(signal) = signal {
                kill_process(Pid::from_raw(child.id() as i32).unwrap(), signal).unwrap();
            }
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    assert_eq!(status.code(), Some(expected));
                    break;
                }
                assert!(Instant::now() < deadline, "executor hung");
                std::thread::sleep(Duration::from_millis(10));
            }
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
            assert!(
                stat.is_empty() || stat.split_whitespace().nth(2) == Some("Z"),
                "grandchild still running"
            );
            std::fs::remove_file(path).unwrap();
        }
    }
}
