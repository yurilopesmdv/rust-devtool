use super::*;

pub fn apply(r: &Recipe, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    let db = r.backend.database.unwrap_or(Database::None);
    let cache = r.backend.cache == Some(Cache::Redis);
    let metrics = r.infrastructure.monitoring == Some(Monitoring::PrometheusGrafana);
    let vault = r.infrastructure.secrets == Some(Secrets::Vault);
    let mut manifest: toml::Table = files[&prefix.join("pyproject.toml")]
        .parse()
        .map_err(|e: toml::de::Error| Error::operation(e.to_string()))?;
    let deps = manifest["project"]["dependencies"].as_array_mut().unwrap();
    if db == Database::Postgres {
        deps.extend(
            ["psycopg[binary]==3.3.6", "psycopg-pool==3.3.1"]
                .map(|s| toml::Value::String(s.into())),
        );
    } else if db == Database::Mongodb {
        deps.push(toml::Value::String("pymongo==4.19.0".into()));
    }
    if cache {
        deps.push(toml::Value::String("redis==8.1.0".into()));
    }
    if metrics {
        deps.push(toml::Value::String("prometheus-client==0.26.0".into()));
    }
    insert(
        files,
        prefix,
        "pyproject.toml",
        toml::to_string_pretty(&manifest).map_err(|e| Error::operation(e.to_string()))?,
    );
    let key = format!(
        "{}-{}-{}",
        match db {
            Database::None => "none",
            Database::Postgres => "postgres",
            Database::Mongodb => "mongodb",
        },
        u8::from(cache),
        u8::from(metrics)
    );
    let lock = crate::catalog::PYTHON_LOCKS
        .iter()
        .find(|(id, _)| *id == key)
        .ok_or_else(|| Error::operation(format!("Lock Python ausente: {key}")))?
        .1;
    let name = manifest["project"]["name"].as_str().unwrap();
    insert(
        files,
        prefix,
        "uv.lock",
        lock.replace("{{PROJECT_NAME}}", name),
    );
    if vault {
        insert(
            files,
            prefix,
            "app/config.py",
            include_str!("../../../integrations/python/vault-config.py"),
        );
    }
    let mut imports = String::from("from contextlib import asynccontextmanager\n\n");
    if db != Database::None || cache || vault {
        imports.push_str("from app.config import read_config\n");
    }
    let mut initialize = String::new();
    let mut cleanup = String::new();
    if db != Database::None {
        imports.push_str("from app.infrastructure.database import Database\n");
        initialize.push_str("        app.state.database = Database(read_config(\"DATABASE_URL\"))\n        await app.state.database.open()\n");
        cleanup.push_str("        if hasattr(app.state, \"database\"):\n            await app.state.database.close()\n");
        insert(
            files,
            prefix,
            "app/infrastructure/database.py",
            if db == Database::Postgres {
                include_str!("../../../integrations/python/postgres.py")
            } else {
                include_str!("../../../integrations/python/mongodb.py")
            },
        );
        insert(
            files,
            prefix,
            "app/modules/example/repository.py",
            if db == Database::Postgres {
                include_str!("../../../integrations/python/postgres-repository.py")
            } else {
                include_str!("../../../integrations/python/mongodb-repository.py")
            },
        );
        insert(
            files,
            prefix,
            "app/infrastructure/schema.py",
            if db == Database::Postgres {
                include_str!("../../../integrations/python/postgres-schema.py")
            } else {
                include_str!("../../../integrations/python/mongodb-schema.py")
            },
        );
        insert(
            files,
            prefix,
            "app/infrastructure/prepare.py",
            "import asyncio\n\nfrom app.config import read_config\nfrom app.infrastructure.database import Database\nfrom app.infrastructure.schema import prepare\n\n\nasync def main():\n    database = Database(read_config(\"DATABASE_URL\"))\n    try:\n        await database.open()\n        await prepare(database)\n    finally:\n        await database.close()\n\n\nif __name__ == \"__main__\":\n    asyncio.run(main())\n",
        );
        if db == Database::Postgres {
            insert(
                files,
                prefix,
                "migrations/001_examples.sql",
                "CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL);\n",
            );
        }
    }
    if cache {
        imports.push_str("from app.infrastructure.cache import Cache\n");
        initialize.push_str("        app.state.cache = Cache(read_config(\"REDIS_URL\"))\n");
        cleanup.push_str("        if hasattr(app.state, \"cache\"):\n            await app.state.cache.close()\n");
        insert(
            files,
            prefix,
            "app/infrastructure/cache.py",
            include_str!("../../../integrations/python/cache.py"),
        );
    }
    if vault && db == Database::None && !cache {
        initialize.push_str("        read_config(\"APP_SECRET\")\n");
    }
    if cleanup.is_empty() {
        cleanup.push_str("        pass\n");
    }
    imports = imports.replace("from app.infrastructure.database import Database\nfrom app.infrastructure.cache import Cache", "from app.infrastructure.cache import Cache\nfrom app.infrastructure.database import Database");
    insert(
        files,
        prefix,
        "app/infrastructure/lifecycle.py",
        format!(
            "{}\n\n\n@asynccontextmanager\nasync def lifespan(app):\n    try:\n{initialize}        yield\n    finally:\n{cleanup}",
            imports.trim_end()
        ),
    );
    replace(
        files,
        prefix,
        "app/main.py",
        "from app.modules.example.router import router",
        "from app.infrastructure.lifecycle import lifespan\nfrom app.modules.example.router import router",
    )?;
    replace(
        files,
        prefix,
        "app/main.py",
        "app = FastAPI(",
        "app = FastAPI(lifespan=lifespan, ",
    )?;
    if db != Database::None {
        replace(
            files,
            prefix,
            "app/main.py",
            "from fastapi import FastAPI",
            "from fastapi import FastAPI, HTTPException, Request",
        )?;
        replace(
            files,
            prefix,
            "app/main.py",
            "async def ready() -> dict[str, str]:\n        return {\"status\": \"ok\"}",
            "async def ready(request: Request) -> dict[str, str]:\n        try:\n            await request.app.state.database.ready()\n        except Exception:\n            raise HTTPException(status_code=503, detail=\"Dependência indisponível\") from None\n        return {\"status\": \"ok\"}",
        )?;
    }
    if metrics {
        insert(
            files,
            prefix,
            "app/infrastructure/metrics.py",
            include_str!("../../../integrations/python/metrics.py"),
        );

        replace(
            files,
            prefix,
            "app/main.py",
            "from app.infrastructure.lifecycle import lifespan",
            "from app.infrastructure.lifecycle import lifespan\nfrom app.infrastructure.metrics import configure_metrics",
        )?;
        replace(
            files,
            prefix,
            "app/main.py",
            "    app.include_router(router)",
            "    app.include_router(router)\n    configure_metrics(app)",
        )?;
    }
    let mut source = String::new();
    if db != Database::None || cache {
        source.push_str("from fastapi import Request\n");
    }
    if cache {
        source.push_str("from pydantic import ValidationError\n");
    }
    if db != Database::None || cache {
        source.push('\n');
    }
    if cache {
        source.push_str("from app.infrastructure.cache import Cache\n");
    }
    if db != Database::None {
        source.push_str("from app.modules.example.repository import ExampleRepository\n");
    }
    source.push_str(
        "from app.modules.example.schemas import ExampleResponse\n\n\nclass ExampleService:\n",
    );
    let mut parameters = vec![];
    let mut arguments = vec![];
    if db != Database::None {
        parameters.push("repository: ExampleRepository");
        arguments.push("ExampleRepository(request.app.state.database)");
    }
    if cache {
        parameters.push("cache: Cache");
        arguments.push("request.app.state.cache");
    }
    if !parameters.is_empty() {
        source.push_str(&format!(
            "    def __init__(self, {}):\n",
            parameters.join(", ")
        ));
        if db != Database::None {
            source.push_str("        self.repository = repository\n");
        }
        if cache {
            source.push_str("        self.cache = cache\n");
        }
        source.push('\n');
    }
    let key = format!("{name}:example");
    source.push_str("    async def get(self) -> ExampleResponse:\n");
    if cache {
        source.push_str(&format!("        cached = await self.cache.get(\"{key}\")\n        if cached:\n            try:\n                return ExampleResponse.model_validate_json(cached)\n            except ValidationError:\n                pass\n"));
    }
    source.push_str(if db != Database::None {
        "        result = await self.repository.read()\n"
    } else {
        "        result = ExampleResponse(message=\"API funcionando\")\n"
    });
    if cache {
        source.push_str(&format!(
            "        await self.cache.set(\"{key}\", result.model_dump_json())\n"
        ));
    }
    source.push_str("        return result\n");
    if db != Database::None {
        source.push_str("\n    async def write(self, message: str) -> ExampleResponse:\n        result = await self.repository.write(message)\n");
        if cache {
            source.push_str(&format!("        await self.cache.remove(\"{key}\")\n"));
        }
        source.push_str("        return result\n");
        replace(
            files,
            prefix,
            "app/modules/example/router.py",
            "import ExampleResponse",
            "import ExampleInput, ExampleResponse",
        )?;
        source_mut(files, prefix, "app/modules/example/router.py")?.push_str("\n\n@router.post(\"\", response_model=ExampleResponse)\nasync def write_example(body: ExampleInput, service: Service) -> ExampleResponse:\n    return await service.write(body.message)\n");
    }
    source.push_str(&format!(
        "\n\ndef get_service({}) -> ExampleService:\n    return ExampleService({})\n",
        if arguments.is_empty() {
            ""
        } else {
            "request: Request"
        },
        arguments.join(", ")
    ));
    insert(files, prefix, "app/modules/example/service.py", source);
    if db != Database::None || vault {
        let test = source_mut(files, prefix, "tests/test_api.py")?;
        let condition = if db != Database::None {
            "not (os.environ.get(\"DATABASE_URL\") or os.environ.get(\"DATABASE_URL_FILE\"))"
        } else if cache {
            "not (os.environ.get(\"REDIS_URL\") or os.environ.get(\"REDIS_URL_FILE\"))"
        } else {
            "not (os.environ.get(\"APP_SECRET\") or os.environ.get(\"APP_SECRET_FILE\"))"
        };
        test.insert_str(0, "import os\n\nimport pytest\n");
        replace_source(
            test,
            "tests/test_api.py",
            "def test_health_and_example():",
            &format!(
                "@pytest.mark.skipif(\n    {condition},\n    reason=\"Requer serviços/segredos de teste\",\n)\ndef test_health_and_example():"
            ),
        )?;
    }
    if cache && db == Database::None && !vault {
        let test = source_mut(files, prefix, "tests/test_api.py")?;
        test.insert_str(0, "import os\n\n");
        replace_source(
            test,
            "tests/test_api.py",
            "def test_health_and_example():",
            "def test_health_and_example(monkeypatch):\n    monkeypatch.setenv(\"REDIS_URL\", os.environ.get(\"REDIS_URL\", \"redis://127.0.0.1:9\"))",
        )?;
    }
    if db != Database::None {
        insert(
            files,
            prefix,
            "tests/test_readiness.py",
            include_str!("../../../integrations/python/test-readiness.py"),
        );
        source_mut(files, prefix, "tests/test_api.py")?.push_str("\n\n@pytest.mark.skipif(\n    not (os.environ.get(\"DATABASE_URL\") or os.environ.get(\"DATABASE_URL_FILE\")),\n    reason=\"Requer banco de teste preparado\",\n)\ndef test_persistence():\n    with TestClient(create_app()) as client:\n        previous = client.get(\"/api/v1/example\").json()\n        try:\n            assert client.post(\"/api/v1/example\", json={\"message\": \"persistido\"}).status_code == 200\n            assert client.get(\"/api/v1/example\").json() == {\"message\": \"persistido\"}\n            assert client.post(\"/api/v1/example\", json={\"message\": 123}).status_code == 422\n        finally:\n            client.post(\"/api/v1/example\", json=previous)\n");
    }
    if metrics {
        source_mut(files, prefix, "tests/test_api.py")?.push_str("\n\ndef test_metrics_have_bounded_route_labels():\n    # No startup dependency is needed to inspect liveness and routing metrics.\n    client = TestClient(create_app())\n    client.get(\"/health\")\n    client.get(\"/missing/random-id\")\n    metrics = client.get(\"/metrics\").text\n    assert \"http_requests_total\" in metrics\n    assert \"unmatched\" in metrics\n    assert \"random-id\" not in metrics\n");
    }
    insert(
        files,
        prefix,
        "tests/test_validation.py",
        "import pytest\nfrom pydantic import ValidationError\n\nfrom app.modules.example.schemas import ExampleInput\n\n\n@pytest.mark.parametrize(\"message\", [\"\", \"x\" * 201, 123, None])\ndef test_invalid_messages(message):\n    with pytest.raises(ValidationError):\n        ExampleInput(message=message)\n",
    );
    if vault {
        insert(
            files,
            prefix,
            "tests/test_config.py",
            "import pytest\n\nfrom app.config import read_config\n\n\ndef test_secret_file_and_ambiguity(tmp_path, monkeypatch):\n    secret = tmp_path / \"secret\"\n    secret.write_text(\"value\\n\")\n    monkeypatch.setenv(\"TEST_SECRET_FILE\", str(secret))\n    monkeypatch.delenv(\"TEST_SECRET\", raising=False)\n    assert read_config(\"TEST_SECRET\") == \"value\"\n    monkeypatch.setenv(\"TEST_SECRET\", \"direct\")\n    with pytest.raises(ValueError):\n        read_config(\"TEST_SECRET\")\n",
        );
    }
    Ok(())
}
