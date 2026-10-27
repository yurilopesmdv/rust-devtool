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
    if template.role != ProjectType::Backend
        || recipe.backend.migrations != Some(Migrations::Dbmate)
    {
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
