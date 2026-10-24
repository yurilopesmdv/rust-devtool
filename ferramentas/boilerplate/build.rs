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
    println!("cargo:rerun-if-changed=templates");
    source.push_str("pub const ASSETS: &[(&str, &str, &str)] = &[\n");
    let assets_root = root.join("templates");
    let mut assets = Vec::new();
    collect(&assets_root, &mut assets);
    assets.sort();
    for path in assets {
        let relative = path.strip_prefix(&assets_root).unwrap();
        let mut parts = relative.components();
        let id = parts.next().unwrap().as_os_str().to_str().unwrap();
        let name = parts.as_path().to_str().unwrap();
        source.push_str(&format!("({id:?}, {name:?}, include_str!({path:?})),\n"));
    }
    source.push_str("];\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("catalog.rs");
    fs::write(output, source).expect("gerar índice do catálogo");
}

fn collect(root: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("templates devem existir") {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, files);
        } else {
            files.push(path);
        }
    }
}
