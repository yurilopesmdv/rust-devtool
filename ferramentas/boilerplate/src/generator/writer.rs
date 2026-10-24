use crate::Error;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct ProjectFiles {
    pub destination: PathBuf,
    pub files: BTreeMap<PathBuf, String>,
}

struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Rendered sources are staged before publication. Published projects are never rolled back.
pub fn publish(projects: &[ProjectFiles]) -> Result<Vec<PathBuf>, Error> {
    let mut staged = Vec::new();
    for project in projects {
        if fs::symlink_metadata(&project.destination).is_ok() {
            return Err(Error::input(format!(
                "Destino já existe: {}",
                project.destination.display()
            )));
        }
        let parent = project
            .destination
            .parent()
            .ok_or_else(|| Error::input("Destino sem diretório pai"))?;
        fs::create_dir_all(parent)
            .map_err(|e| Error::operation(format!("Criar diretório pai: {e}")))?;
        let stage = loop {
            let path = parent.join(format!(
                ".boilerplate-stage-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break Stage(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(Error::operation(format!("Criar staging: {e}"))),
            }
        };
        for (relative, content) in &project.files {
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_)))
            {
                return Err(Error::input("Caminho de template não é relativo e seguro"));
            }
            let path = stage.0.join(relative);
            fs::create_dir_all(path.parent().unwrap())
                .map_err(|e| Error::operation(e.to_string()))?;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| Error::operation(format!("Criar arquivo: {e}")))?;
            file.write_all(content.as_bytes())
                .map_err(|e| Error::operation(e.to_string()))?;
        }
        staged.push((stage, &project.destination));
    }
    let mut published = Vec::new();
    for (stage, destination) in &staged {
        rustix::fs::renameat_with(
            rustix::fs::CWD,
            &stage.0,
            rustix::fs::CWD,
            *destination,
            rustix::fs::RenameFlags::NOREPLACE,
        )
        .map_err(|e| {
            Error::operation(format!(
                "Publicação de {} falhou: {e}. Projetos já publicados preservados: {published:?}",
                destination.display()
            ))
        })?;
        published.push(destination.to_path_buf());
    }
    Ok(published)
}

pub fn safe_path(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
}
