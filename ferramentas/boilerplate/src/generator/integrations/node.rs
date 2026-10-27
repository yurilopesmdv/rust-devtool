use super::*;
use serde_json::{Value, json};
use std::collections::BTreeSet;
const PG: &str = include_str!("../../../integrations/node/postgres.ts");
const MONGO: &str = include_str!("../../../integrations/node/mongodb.ts");
const REDIS: &str = include_str!("../../../integrations/node/redis.ts");
const METRICS: &str = include_str!("../../../integrations/node/metrics.ts");

/// Shared dependency composition for generation and lock maintenance.
pub fn manifest(source: &str, db: Database, cache: bool, metrics: bool) -> Result<Value, Error> {
    let mut package: Value =
        serde_json::from_str(source).map_err(|e| Error::operation(e.to_string()))?;
    if !package.is_object()
        || !package["devDependencies"].is_object()
        || !package["scripts"].is_object()
    {
        return Err(Error::operation(
            "Manifesto Node sem objeto/scripts/devDependencies",
        ));
    }
    if db != Database::None {
        let driver = if db == Database::Postgres {
            "pg"
        } else {
            "mongodb"
        };
        package["dependencies"][driver] = json!(if db == Database::Postgres {
            "8.23.1"
        } else {
            "7.7.0"
        });
        if db == Database::Postgres {
            package["devDependencies"]["@types/pg"] = json!("8.23.1");
        }
        package["scripts"]["db:prepare"] =
            json!("node --env-file-if-exists=.env dist/infrastructure/database/prepare.js");
    }
    if cache {
        package["dependencies"]["redis"] = json!("6.3.0");
    }
    if metrics {
        package["dependencies"]["prom-client"] = json!("15.1.3");
    }
    Ok(package)
}

