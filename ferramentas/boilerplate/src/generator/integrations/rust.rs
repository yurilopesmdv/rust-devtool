use super::*;

pub fn apply(r: &Recipe, files: &mut Files, prefix: &Path) -> Result<(), Error> {
    let db = r.backend.database.unwrap_or(Database::None);
    let cache = r.backend.cache == Some(Cache::Redis);
    let metrics = r.infrastructure.monitoring == Some(Monitoring::PrometheusGrafana);
    let vault = r.infrastructure.secrets == Some(Secrets::Vault);
    let mut cargo = files[&prefix.join("Cargo.toml")]
        .parse::<toml::Table>()
        .map_err(|e| Error::operation(e.to_string()))?;
    let dependencies = cargo
        .get_mut("dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();
    if db == Database::Postgres {
        dependencies.insert("sqlx".into(),toml::from_str::<toml::Table>("dep = { version = '=0.8.6', default-features = false, features = ['runtime-tokio', 'tls-rustls', 'postgres'] }").unwrap().remove("dep").unwrap());
    }
    if db == Database::Mongodb {
        dependencies.insert("mongodb".into(), toml::Value::String("=3.9.1".into()));
    }
    if cache {
        dependencies.insert("redis".into(),toml::from_str::<toml::Table>("dep = { version = '=0.32.7', features = ['tokio-rustls-comp','connection-manager','tls-rustls-webpki-roots'] }").unwrap().remove("dep").unwrap());
    }
    if metrics {
        dependencies.insert("prometheus".into(), toml::Value::String("=0.14.0".into()));
    }
    insert(
        files,
        prefix,
        "Cargo.toml",
        toml::to_string_pretty(&cargo).map_err(|e| Error::operation(e.to_string()))?,
    );
    // The installer resolves a lock after composition; no stale base lock can masquerade as complete.
    if db != Database::None || cache || metrics {
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
        let lock = crate::catalog::RUST_LOCKS
            .iter()
            .find(|(id, _)| *id == key)
            .ok_or_else(|| Error::operation("Lock Rust da combinação não encontrado"))?
            .1;
        let name = cargo["package"]["name"].as_str().unwrap();
        insert(
            files,
            prefix,
            "Cargo.lock",
            lock.replace("{{PROJECT_NAME}}", name),
        );
    }
    let mut modules =
        String::from("pub type Failure=Box<dyn std::error::Error+Send+Sync>;\npub mod config;\n");
    let mut start = String::new();
    let mut stop = String::new();
    let mut ready = String::new();
    if vault && db == Database::None && !cache {
        start.push_str("let _=config::read(\"APP_SECRET\")?;\n");
    }
    let config = if vault {
        r#"pub fn read(key:&str)->Result<String,super::Failure>{let direct=std::env::var(key).ok();let file=std::env::var(format!("{key}_FILE")).ok();if direct.is_some()&&file.is_some(){return Err(format!("{key}: use valor OU arquivo").into());}let value=match file{Some(file)=>std::fs::read_to_string(file)?.trim().to_string(),None=>direct.unwrap_or_default()};if value.is_empty(){return Err(format!("{key} ausente").into());}Ok(value)}"#
    } else {
        r#"pub fn read(key:&str)->Result<String,super::Failure>{let value=std::env::var(key).map_err(|_|format!("{key} ausente"))?;if value.is_empty(){return Err(format!("{key} vazio").into());}Ok(value)}"#
    };
    insert(files, prefix, "src/infrastructure/config.rs", config);
    if db != Database::None {
        modules.push_str("pub mod database;\n");
        start.push_str("database::initialize().await?;\n");
        stop.push_str("database::close().await;\n");
        ready.push_str("database::ready().await?;\n");
        insert(
            files,
            prefix,
            "src/infrastructure/database.rs",
            if db == Database::Postgres {
                POSTGRES
            } else {
                MONGODB
            },
        );
        let name = r.project.name.as_deref().unwrap().replace('-', "_");
        let crate_name = if r.project.kind == Some(ProjectType::Fullstack) {
            format!("{name}_api")
        } else {
            name
        };
        insert(
            files,
            prefix,
            "examples/prepare.rs",
            format!(
                "use {crate_name}::infrastructure::{{database,Failure}};\n#[tokio::main]async fn main()->Result<(),Failure>{{database::initialize().await?;let result=database::prepare().await;database::close().await;result}}\n"
            ),
        );
        insert(
            files,
            prefix,
            "src/modules/example/repositories/mod.rs",
            "pub use crate::infrastructure::database::{read,write};\n",
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
        modules.push_str("pub mod cache;\n");
        start.push_str("cache::initialize().await?;\n");
        insert(files, prefix, "src/infrastructure/cache.rs", CACHE);
    }
    if metrics {
        modules.push_str("pub mod metrics;\n");
        insert(files, prefix, "src/infrastructure/metrics.rs", METRICS);
    }
    modules.push_str(&format!("pub async fn initialize()->Result<(),Failure>{{{start}Ok(())}}\npub async fn close(){{{stop}}}\npub async fn ready()->Result<(),Failure>{{{ready}Ok(())}}\n"));
    insert(files, prefix, "src/infrastructure/mod.rs", modules);
    let mut service = String::new();
    if cache {
        service.push_str("if let Some(value)=crate::infrastructure::cache::get(\"example\").await && let Ok(value)=serde_json::from_str(&value){return Ok(value);}\n");
    }
    service.push_str(if db!=Database::None{"let message=repositories::read().await?;let value=serde_json::json!({\"message\":message});\n"}else{"let value=serde_json::json!({\"message\":\"API funcionando\"});\n"});
    if cache {
        service
            .push_str("crate::infrastructure::cache::set(\"example\",&value.to_string()).await;\n");
    }
    service.push_str("Ok(value)");
    let mut example = format!(
        "{}\nasync fn service()->Result<serde_json::Value,crate::infrastructure::Failure>{{{service}}}\npub async fn controller()->Result<axum::Json<serde_json::Value>,(axum::http::StatusCode,&'static str)>{{service().await.map(axum::Json).map_err(|_|(axum::http::StatusCode::INTERNAL_SERVER_ERROR,\"Dependência indisponível ou banco não preparado\"))}}\n",
        if db != Database::None {
            "pub mod repositories;"
        } else {
            ""
        }
    );
    if db != Database::None {
        example.push_str(&format!("pub async fn write_controller(axum::Json(body):axum::Json<serde_json::Value>)->Result<axum::Json<serde_json::Value>,(axum::http::StatusCode,&'static str)>{{let message=body.get(\"message\").and_then(|v|v.as_str()).filter(|s|!s.is_empty()&&s.chars().count()<=200).ok_or((axum::http::StatusCode::BAD_REQUEST,\"message precisa conter 1 a 200 caracteres\"))?;repositories::write(message).await.map_err(|_|(axum::http::StatusCode::INTERNAL_SERVER_ERROR,\"Falha de persistência\"))?;{}Ok(axum::Json(serde_json::json!({{\"message\":message}})))}}\n",if cache{"crate::infrastructure::cache::remove(\"example\").await;"}else{""}));
    }
    insert(files, prefix, "src/modules/example/mod.rs", example);
    let origin = files[&prefix.join("src/lib.rs")]
        .split("http://localhost:")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("5173")
        .to_string();
    insert(
        files,
        prefix,
        "src/lib.rs",
        format!(
            r#"pub mod modules;
pub mod infrastructure;
use axum::{{Json,Router,routing::get,http::{{StatusCode,Method,HeaderValue,header::CONTENT_TYPE}}}};
use serde_json::json;
use tower_http::cors::CorsLayer;
pub fn app()->Router{{
 let origin=std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_|"http://localhost:{origin}".into());
 let router=Router::new().route("/health",get(||async{{Json(json!({{"status":"ok"}}))}}))
 .route("/ready",get(||async{{infrastructure::ready().await.map(|_|Json(json!({{"status":"ok"}}))).map_err(|_|StatusCode::SERVICE_UNAVAILABLE)}}))
 .route("/api/v1/example",get(modules::example::controller){}){};
 router.layer(CorsLayer::new().allow_origin(origin.parse::<HeaderValue>().expect("FRONTEND_ORIGIN inválida")).allow_methods([Method::GET,Method::POST,Method::OPTIONS]).allow_headers([CONTENT_TYPE]))
}}
"#,
            if db != Database::None {
                ".post(modules::example::write_controller)"
            } else {
                ""
            },
            if metrics {
                ".route(\"/metrics\",get(infrastructure::metrics::render)).layer(axum::middleware::from_fn(infrastructure::metrics::track))"
            } else {
                ""
            }
        ),
    );
    replace(
        files,
        prefix,
        "src/main.rs",
        "Box<dyn std::error::Error>",
        "Box<dyn std::error::Error+Send+Sync>",
    )?;
    let name = r.project.name.as_deref().unwrap().replace('-', "_");
    let crate_name = if r.project.kind == Some(ProjectType::Fullstack) {
        format!("{name}_api")
    } else {
        name
    };
    files
        .get_mut(&prefix.join("src/main.rs"))
        .unwrap()
        .insert_str(0, &format!("use {crate_name}::infrastructure;\n"));
    replace(
        files,
        prefix,
        "src/main.rs",
        "    let port:",
        "    infrastructure::initialize().await?;\n    let port:",
    )?;
    replace(
        files,
        prefix,
        "src/main.rs",
        "    Ok(())",
        "    infrastructure::close().await;\n    Ok(())",
    )?;
    if db != Database::None {
        insert(
            files,
            prefix,
            "tests/api.rs",
            format!(
                "use {crate_name}::{{app,infrastructure}};\nuse axum::{{body::Body,http::{{Request,StatusCode}}}};use tower::ServiceExt;\n#[tokio::test]async fn liveness_does_not_require_a_database(){{let response=app().oneshot(Request::builder().uri(\"/health\").body(Body::empty()).unwrap()).await.unwrap();assert_eq!(response.status(),StatusCode::OK);}}\n#[tokio::test]#[ignore=\"Requer DATABASE_URL e banco preparado; rode cargo test -- --ignored\"]async fn persistence(){{infrastructure::initialize().await.unwrap();infrastructure::database::prepare().await.unwrap();infrastructure::database::write(\"persistido\").await.unwrap();assert_eq!(infrastructure::database::read().await.unwrap(),\"persistido\");infrastructure::close().await;}}\n"
            ),
        );
    }
    if vault {
        insert(
            files,
            prefix,
            "tests/secrets.rs",
            format!(
                "#[test]fn rejects_missing_configuration(){{assert!({crate_name}::infrastructure::config::read(\"BOILERPLATE_MISSING_VALUE_TEST\").is_err());}}\n"
            ),
        );
    }
    if db != Database::None
        && let Some(docker) = files.get_mut(&prefix.join("Dockerfile"))
    {
        *docker = docker.replace(
            "RUN cargo build --release",
            "RUN cargo build --release && cargo build --release --example prepare",
        );
        *docker = docker.replace(
            "USER 10001",
            "COPY --from=build /app/target/release/examples/prepare /app/prepare\nUSER 10001",
        );
    }
    Ok(())
}

const POSTGRES: &str = r#"use super::{Failure,config};
use sqlx::{PgPool,postgres::PgPoolOptions};use tokio::sync::OnceCell;
static POOL:OnceCell<PgPool>=OnceCell::const_new();
fn pool()->Result<&'static PgPool,Failure>{POOL.get().ok_or_else(||"Banco não inicializado".into())}
pub async fn initialize()->Result<(),Failure>{POOL.get_or_try_init(||async{PgPoolOptions::new().max_connections(10).acquire_timeout(std::time::Duration::from_secs(5)).connect(&config::read("DATABASE_URL")?).await.map_err(|e|->Failure{e.into()})}).await?;Ok(())}
pub async fn ready()->Result<(),Failure>{sqlx::query("SELECT 1").execute(pool()?).await?;Ok(())}
pub async fn close(){if let Some(pool)=POOL.get(){pool.close().await;}}
pub async fn prepare()->Result<(),Failure>{sqlx::query("CREATE TABLE IF NOT EXISTS examples(key TEXT PRIMARY KEY,message TEXT NOT NULL)").execute(pool()?).await?;sqlx::query("INSERT INTO examples(key,message) VALUES('default','API funcionando') ON CONFLICT DO NOTHING").execute(pool()?).await?;Ok(())}
pub async fn read()->Result<String,Failure>{sqlx::query_scalar::<_,String>("SELECT message FROM examples WHERE key=$1").bind("default").fetch_optional(pool()?).await?.ok_or_else(||"Execute prepare".into())}
pub async fn write(message:&str)->Result<(),Failure>{sqlx::query("INSERT INTO examples(key,message) VALUES($1,$2) ON CONFLICT(key) DO UPDATE SET message=EXCLUDED.message").bind("default").bind(message).execute(pool()?).await?;Ok(())}
"#;
const MONGODB: &str = r#"use super::{Failure,config};use mongodb::{Client,bson::doc};use tokio::sync::OnceCell;
static CLIENT:OnceCell<Client>=OnceCell::const_new();
fn database()->Result<mongodb::Database,Failure>{CLIENT.get().ok_or("Banco não inicializado")?.default_database().ok_or_else(||"DATABASE_URL precisa incluir o nome do banco".into())}
pub async fn initialize()->Result<(),Failure>{CLIENT.get_or_try_init(||async{Client::with_uri_str(config::read("DATABASE_URL")?).await.map_err(|e|->Failure{e.into()})}).await?;ready().await}
pub async fn ready()->Result<(),Failure>{database()?.run_command(doc!{"ping":1}).await?;Ok(())}
pub async fn close(){if let Some(client)=CLIENT.get(){client.clone().shutdown().await;}}
pub async fn prepare()->Result<(),Failure>{let collection=database()?.collection::<mongodb::bson::Document>("examples");collection.create_index(mongodb::IndexModel::builder().keys(doc!{"message":1}).build()).await?;collection.update_one(doc!{"_id":"default"},doc!{"$setOnInsert":{"message":"API funcionando"}}).upsert(true).await?;Ok(())}
pub async fn read()->Result<String,Failure>{let row=database()?.collection::<mongodb::bson::Document>("examples").find_one(doc!{"_id":"default"}).await?.ok_or("Execute prepare")?;Ok(row.get_str("message")?.to_string())}
pub async fn write(message:&str)->Result<(),Failure>{database()?.collection::<mongodb::bson::Document>("examples").update_one(doc!{"_id":"default"},doc!{"$set":{"message":message}}).upsert(true).await?;Ok(())}
"#;
const CACHE: &str = r#"use super::{Failure,config};use redis::{AsyncCommands,aio::ConnectionManager};use tokio::{sync::OnceCell,time::{timeout,Duration}};
static CACHE:OnceCell<Option<ConnectionManager>>=OnceCell::const_new();
pub async fn initialize()->Result<(),Failure>{let url=config::read("REDIS_URL")?;let client=redis::Client::open(url)?;let connection=timeout(Duration::from_secs(2),client.get_connection_manager()).await.ok().and_then(Result::ok);if connection.is_none(){eprintln!("Cache indisponível; seguindo sem cache");}let _=CACHE.set(connection);Ok(())}
pub async fn get(key:&str)->Option<String>{let mut connection=CACHE.get()?.as_ref()?.clone();timeout(Duration::from_secs(2),connection.get(key)).await.ok()?.ok()}
pub async fn set(key:&str,value:&str){if let Some(Some(connection))=CACHE.get(){let mut connection=connection.clone();let _:Result<redis::RedisResult<()>,_>=timeout(Duration::from_secs(2),connection.set_ex(key,value,30)).await;}}
pub async fn remove(key:&str){if let Some(Some(connection))=CACHE.get(){let mut connection=connection.clone();let _:Result<redis::RedisResult<usize>,_>=timeout(Duration::from_secs(2),connection.del(key)).await;}}
"#;
const METRICS: &str = r#"use axum::{extract::{Request,MatchedPath},middleware::Next,response::{Response,IntoResponse},http::{header::CONTENT_TYPE,StatusCode}};
use prometheus::{Registry,IntCounterVec,HistogramVec,Encoder,TextEncoder};use std::sync::OnceLock;
struct Metrics{registry:Registry,requests:IntCounterVec,duration:HistogramVec}
fn metrics()->&'static Metrics{static METRICS:OnceLock<Metrics>=OnceLock::new();METRICS.get_or_init(||{let registry=Registry::new();let requests=IntCounterVec::new(prometheus::Opts::new("http_requests_total","Requisições HTTP"),&["method","route","status"]).expect("descritor válido");let duration=HistogramVec::new(prometheus::HistogramOpts::new("http_request_duration_seconds","Duração HTTP"),&["method","route"]).expect("descritor válido");registry.register(Box::new(requests.clone())).expect("registro único");registry.register(Box::new(duration.clone())).expect("registro único");Metrics{registry,requests,duration}})}
pub async fn track(request:Request,next:Next)->Response{let route=request.extensions().get::<MatchedPath>().map(|p|p.as_str()).unwrap_or("unmatched").to_string();let raw=request.method().as_str();let method=if ["GET","POST","PUT","PATCH","DELETE","HEAD","OPTIONS"].contains(&raw){raw}else{"OTHER"}.to_string();let start=std::time::Instant::now();let response=next.run(request).await;if route!="/metrics"{let m=metrics();m.requests.with_label_values(&[&method,&route,&response.status().as_u16().to_string()]).inc();m.duration.with_label_values(&[&method,&route]).observe(start.elapsed().as_secs_f64());}response}
pub async fn render()->Response{let mut output=Vec::new();let encoder=TextEncoder::new();if encoder.encode(&metrics().registry.gather(),&mut output).is_err(){return StatusCode::INTERNAL_SERVER_ERROR.into_response();}([(CONTENT_TYPE,"text/plain; version=0.0.4; charset=utf-8")],output).into_response()}
"#;
