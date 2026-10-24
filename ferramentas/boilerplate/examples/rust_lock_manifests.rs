use rust_boilerplate::{config::Database, generator::integrations::rust::manifest};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = include_str!("../templates/rust-axum/Cargo.toml")
        .replace("{{PROJECT_NAME}}", "rust-lock-fixture");
    let mut manifests = BTreeMap::new();
    for (label, database) in [
        ("none", Database::None),
        ("postgres", Database::Postgres),
        ("mongodb", Database::Mongodb),
    ] {
        for cache in [false, true] {
            for metrics in [false, true] {
                let key = format!("{label}-{}-{}", u8::from(cache), u8::from(metrics));
                manifests.insert(
                    key,
                    toml::to_string_pretty(&manifest(&source, database, cache, metrics)?)?,
                );
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&manifests)?);
    Ok(())
}