pub fn apply(r: &Recipe, t: &Template, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    let db = r.backend.database.unwrap_or(Database::None);
    let cache = r.backend.cache == Some(Cache::Redis);
    let metrics = r.infrastructure.monitoring == Some(Monitoring::PrometheusGrafana);
    let vault = r.infrastructure.secrets == Some(Secrets::Vault);
    let package = manifest(
        source_mut(files, prefix, "package.json")?,
        db,
        cache,
        metrics,
    )?;
    let mut imports = String::new();
    let mut start = String::new();
    let mut stop = String::new();
    let mut ready = String::new();
    if vault && db == Database::None && !cache {
        imports.push_str("import {readConfig} from '../config/env.js';\n");
        start.push_str("readConfig('APP_SECRET');\n");
    }
    let config = if vault {
        include_str!("../../../integrations/node/config-vault.ts")
    } else {
        include_str!("../../../integrations/node/config-env.ts")
    };
    insert(files, prefix, "src/config/env.ts", config);
    if db != Database::None {
        insert(
            files,
            prefix,
            "src/infrastructure/database/client.ts",
            if db == Database::Postgres { PG } else { MONGO },
        );
        imports.push_str("import * as database from './database/client.js';\n");
        start.push_str("await database.initialize();\n");
        stop.push_str("await database.close();\n");
        ready.push_str("await database.ready();\n");
        insert(
            files,
            prefix,
            "src/modules/example/repositories/example.repository.ts",
            if db == Database::Postgres {
                include_str!("../../../integrations/node/postgres-repository.ts")
            } else {
                include_str!("../../../integrations/node/mongodb-repository.ts")
            },
        );
        insert(
            files,
            prefix,
            "src/infrastructure/database/schema.ts",
            if db == Database::Postgres {
                include_str!("../../../integrations/node/postgres-schema.ts")
            } else {
                include_str!("../../../integrations/node/mongodb-schema.ts")
            },
        );
        insert(
            files,
            prefix,
            "src/infrastructure/database/prepare.ts",
            include_str!("../../../integrations/node/database-prepare.ts"),
        );
        if db == Database::Postgres {
            insert(
                files,
                prefix,
                "migrations/001_examples.sql",
                "CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL);\n-- Aplicada idempotentemente pelo comando db:prepare.\n",
            );
        }
    }
    if cache {
        insert(files, prefix, "src/infrastructure/cache/client.ts", REDIS);
        imports.push_str("import * as cache from './cache/client.js';\n");
        start.push_str("await cache.initialize();\n");
        stop.push_str("await cache.close();\n");
    }
    let mut lifecycle = include_str!("../../../integrations/node/lifecycle.ts").to_string();
    let start = if start.is_empty() {
        "// Nenhuma conexão externa selecionada."
    } else {
        start.trim_end()
    };
    for (slot, value) in [
        ("imports", imports.trim_end()),
        ("start", start),
        ("stop", stop.trim_end()),
        ("ready", ready.trim_end()),
    ] {
        replace_source(
            &mut lifecycle,
            "node/lifecycle.ts",
            &format!("// @boilerplate:{slot}"),
            value,
        )?;
    }
    insert(files, prefix, "src/infrastructure/lifecycle.ts", lifecycle);
    let mut service = String::new();
    if db != Database::None {
        service.push_str("import * as repository from '../repositories/example.repository.js';\n");
    }
    if cache {
        service.push_str("import * as cache from '../../../infrastructure/cache/client.js';\n");
    }
    let mut get = String::new();
    if cache {
        get.push_str("const cached=await cache.get('example');if(cached){try{const parsed:unknown=JSON.parse(cached);if(typeof parsed==='object'&&parsed!==null&&'message' in parsed&&typeof parsed.message==='string')return {message:parsed.message};}catch{/* Descarta cache inválido. */}}\n");
    }
    get.push_str(if db != Database::None {
        "const result=await repository.read();\n"
    } else {
        "const result={message:'API funcionando'};\n"
    });
    if cache {
        get.push_str("await cache.set('example',JSON.stringify(result));\n");
    }
    get.push_str("return result;");
    let write = if cache {
        "const result=await repository.write(message);await cache.remove('example');return result;"
    } else {
        "return repository.write(message);"
    };
    let express = t.integration_adapter.as_deref() == Some("node-express");
    let cache_key = format!("'{}:example'", package["name"].as_str().unwrap());
    get = get.replace("'example'", &cache_key);
    let write = write.replace("'example'", &cache_key);
    if express {
        service.push_str(&format!("export async function getExample(){{{get}}}\n"));
        if db != Database::None {
            service.push_str(&format!(
                "export async function writeExample(message:string){{{write}}}\n"
            ));
        }
        insert(
            files,
            prefix,
            "src/modules/example/services/example.service.ts",
            service,
        );
        insert(
            files,
            prefix,
            "src/modules/example/controllers/example.controller.ts",
            format!(
                "import type {{Request,Response}} from 'express';\nimport {{getExample{}}} from '../services/example.service.js';\nexport async function exampleController(_req:Request,res:Response){{res.json(await getExample());}}\n{}",
                if db != Database::None {
                    ",writeExample"
                } else {
                    ""
                },
                if db != Database::None {
                    "export async function writeController(req:Request,res:Response){const message=req.body?.message;if(typeof message!=='string'||message.length<1||Array.from(message).length>200){res.status(400).json({error:'message precisa conter 1 a 200 caracteres'});return;}res.json(await writeExample(message));}\n"
                } else {
                    ""
                }
            ),
        );
        if db != Database::None {
            replace(
                files,
                prefix,
                "src/http/routes.ts",
                "{ exampleController }",
                "{ exampleController, writeController }",
            )?;
            replace(
                files,
                prefix,
                "src/http/routes.ts",
                "routes.get('/example', exampleController);",
                "routes.get('/example', exampleController);\nroutes.post('/example',writeController);",
            )?;
        }
        source_mut(files, prefix, "src/server.ts")?;
        insert(
            files,
            prefix,
            "src/server.ts",
            include_str!("../../../integrations/node/express-server.ts"),
        );
        replace(
            files,
            prefix,
            "src/app.ts",
            "app.disable('x-powered-by');",
            "app.disable('x-powered-by');\napp.get('/ready',async(_req,res)=>{try{res.json(await readiness());}catch{res.status(503).json({status:'unavailable'});}});",
        )?;
        replace(
            files,
            prefix,
            "src/app.ts",
            "app.get('/ready', (_req, res) => {\n  res.json({ status: 'ok' });\n});\n",
            "",
        )?;
        source_mut(files, prefix, "src/app.ts")?.insert_str(
            0,
            "import {readiness} from './infrastructure/lifecycle.js';\n",
        );
    } else {
        if db != Database::None {
            service = service.replace(
                "import * as repository from '../repositories/example.repository.js';",
                "import { ExampleRepository } from '../repositories/example.repository.js';",
            );
        }
        let get = get.replace("repository.", "this.repository.");
        let write = write.replace("repository.", "this.repository.");
        let constructor = if db != Database::None {
            "constructor(@Inject(ExampleRepository) private readonly repository:ExampleRepository){}"
        } else {
            ""
        };
        service = format!(
            "import {{Injectable, Inject}} from '@nestjs/common';\n{service}\n@Injectable() export class ExampleService{{{constructor}async get(){{{get}}}{}}}\n",
            if db != Database::None {
                format!("async write(message:string){{{write}}}")
            } else {
                String::new()
            }
        );
        if db == Database::None {
            service = service.replace("Injectable, Inject", "Injectable");
        }
        insert(
            files,
            prefix,
            "src/modules/example/example.service.ts",
            service
                .replace("'../repositories/", "'./repositories/")
                .replace("'../../../infrastructure/", "'../../infrastructure/"),
        );
        if db != Database::None {
            let repository = source_mut(
                files,
                prefix,
                "src/modules/example/repositories/example.repository.ts",
            )?;
            let methods = anchor_position(
                repository,
                "src/modules/example/repositories/example.repository.ts",
                "export async function read()",
            )?;
            *repository = format!(
                "import {{Injectable}} from '@nestjs/common';\n{}\n@Injectable() export class ExampleRepository {{ {} }}\n",
                &repository[..methods],
                repository[methods..].replace("export async function ", "async ")
            );
            source_mut(files, prefix, "src/modules/example/example.module.ts")?.insert_str(
                0,
                "import { ExampleRepository } from './repositories/example.repository.js';\n",
            );
            replace(
                files,
                prefix,
                "src/modules/example/example.module.ts",
                "providers: [ExampleService]",
                "providers: [ExampleService, ExampleRepository]",
            )?;
            replace(
                files,
                prefix,
                "src/modules/example/example.controller.ts",
                "Controller, Get, Inject",
                "Controller, Get, Inject, Post, Body, BadRequestException, HttpCode",
            )?;
            replace(
                files,
                prefix,
                "src/modules/example/example.controller.ts",
                "  @Get()\n  get() {\n    return this.service.get();\n  }",
                "  @Get()\n  get() {\n    return this.service.get();\n  }\n @Post() @HttpCode(200) write(@Body() body:{message?:unknown}){const message=body?.message;if(typeof message!=='string'||message.length<1||Array.from(message).length>200)throw new BadRequestException('message precisa conter 1 a 200 caracteres');return this.service.write(message);}",
            )?;
        }
        source_mut(files, prefix, "src/app.ts")?.insert_str(
            0,
            "import {readiness} from './infrastructure/lifecycle.js';\n",
        );
        replace(
            files,
            prefix,
            "src/app.ts",
            "app.enableShutdownHooks();",
            "app.use('/ready',async(_req:unknown,res:{status:(code:number)=>{json:(value:unknown)=>unknown},json:(value:unknown)=>unknown})=>{try{res.json(await readiness());}catch{res.status(503).json({status:'unavailable'});}});\napp.enableShutdownHooks();",
        )?;
    }
    if !express && db != Database::None {
        insert(
            files,
            prefix,
            "tests/service.test.mjs",
            "import 'reflect-metadata';\nimport test from 'node:test';\nimport assert from 'node:assert/strict';\nimport {ExampleService} from '../dist/modules/example/example.service.js';\ntest('service coordena repository injetado sem criar conexão',async()=>{const calls=[];const repository={read:async()=>{calls.push('read');return {message:'origem'};},write:async(message)=>{calls.push('write');return {message};}};const service=new ExampleService(repository);assert.deepEqual(await service.get(),{message:'origem'});assert.deepEqual(await service.write('novo'),{message:'novo'});assert.deepEqual(calls,['read','write']);});\n",
        );
    }
    if !express {
        insert(
            files,
            prefix,
            "src/infrastructure/infrastructure.module.ts",
            include_str!("../../../integrations/node/nest-lifecycle.module.ts"),
        );
        source_mut(files, prefix, "src/app.module.ts")?.insert_str(
            0,
            "import {InfrastructureModule} from './infrastructure/infrastructure.module.js';\n",
        );
        replace(
            files,
            prefix,
            "src/app.module.ts",
            "imports: [ExampleModule]",
            "imports: [InfrastructureModule, ExampleModule]",
        )?;
    }
    if metrics {
        insert(
            files,
            prefix,
            "src/infrastructure/observability/metrics.ts",
            METRICS,
        );
        let path = "src/app.ts";
        source_mut(files, prefix, path)?.insert_str(0,"import {metricsMiddleware,metricsHandler} from './infrastructure/observability/metrics.js';\n");
        if express {
            replace(
                files,
                prefix,
                path,
                "app.disable('x-powered-by');",
                "app.use(metricsMiddleware);\napp.get('/metrics',metricsHandler);\napp.disable('x-powered-by');",
            )?;
        } else {
            replace(
                files,
                prefix,
                path,
                "app.enableShutdownHooks();",
                "app.use(metricsMiddleware);app.getHttpAdapter().get('/metrics',(req,res)=>metricsHandler(req,res));app.enableShutdownHooks();",
            )?;
        }
    }
    let skip = if db != Database::None {
        "!process.env.DATABASE_URL && !process.env.DATABASE_URL_FILE"
    } else if vault && cache {
        "!process.env.REDIS_URL && !process.env.REDIS_URL_FILE"
    } else if vault {
        "!process.env.APP_SECRET && !process.env.APP_SECRET_FILE"
    } else {
        "false"
    };
    let tests = "tests/api.test.mjs";
    if db != Database::None {
        source_mut(files, prefix, tests)?.insert_str(
            0,
            "import * as database from '../dist/infrastructure/database/client.js';\n",
        );
    }
    let source = source_mut(files, prefix, tests)?;
    replace_source(
        source,
        tests,
        "async () =>",
        &format!("{{skip:{skip}}}, async () =>"),
    )?;
    if cache && db == Database::None && !vault {
        source.insert_str(0,"// Exercita fallback com Redis indisponível; não simula um cache real.\nif(!process.env.REDIS_URL)process.env.REDIS_URL='redis://127.0.0.1:9';\n");
    }
    if express {
        source.insert_str(
            0,
            "import { initialize, close } from '../dist/infrastructure/lifecycle.js';\n",
        );
        replace_source(
            source,
            tests,
            "  const server =",
            "  await initialize();\n  const server =",
        )?;
        replace_source(
            source,
            tests,
            "server.close(resolve));",
            "server.close(resolve)); await close();",
        )?;
    }
    let base = if express { "base" } else { "url" };
    let mut checks = String::new();
    if db != Database::None {
        checks.push_str(r#"
assert.equal((await fetch(`${__BASE__}/ready`)).status,200);
const previous=await (await fetch(`${__BASE__}/api/v1/example`)).json();
try {
for(const [message,expected] of [['😀'.repeat(200),200],['😀'.repeat(201),400],[123,400]]){
assert.equal((await fetch(`${__BASE__}/api/v1/example`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({message})})).status,expected);
if(expected===200)assert.equal((await (await fetch(`${__BASE__}/api/v1/example`)).json()).message,message);
}
} finally {
await fetch(`${__BASE__}/api/v1/example`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(previous)});
}
"#);
    }
    if metrics {
        checks.push_str(r#"const metrics=await (await fetch(`${__BASE__}/metrics`)).text();assert.match(metrics,/http_requests_total/);assert.match(metrics,/http_request_duration_seconds/);
"#);
    }
    if db != Database::None {
        checks.push_str("await database.close();\nassert.equal((await fetch(`${__BASE__}/ready`)).status,503);\nassert.equal((await fetch(`${__BASE__}/health`)).status,200);\n");
    }
    let source = source_mut(files, prefix, tests)?;
    let marker = if express {
        "  } finally {"
    } else {
        " } finally {"
    };
    replace_source(
        source,
        tests,
        marker,
        &(checks.replace("__BASE__", base) + marker),
    )?;
    if vault {
        insert(
            files,
            prefix,
            "tests/secrets.test.mjs",
            "import test from 'node:test';\nimport assert from 'node:assert/strict';\nimport {mkdtempSync,writeFileSync,rmSync} from 'node:fs';\nimport {tmpdir} from 'node:os';\nimport {join} from 'node:path';\nimport {readConfig} from '../dist/config/env.js';\ntest('segredo por arquivo e detecção de ambiguidade',()=>{const dir=mkdtempSync(join(tmpdir(),'secret-'));try{writeFileSync(join(dir,'value'),'teste\\n');process.env.TEST_VALUE_FILE=join(dir,'value');assert.equal(readConfig('TEST_VALUE'),'teste');process.env.TEST_VALUE='direto';assert.throws(()=>readConfig('TEST_VALUE'));}finally{delete process.env.TEST_VALUE;delete process.env.TEST_VALUE_FILE;rmSync(dir,{recursive:true});}});\n",
        );
    }
    // Optional dependencies are taken from a tested superset lock, then pruned by dependency reachability.
    let superset = match t.integration_adapter.as_deref() {
        Some("node-express") => include_str!("../../../integrations/node/express-lock.json"),
        _ => include_str!("../../../integrations/node/nest-lock.json"),
    };
    let lock = prune_lock(superset, &package)?;
    insert(
        files,
        prefix,
        "package.json",
        serde_json::to_string_pretty(&package).unwrap() + "\n",
    );
    insert(
        files,
        prefix,
        "package-lock.json",
        serde_json::to_string_pretty(&lock).unwrap() + "\n",
    );
    Ok(())
}

