use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=catalog");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut paths: Vec<_> = fs::read_dir(root.join("catalog"))
        .expect("catálogo deve existir")
        .map(|entry| entry.expect("entrada do catálogo").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    paths.sort();
    let mut source = String::from("pub const SOURCES: &[&str] = &[\n");
    for path in paths {
        source.push_str(&format!("include_str!({:?}),\n", path));
    }
    source.push_str("];\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("catalog.rs");
    fs::write(output, source).expect("gerar índice do catálogo");
}
