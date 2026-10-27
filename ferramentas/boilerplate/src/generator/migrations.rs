use super::integrations::{Files, insert};
use crate::{
    Error,
    catalog::Template,
    config::{Migrations, ProjectType, Recipe},
};
use std::path::Path;

pub fn apply(
    recipe: &Recipe,
    template: &Template,
    files: &mut Files,
    prefix: &Path,
) -> Result<(), Error> {
    if template.role != ProjectType::Backend {
        return Ok(());
    }
    if recipe.backend.migrations == Some(Migrations::MongodbNative) {
        return mongo(template, files, prefix);
    }
    if recipe.backend.migrations != Some(Migrations::Dbmate) {
        return Ok(());
    }
    insert(
        files,
        prefix,
        "scripts/_pg_client.py",
        include_str!("../../integrations/migrations/pg_client.py"),
    );
    if let Some(ignore) = files.get_mut(&prefix.join(".dockerignore")) {
        ignore.push_str("\n.tools/\n");
    }
    insert(
        files,
        prefix,
        "scripts/migrate.sh",
        include_str!("../../integrations/migrations/migrate.sh"),
    );
    insert(
        files,
        prefix,
        "migrations/sql/20261027000100_create_examples.sql",
        include_str!("../../integrations/migrations/initial.sql"),
    );
    insert(
        files,
        prefix,
        "docs/MIGRATIONS.md",
        include_str!("../../integrations/migrations/README.md"),
    );
    Ok(())
}

fn mongo(template: &Template, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    let (path, source) = match template.package_manager.as_str() {
        "npm" => (
            "scripts/migrate.mjs",
            include_str!("../../integrations/migrations/mongo/migrate.mjs"),
        ),
        "uv" => (
            "scripts/migrate.py",
            include_str!("../../integrations/migrations/mongo/migrate.py"),
        ),
        "cargo" => (
            "src/bin/migrate.rs",
            include_str!("../../integrations/migrations/mongo/migrate.rs"),
        ),
        _ => return Err(Error::input("Stack sem executor nativo Mongo")),
    };
    insert(files, prefix, path, source);
    insert(
        files,
        prefix,
        "migrations/mongo/20261027000100_create_examples.json",
        include_str!("../../integrations/migrations/mongo/initial.json"),
    );
    insert(
        files,
        prefix,
        "docs/MIGRATIONS.md",
        include_str!("../../integrations/migrations/mongo/README.md"),
    );
    if template.package_manager == "cargo" {
        let source = files
            .get_mut(&prefix.join("Cargo.toml"))
            .ok_or_else(|| Error::operation("Cargo.toml ausente"))?;
        let mut manifest: toml::Table = source
            .parse()
            .map_err(|e: toml::de::Error| Error::operation(e.to_string()))?;
        let package = manifest
            .get_mut("package")
            .and_then(toml::Value::as_table_mut)
            .ok_or_else(|| Error::operation("Cargo package ausente"))?;
        let name = package
            .get("name")
            .cloned()
            .ok_or_else(|| Error::operation("Cargo name ausente"))?;
        package.insert("default-run".into(), name);
        *source = toml::to_string_pretty(&manifest).map_err(|e| Error::operation(e.to_string()))?;
    }
    Ok(())
}