pub fn prune_lock(source: &str, package: &Value) -> Result<Value, Error> {
    let mut lock: Value =
        serde_json::from_str(source).map_err(|e| Error::operation(e.to_string()))?;
    lock["name"] = package["name"].clone();
    lock["packages"][""]["name"] = package["name"].clone();
    lock["packages"][""]["engines"] = package["engines"].clone();
    for key in ["dependencies", "devDependencies"] {
        lock["packages"][""][key] = package[key].clone();
    }
    let all = lock["packages"]
        .as_object()
        .ok_or_else(|| Error::operation("Lockfile sem packages"))?
        .clone();
    let mut seen = BTreeSet::new();
    let mut queue = vec![String::new()];
    while let Some(path) = queue.pop() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let entry = if path.is_empty() {
            &lock["packages"][""]
        } else {
            &all[&path]
        };
        for key in [
            "dependencies",
            "optionalDependencies",
            "peerDependencies",
            "devDependencies",
        ] {
            if key == "devDependencies" && !path.is_empty() {
                continue;
            }
            if let Some(deps) = entry[key].as_object() {
                for dep in deps.keys() {
                    let mut base = path.clone();
                    loop {
                        let candidate = if base.is_empty() {
                            format!("node_modules/{dep}")
                        } else {
                            format!("{base}/node_modules/{dep}")
                        };
                        if all.contains_key(&candidate) {
                            queue.push(candidate);
                            break;
                        }
                        if base.is_empty() {
                            if key == "dependencies" || key == "devDependencies" {
                                return Err(Error::operation(format!(
                                    "Lockfile sem dependência {dep}"
                                )));
                            }
                            break;
                        }
                        base = base
                            .rsplit_once("/node_modules/")
                            .map(|(prefix, _)| prefix.to_string())
                            .unwrap_or_default();
                    }
                }
            }
        }
    }
    lock["packages"]
        .as_object_mut()
        .unwrap()
        .retain(|path, _| seen.contains(path));
    Ok(lock)
}

