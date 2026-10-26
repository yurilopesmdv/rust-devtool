use super::*;

pub fn apply(r: &Recipe, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    if r.backend.database != Some(Database::Postgres) {
        return Err(Error::input("Go suporta somente PostgreSQL neste adapter"));
    }
    let module = source_mut(files, prefix, "go.mod")?
        .lines()
        .find_map(|line| line.strip_prefix("module "))
        .ok_or_else(|| Error::operation("go.mod sem module"))?
        .to_string();
    insert(
        files,
        prefix,
        "go.mod",
        include_str!("../../../integrations/go/postgres.mod").replace("{{PROJECT_NAME}}", &module),
    );
    insert(
        files,
        prefix,
        "go.sum",
        include_str!("../../../integrations/go/postgres.sum"),
    );
    for (path, source) in [
        (
            "internal/infrastructure/database/client.go",
            include_str!("../../../integrations/go/database.go"),
        ),
        (
            "internal/infrastructure/database/schema.go",
            include_str!("../../../integrations/go/schema.go"),
        ),
        (
            "internal/modules/example/repositories/repository.go",
            include_str!("../../../integrations/go/repository.go"),
        ),
        (
            "internal/modules/example/service.go",
            include_str!("../../../integrations/go/service.go"),
        ),
        (
            "cmd/prepare/main.go",
            include_str!("../../../integrations/go/prepare.go"),
        ),
        (
            "internal/httpapi/persistence_test.go",
            include_str!("../../../integrations/go/persistence_test.go"),
        ),
    ] {
        insert(
            files,
            prefix,
            path,
            source.replace("{{PROJECT_NAME}}", &module),
        );
    }
    replace(
        files,
        prefix,
        "cmd/api/main.go",
        &format!("\t\"{module}/internal/config\""),
        &format!(
            "\t\"{module}/internal/config\"\n\t\"{module}/internal/infrastructure/database\"\n\t\"{module}/internal/modules/example/repositories\""
        ),
    )?;
    replace(
        files,
        prefix,
        "cmd/api/main.go",
        "service := example.NewService()",
        "pool, err := database.Open(ctx)\nif err != nil { return err }\ndefer pool.Close()\nservice := example.NewService(repositories.Repository{Pool:pool})",
    )?;
    replace(
        files,
        prefix,
        "cmd/api/main.go",
        "httpapi.New(service, nil, cfg.FrontendOrigin)",
        "httpapi.New(service, pool.Ping, cfg.FrontendOrigin)",
    )?;
    replace(
        files,
        prefix,
        "internal/modules/example/controller.go",
        "\"encoding/json\"",
        "\"encoding/json\"\n\"io\"",
    )?;
    source_mut(files, prefix, "internal/modules/example/controller.go")?
        .push_str(include_str!("../../../integrations/go/write-controller.go"));
    replace(
        files,
        prefix,
        "internal/httpapi/router.go",
        "// @boilerplate:routes",
        "mux.HandleFunc(\"POST /api/v1/example\",controller.Write)",
    )?;
    insert(
        files,
        prefix,
        "migrations/001_examples.sql",
        "CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL);\n-- Preparação demonstrativa explícita; sem histórico de migrations.\n",
    );
    if files.contains_key(&prefix.join("Dockerfile")) {
        replace(
            files,
            prefix,
            "Dockerfile",
            "# @boilerplate:prepare-build",
            "RUN CGO_ENABLED=0 go build -trimpath -buildvcs=false -o /prepare ./cmd/prepare",
        )?;
        replace(
            files,
            prefix,
            "Dockerfile",
            "# @boilerplate:prepare-copy",
            "COPY --from=build /prepare /app/prepare",
        )?;
    }
    Ok(())
}
