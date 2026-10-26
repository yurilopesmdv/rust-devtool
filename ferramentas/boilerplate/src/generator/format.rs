use super::integrations::Files;
use crate::Error;
use dprint_plugin_typescript::{
    FormatTextOptions,
    configuration::{ConfigurationBuilder, QuoteStyle},
    format_text,
};
use std::{
    collections::BTreeMap,
    io::Write,
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
};

/// Format composed sources before publishing, including code contributed by adapters.
pub fn apply(files: &mut Files) -> Result<(), Error> {
    let config = ConfigurationBuilder::new()
        .line_width(100)
        .indent_width(2)
        .quote_style(QuoteStyle::PreferSingle)
        .build();
    for (path, source) in files.iter_mut() {
        match path.extension().and_then(|s| s.to_str()) {
            Some("ts" | "tsx" | "js" | "mjs" | "jsx" | "cjs") => {
                for _ in 0..4 {
                    let result = format_text(FormatTextOptions {
                        path,
                        extension: None,
                        text: source.clone(),
                        config: &config,
                        external_formatter: None,
                    })
                    .map_err(|e| Error::operation(format!("Formatar {}: {e}", path.display())))?;
                    match result {
                        Some(formatted) if formatted != *source => *source = formatted,
                        _ => break,
                    }
                }
            }
            Some("go") => {
                let mut child = Command::new("gofmt")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| {
                        Error::operation(format!("gofmt necessário; instale Go 1.27.3: {e}"))
                    })?;
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(source.as_bytes())
                    .map_err(|e| Error::operation(e.to_string()))?;
                let output = child
                    .wait_with_output()
                    .map_err(|e| Error::operation(e.to_string()))?;
                if !output.status.success() {
                    return Err(Error::operation(format!(
                        "gofmt rejeitou {}",
                        path.display()
                    )));
                }
                *source = String::from_utf8(output.stdout)
                    .map_err(|e| Error::operation(e.to_string()))?;
            }
            Some("rs") => {
                static CACHE: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
                let cache = CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
                if let Some(formatted) = cache.lock().unwrap().get(source).cloned() {
                    *source = formatted;
                    continue;
                }
                let original = source.clone();
                let mut child = Command::new("rustfmt")
                    .args(["--edition", "2024", "--emit", "stdout", "--config", "skip_children=true"])
                    .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
                    .spawn().map_err(|e| Error::operation(format!("rustfmt necessário para formatar fontes Rust: {e}; instale com rustup component add rustfmt")))?;
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(source.as_bytes())
                    .map_err(|e| Error::operation(e.to_string()))?;
                let output = child
                    .wait_with_output()
                    .map_err(|e| Error::operation(e.to_string()))?;
                if !output.status.success() {
                    return Err(Error::operation(format!(
                        "rustfmt rejeitou {}: {}",
                        path.display(),
                        String::from_utf8_lossy(&output.stderr)
                    )));
                }
                *source = String::from_utf8(output.stdout)
                    .map_err(|e| Error::operation(e.to_string()))?;
                let mut cache = cache.lock().unwrap();
                if cache.len() < 1024 {
                    cache.insert(original, source.clone());
                }
            }
            _ => {}
        }
    }
    Ok(())
}