pub fn next_metrics(files: &mut Files, prefix: &Path) -> Result<(), Error> {
    let package = manifest(
        source_mut(files, prefix, "package.json")?,
        Database::None,
        false,
        true,
    )?;
    let lock = prune_lock(
        include_str!("../../../integrations/node/next-lock.json"),
        &package,
    )?;
    insert(
        files,
        prefix,
        "package.json",
        serde_json::to_string_pretty(&package).unwrap() + "\n",
    );
    insert(
        files,
        prefix,
        "package-lock.json",
        serde_json::to_string_pretty(&lock).unwrap() + "\n",
    );
    insert(
        files,
        prefix,
        "src/app/metrics/route.ts",
        "import {Registry,collectDefaultMetrics} from 'prom-client';\nexport const runtime='nodejs';export const dynamic='force-dynamic';\nconst registry=new Registry();collectDefaultMetrics({register:registry});\nexport async function GET(){return new Response(await registry.metrics(),{headers:{'Content-Type':registry.contentType}}); }\n",
    );
    Ok(())
}

#[cfg(test)]
mod lock_tests {
    use super::*;

    #[test]
    fn pruned_supersets_keep_only_selected_optional_drivers() {
        for (source, lock) in [
            (
                include_str!("../../../templates/express-ts/package.json"),
                include_str!("../../../integrations/node/express-lock.json"),
            ),
            (
                include_str!("../../../templates/nest-ts/package.json"),
                include_str!("../../../integrations/node/nest-lock.json"),
            ),
        ] {
            for db in [Database::None, Database::Postgres, Database::Mongodb] {
                let package = manifest(source, db, false, false).unwrap();
                let pruned = prune_lock(lock, &package).unwrap();
                for (name, selected) in [
                    ("pg", db == Database::Postgres),
                    ("@types/pg", db == Database::Postgres),
                    ("mongodb", db == Database::Mongodb),
                    ("redis", false),
                    ("prom-client", false),
                ] {
                    assert_eq!(
                        pruned["packages"]
                            .get(format!("node_modules/{name}"))
                            .is_some(),
                        selected,
                        "{name} {db:?}"
                    );
                }
            }
        }
    }
}
