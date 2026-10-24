use super::*;
use serde_json::{Value, json};
use std::collections::BTreeSet;
const PG: &str = include_str!("../../../integrations/node/postgres.ts");
const MONGO: &str = include_str!("../../../integrations/node/mongodb.ts");
const REDIS: &str = include_str!("../../../integrations/node/redis.ts");
const METRICS: &str = include_str!("../../../integrations/node/metrics.ts");

pub fn apply(r: &Recipe, t: &Template, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    let db = r.backend.database.unwrap_or(Database::None);
    let cache = r.backend.cache == Some(Cache::Redis);
    let metrics = r.infrastructure.monitoring == Some(Monitoring::PrometheusGrafana);
    let vault = r.infrastructure.secrets == Some(Secrets::Vault);
    let mut package: Value = serde_json::from_str(&files[&prefix.join("package.json")])
        .map_err(|e| Error::operation(e.to_string()))?;
    let mut imports = String::new();
    let mut start = String::new();
    let mut stop = String::new();
    let mut ready = String::new();
    if vault && db == Database::None && !cache {
        imports.push_str("import {readConfig} from './config.js';\n");
        start.push_str("readConfig('APP_SECRET');\n");
    }
    let config = if vault {
        "import { readFileSync } from 'node:fs';\nexport function readConfig(key:string, required=true):string|undefined { const direct=process.env[key]; const path=process.env[`${key}_FILE`]; if(direct!==undefined&&path!==undefined) throw new Error(`${key}: configure valor OU arquivo, não ambos`); const value=path ? readFileSync(path,'utf8').trim() : direct; if(required&&!value) throw new Error(`${key} ausente`); return value; }\n"
    } else {
        "export function readConfig(key:string, required=true):string|undefined { const value=process.env[key]; if(required&&!value) throw new Error(`${key} ausente`); return value; }\n"
    };
    insert(files, prefix, "src/infrastructure/config.ts", config);
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
            "export { read, write } from '../../../infrastructure/database/client.js';\n",
        );
        insert(
            files,
            prefix,
            "src/infrastructure/database/prepare.ts",
            "import { initialize, prepare, close } from './client.js';\nasync function main(){try{await initialize();await prepare();}finally{await close();}}\nvoid main().catch(()=>{console.error('Falha ao preparar banco; confira configuração e conectividade');process.exitCode=1;});\n",
        );
        package["scripts"]["db:prepare"] =
            json!("node --env-file-if-exists=.env dist/infrastructure/database/prepare.js");
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
        package["dependencies"]["redis"] = json!("6.3.0");
        insert(files, prefix, "src/infrastructure/cache/client.ts", REDIS);
        imports.push_str("import * as cache from './cache/client.js';\n");
        start.push_str("await cache.initialize();\n");
        stop.push_str("await cache.close();\n");
    }
    insert(
        files,
        prefix,
        "src/infrastructure/lifecycle.ts",
        format!(
            "{imports}export async function initialize(){{try{{{start}}}catch{{await close();throw new Error('Dependência obrigatória indisponível; confira configuração');}}}}\nexport async function close(){{{stop}}}\nexport async function readiness(){{{ready}return {{status:'ok'}};}}\n"
        ),
    );
    let mut service = String::new();
    if db != Database::None {
        service.push_str("import * as repository from '../repositories/example.repository.js';\n");
    }
    if cache {
        service.push_str("import * as cache from '../../../infrastructure/cache/client.js';\n");
    }
    let mut get = String::new();
    if cache {
        get.push_str("const cached=await cache.get('example');if(cached){try{return JSON.parse(cached) as {message:string};}catch{/* Descarta cache inválido. */}}\n");
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
        "await cache.remove('example');return repository.write(message);"
    } else {
        "return repository.write(message);"
    };
    let express = t.integration_adapter.as_deref() == Some("node-express");
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
                    "export async function writeController(req:Request,res:Response){const message=req.body?.message;if(typeof message!=='string'||message.length<1||message.length>200){res.status(400).json({error:'message precisa conter 1 a 200 caracteres'});return;}res.json(await writeExample(message));}\n"
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
        replace(
            files,
            prefix,
            "src/server.ts",
            "const server = app.listen",
            "async function main(){ await initialize();\nconst server = app.listen",
        )?;
        replace(
            files,
            prefix,
            "src/server.ts",
            "server.close(() => process.exit(0));",
            "server.close(() => { void close().finally(()=>process.exit(0)); });",
        )?;
        let s = files.get_mut(&prefix.join("src/server.ts")).unwrap();
        s.insert_str(
            0,
            "import {initialize,close} from './infrastructure/lifecycle.js';\n",
        );
        s.push_str("}\nvoid main().catch(()=>{console.error('Falha no bootstrap; confira configuração e dependências');process.exitCode=1;});\n");
        replace(
            files,
            prefix,
            "src/app.ts",
            "app.disable('x-powered-by');",
            "app.disable('x-powered-by');\napp.get('/ready',async(_req,res)=>{try{res.json(await readiness());}catch{res.status(503).json({status:'unavailable'});}});",
        )?;
        files
            .get_mut(&prefix.join("src/app.ts"))
            .unwrap()
            .insert_str(
                0,
                "import {readiness} from './infrastructure/lifecycle.js';\n",
            );
    } else {
        service = format!(
            "import {{Injectable}} from '@nestjs/common';\n{service}\n@Injectable() export class ExampleService{{async get(){{{get}}}{}}}\n",
            if db != Database::None {
                format!("async write(message:string){{{write}}}")
            } else {
                String::new()
            }
        );
        insert(
            files,
            prefix,
            "src/modules/example/example.service.ts",
            service
                .replace("'../repositories/", "'./repositories/")
                .replace("'../../../infrastructure/", "'../../infrastructure/"),
        );
        if db != Database::None {
            replace(
                files,
                prefix,
                "src/modules/example/example.controller.ts",
                "Controller, Get, Inject",
                "Controller, Get, Inject, Post, Body, BadRequestException",
            )?;
            replace(
                files,
                prefix,
                "src/modules/example/example.controller.ts",
                " @Get() get() { return this.service.get(); }",
                " @Get() get() { return this.service.get(); }\n @Post() write(@Body() body:{message?:unknown}){const message=body?.message;if(typeof message!=='string'||message.length<1||message.length>200)throw new BadRequestException('message precisa conter 1 a 200 caracteres');return this.service.write(message);}",
            )?;
        }
        replace(
            files,
            prefix,
            "src/app.ts",
            "const app = await NestFactory.create",
            "await initialize(); const app = await NestFactory.create",
        )?;
        files
            .get_mut(&prefix.join("src/app.ts"))
            .unwrap()
            .insert_str(
                0,
                "import {initialize,readiness} from './infrastructure/lifecycle.js';\n",
            );
        replace(
            files,
            prefix,
            "src/app.ts",
            "app.enableShutdownHooks();",
            "app.use('/ready',async(_req:unknown,res:{status:(code:number)=>{json:(value:unknown)=>unknown},json:(value:unknown)=>unknown})=>{try{res.json(await readiness());}catch{res.status(503).json({status:'unavailable'});}});\napp.enableShutdownHooks();",
        )?;
    }
    if !express {
        files.get_mut(&prefix.join("src/app.module.ts")).unwrap().insert_str(0,"import {close} from './infrastructure/lifecycle.js';\nclass IntegrationLifecycle {async onApplicationShutdown(){await close();}}\n");
        replace(
            files,
            prefix,
            "src/app.module.ts",
            "providers: [ExampleService]",
            "providers: [ExampleService,IntegrationLifecycle]",
        )?;
    }
    if metrics {
        package["dependencies"]["prom-client"] = json!("15.1.3");
        insert(
            files,
            prefix,
            "src/infrastructure/observability/metrics.ts",
            METRICS,
        );
        let path = "src/app.ts";
        files.get_mut(&prefix.join(path)).unwrap().insert_str(0,"import {metricsMiddleware,metricsHandler} from './infrastructure/observability/metrics.js';\n");
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
    let skip = if db != Database::None || vault {
        "!process.env.DATABASE_URL && !process.env.DATABASE_URL_FILE"
    } else {
        "false"
    };
    let tests = "tests/api.test.mjs";
    let source = files.get_mut(&prefix.join(tests)).unwrap();
    *source = source.replacen("async () =>", &format!("{{skip:{skip}}}, async () =>"), 1);
    if express {
        source.insert_str(
            0,
            "import { initialize, close } from '../dist/infrastructure/lifecycle.js';\n",
        );
        *source = source.replacen(
            "  const server =",
            "  await initialize();\n  const server =",
            1,
        );
        *source = source.replacen(
            "server.close(resolve));",
            "server.close(resolve)); await close();",
            1,
        );
    }
    if vault {
        insert(
            files,
            prefix,
            "tests/secrets.test.mjs",
            "import test from 'node:test';\nimport assert from 'node:assert/strict';\nimport {mkdtempSync,writeFileSync,rmSync} from 'node:fs';\nimport {tmpdir} from 'node:os';\nimport {join} from 'node:path';\nimport {readConfig} from '../dist/infrastructure/config.js';\ntest('segredo por arquivo e detecção de ambiguidade',()=>{const dir=mkdtempSync(join(tmpdir(),'secret-'));try{writeFileSync(join(dir,'value'),'teste\\n');process.env.TEST_VALUE_FILE=join(dir,'value');assert.equal(readConfig('TEST_VALUE'),'teste');process.env.TEST_VALUE='direto';assert.throws(()=>readConfig('TEST_VALUE'));}finally{delete process.env.TEST_VALUE;delete process.env.TEST_VALUE_FILE;rmSync(dir,{recursive:true});}});\n",
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
    let mut package: Value = serde_json::from_str(&files[&prefix.join("package.json")])
        .map_err(|e| Error::operation(e.to_string()))?;
    package["dependencies"]["prom-client"] = json!("15.1.3");
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
