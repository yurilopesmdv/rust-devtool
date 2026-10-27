use mongodb::{
    Client, Collection, Database, IndexModel,
    bson::{Bson, DateTime, Document, doc, oid::ObjectId},
    options::ClientOptions,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

type Failure = Box<dyn Error + Send + Sync>;
struct Migration {
    id: String,
    checksum: String,
    operation: Operation,
}
enum Operation {
    Bootstrap,
    Require {
        field: String,
    },
    Restore {
        from: String,
    },
    Copy {
        source: String,
        target: String,
        batch: i64,
    },
}

fn field(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}
fn checksum(bytes: &[u8]) -> Result<String, Failure> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("stdin indisponível")?
        .write_all(bytes)?;
    let result = child.wait_with_output()?;
    let hash = String::from_utf8(result.stdout)?
        .split_whitespace()
        .next()
        .ok_or("Hash ausente")?
        .to_owned();
    if !result.status.success() || hash.len() != 64 || !hash.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err("Checksum inválido".into());
    }
    Ok(hash)
}
fn load() -> Result<Vec<Migration>, Failure> {
    let mut paths = fs::read_dir("migrations/mongo")?.collect::<Result<Vec<_>, _>>()?;
    paths.sort_by_key(|entry| entry.file_name());
    let mut files = Vec::new();
    let mut ids = BTreeSet::new();
    for entry in paths {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "Nome não UTF-8")?;
        if !name.ends_with(".json") {
            continue;
        }
        if !entry.file_type()?.is_file()
            || !name.is_ascii()
            || name.len() < 21
            || !name.bytes().take(14).all(|c| c.is_ascii_digit())
            || name.as_bytes()[14] != b'_'
            || !name[15..name.len() - 5]
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
        {
            return Err("Nome de migration inválido".into());
        }
        let id = name[..14].to_owned();
        if !ids.insert(id.clone()) {
            return Err("Versão duplicada".into());
        }
        let bytes = fs::read(entry.path())?;
        if bytes.len() > 1048576 {
            return Err("Arquivo muito grande".into());
        }
        let spec: Value = serde_json::from_slice(&bytes)?;
        let object = spec.as_object().ok_or("Objeto obrigatório")?;
        let kind = spec["operation"].as_str().ok_or("Operação obrigatória")?;
        let allowed: &[&str] = if kind == "bootstrap_examples" {
            &["operation"]
        } else if kind == "require_field" {
            &["operation", "field"]
        } else if kind == "restore_validator" {
            &["operation", "from_version"]
        } else {
            &["operation", "source", "target", "batch_size"]
        };
        if object.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err("Opção desconhecida".into());
        }
        let operation = match kind {
            "bootstrap_examples" => Operation::Bootstrap,
            "require_field" => {
                let name = spec["field"]
                    .as_str()
                    .ok_or("Campo obrigatório")?
                    .to_owned();
                if !field(&name) {
                    return Err("Campo inválido".into());
                }
                Operation::Require { field: name }
            }
            "restore_validator" => {
                let origin = spec["from_version"]
                    .as_str()
                    .ok_or("Versão de origem obrigatória")?
                    .to_owned();
                if origin.len() != 14 || !origin.bytes().all(|c| c.is_ascii_digit()) || origin >= id
                {
                    return Err("Versão de origem inválida".into());
                }
                Operation::Restore { from: origin }
            }
            "copy_field" => {
                let source = spec["source"]
                    .as_str()
                    .ok_or("Fonte obrigatória")?
                    .to_owned();
                let target = spec["target"]
                    .as_str()
                    .ok_or("Destino obrigatório")?
                    .to_owned();
                let batch = spec["batch_size"]
                    .as_f64()
                    .filter(|value| value.fract() == 0.0 && (1.0..=1000.0).contains(value))
                    .ok_or("Lote inválido")? as i64;
                if !field(&source) || !field(&target) || source == target {
                    return Err("Campo inválido".into());
                }
                Operation::Copy {
                    source,
                    target,
                    batch,
                }
            }
            _ => return Err("Operação desconhecida".into()),
        };
        files.push(Migration {
            id,
            checksum: checksum(&bytes)?,
            operation,
        });
    }
    for file in &files {
        if let Operation::Restore { from } = &file.operation
            && !files.iter().any(|source| {
                source.id == *from && matches!(source.operation, Operation::Require { .. })
            })
        {
            return Err("Origem de validator inexistente".into());
        }
    }
    Ok(files)
}
fn configuration() -> Result<String, Failure> {
    let prefix = if std::env::var("MIGRATION_DATABASE_URL").is_ok_and(|s| !s.is_empty())
        || std::env::var("MIGRATION_DATABASE_URL_FILE").is_ok_and(|s| !s.is_empty())
    {
        "MIGRATION_DATABASE_URL"
    } else {
        "DATABASE_URL"
    };
    let direct = std::env::var(prefix).ok().filter(|s| !s.is_empty());
    let file = std::env::var(format!("{prefix}_FILE"))
        .ok()
        .filter(|s| !s.is_empty());
    let url = match (direct, file) {
        (Some(_), Some(_)) => return Err("Configuração ambígua".into()),
        (Some(url), None) => url,
        (None, Some(file)) => fs::read_to_string(Path::new(&file))?
            .trim_end_matches(['\r', '\n'])
            .to_owned(),
        _ => return Err("URI Mongo obrigatória".into()),
    };
    if !url.starts_with("mongodb://") && !url.starts_with("mongodb+srv://") {
        return Err("URI Mongo obrigatória".into());
    }
    Ok(url)
}
async fn rows(collection: &Collection<Document>) -> Result<Vec<Document>, Failure> {
    let mut cursor = collection.find(doc! {}).await?;
    let mut result = Vec::new();
    while cursor.advance().await? {
        result.push(cursor.deserialize_current()?);
    }
    Ok(result)
}
async fn history(
    ledger: &Collection<Document>,
    files: &[Migration],
) -> Result<Vec<Document>, Failure> {
    let records = rows(ledger).await?;
    for row in &records {
        let id = row.get_str("_id")?;
        let hash = row.get_str("checksum")?;
        if !files
            .iter()
            .any(|file| file.id == id && file.checksum == hash)
        {
            return Err("Histórico divergente".into());
        }
    }
    Ok(records)
}
async fn apply_file(database: &Database, file: &Migration, owner: &str) -> Result<(), Failure> {
    let locks = database.collection::<Document>("_boilerplate_migration_lock");
    let ledger = database.collection::<Document>("_boilerplate_migrations");
    let examples = database.collection::<Document>("examples");
    match &file.operation {
        Operation::Bootstrap => {
            examples
                .create_index(IndexModel::builder().keys(doc! {"message":1}).build())
                .await?;
            examples
                .update_one(
                    doc! {"_id":"default"},
                    doc! {"$setOnInsert":{"message":"API funcionando"}},
                )
                .upsert(true)
                .await?;
        }
        Operation::Require { field } => {
            let missing = doc! {"$expr":{"$ne":[{"$type":format!("${field}")},"string"]}};
            if examples.count_documents(missing).await? > 0 {
                return Err("Backfill pendente".into());
            }
            let info = database
                .run_command(doc! {"listCollections":1,"filter":{"name":"examples"}})
                .await?;
            let options = info
                .get_document("cursor")
                .ok()
                .and_then(|cursor| cursor.get_array("firstBatch").ok())
                .and_then(|batch| batch.first())
                .and_then(Bson::as_document)
                .and_then(|spec| spec.get_document("options").ok())
                .cloned()
                .unwrap_or_default();
            let backup = doc! {"validator":options.get_document("validator").cloned().unwrap_or_default(),"level":options.get_str("validationLevel").unwrap_or("strict"),"action":options.get_str("validationAction").unwrap_or("error")};
            ledger
                .update_one(
                    doc! {"_id":&file.id,"owner":owner,"validator_before":{"$exists":false}},
                    doc! {"$set":{"validator_before":backup}},
                )
                .await?;
            let row = ledger
                .find_one(doc! {"_id":&file.id})
                .await?
                .ok_or("Histórico ausente")?;
            let previous = row
                .get_document("validator_before")?
                .get_document("validator")?
                .clone();
            let mut properties = Document::new();
            properties.insert(field.clone(), doc! {"bsonType":"string"});
            let rule = doc! {"$jsonSchema":{"bsonType":"object","required":[field.clone()],"properties":properties}};
            let validator = if previous.is_empty() {
                rule
            } else {
                doc! {"$and":[previous,rule]}
            };
            database.run_command(doc!{"collMod":"examples","validator":validator,"validationLevel":"strict","validationAction":"error"}).await?;
        }
        Operation::Restore { from } => {
            let source = ledger
                .find_one(doc! {"_id":from,"status":"applied"})
                .await?
                .ok_or("Origem não aplicada")?;
            let backup = source.get_document("validator_before")?;
            database.run_command(doc!{"collMod":"examples","validator":backup.get_document("validator")?.clone(),"validationLevel":backup.get_str("level")?,"validationAction":backup.get_str("action")?}).await?;
        }
        Operation::Copy {
            source,
            target,
            batch,
        } => loop {
            if locks
                .find_one(doc! {"_id":"runner","owner":owner})
                .await?
                .is_none()
            {
                return Err("Reserva perdida".into());
            }
            let mut filter = Document::new();
            filter.insert(target.clone(), doc! {"$exists":false});
            let mut cursor = examples
                .find(filter)
                .sort(doc! {"_id":1})
                .limit(*batch)
                .await?;
            let mut docs = Vec::new();
            while cursor.advance().await? {
                docs.push(cursor.deserialize_current()?);
            }
            if docs.is_empty() {
                break;
            }
            for doc in docs {
                let value = doc
                    .get_str(source)
                    .map_err(|_| "Fonte do backfill precisa ser string")?
                    .to_owned();
                let id = doc.get("_id").ok_or("Documento sem id")?.clone();
                let mut filter = doc! {"_id":id};
                filter.insert(target.clone(), doc! {"$exists":false});
                let mut update = Document::new();
                update.insert(target.clone(), value);
                let changed = examples.update_one(filter, doc! {"$set":update}).await?;
                ledger.update_one(doc!{"_id":&file.id,"owner":owner},doc!{"$inc":{"checkpoint":changed.modified_count as i64},"$set":{"updated_at":DateTime::now()}}).await?;
            }
        },
    }
    if locks
        .find_one(doc! {"_id":"runner","owner":owner})
        .await?
        .is_none()
    {
        return Err("Reserva perdida".into());
    }
    Ok(())
}
async fn apply(database: &Database, files: &[Migration], owner: &str) -> Result<(), Failure> {
    let ledger = database.collection::<Document>("_boilerplate_migrations");
    let current = history(&ledger, files).await?;
    let highest = current
        .iter()
        .filter(|row| row.get_str("status") == Ok("applied"))
        .filter_map(|row| row.get_str("_id").ok())
        .max()
        .unwrap_or("");
    for file in files {
        if ledger
            .find_one(doc! {"_id":&file.id})
            .await?
            .is_some_and(|row| row.get_str("status") == Ok("applied"))
        {
            continue;
        }
        if file.id.as_str() < highest {
            return Err("Migration fora de ordem".into());
        }
        ledger.update_one(doc!{"_id":&file.id},doc!{"$setOnInsert":{"checksum":&file.checksum,"checkpoint":0_i64,"created_at":DateTime::now()},"$set":{"status":"running","owner":owner,"updated_at":DateTime::now()}}).upsert(true).await?;
        match apply_file(database, file, owner).await {
            Ok(()) => {
                ledger.update_one(doc!{"_id":&file.id,"owner":owner},doc!{"$set":{"status":"applied","updated_at":DateTime::now()},"$unset":{"error_code":""}}).await?;
            }
            Err(error) => {
                let _=ledger.update_one(doc!{"_id":&file.id,"owner":owner},doc!{"$set":{"status":"failed","error_code":"execution_failed","updated_at":DateTime::now()}}).await;
                return Err(error);
            }
        }
    }
    println!("{}", json!({"applied":true}));
    Ok(())
}
async fn run(client: &Client, files: &[Migration], args: &[String]) -> Result<(), Failure> {
    let database = client
        .default_database()
        .ok_or("URI precisa incluir banco")?;
    database.run_command(doc! {"ping":1}).await?;
    let locks = database.collection::<Document>("_boilerplate_migration_lock");
    let ledger = database.collection::<Document>("_boilerplate_migrations");
    if args.first().map(String::as_str) == Some("unlock") {
        if args.len() != 3
            || args[2] != "--confirm-stopped"
            || args[1].is_empty()
            || args[1].len() > 128
            || !args[1]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err("Confirmação obrigatória".into());
        }
        if locks
            .delete_one(doc! {"_id":"runner","owner":&args[1]})
            .await?
            .deleted_count
            != 1
        {
            return Err("Reserva não corresponde".into());
        }
        println!("{}", json!({"unlocked":true}));
        return Ok(());
    }
    if args.len() != 1 || !matches!(args[0].as_str(), "status" | "up") || files.is_empty() {
        return Err("Uso: status|up|unlock OWNER --confirm-stopped".into());
    }
    let records = history(&ledger, files).await?;
    if args[0] == "status" {
        let reservation = locks.find_one(doc! {"_id":"runner"}).await?;
        let owner = reservation
            .as_ref()
            .and_then(|row| row.get_str("owner").ok());
        let states:Vec<_>=files.iter().map(|file|{let row=records.iter().find(|row|row.get_str("_id")==Ok(file.id.as_str()));let checkpoint=row.and_then(|row|row.get("checkpoint")).and_then(|value|match value{Bson::Int32(n)=>Some(i64::from(*n)),Bson::Int64(n)=>Some(*n),_=>None}).unwrap_or(0);json!({"version":file.id,"status":row.and_then(|row|row.get_str("status").ok()).unwrap_or("pending"),"checkpoint":checkpoint})}).collect();
        println!("{}", json!({"reserved_by":owner,"migrations":states}));
        return Ok(());
    }
    let owner = ObjectId::new().to_hex();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match locks
            .insert_one(doc! {"_id":"runner","owner":&owner,"started_at":DateTime::now()})
            .await
        {
            Ok(_) => break,
            Err(error) => {
                if locks.find_one(doc! {"_id":"runner"}).await?.is_none() {
                    return Err(error.into());
                }
                if Instant::now() >= deadline {
                    return Err("Reserva indisponível".into());
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
    // Always release our own reservation after cancellation or a bounded execution timeout.
    let result = tokio::select! {
        result=apply(&database,files,&owner)=>result,
        _=tokio::signal::ctrl_c()=>Err("interrompido".into()),
        _=tokio::time::sleep(Duration::from_secs(900))=>Err("timeout".into()),
    };
    locks
        .delete_one(doc! {"_id":"runner","owner":&owner})
        .await?;
    result
}
#[tokio::main]
async fn main() {
    async fn entry() -> Result<(), Failure> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        let files = if args.first().map(String::as_str) == Some("unlock") {
            Vec::new()
        } else {
            load()?
        };
        let mut options = ClientOptions::parse(configuration()?).await?;
        options.server_selection_timeout = Some(Duration::from_secs(5));
        options.connect_timeout = Some(Duration::from_secs(5));
        options.max_pool_size = Some(4);
        let client = Client::with_options(options)?;
        let result = run(&client, &files, &args).await;
        client.shutdown().await;
        result
    }
    if entry().await.is_err() {
        eprintln!(
            "Migration Mongo falhou; verifique configuração, reserva, checksum e dados. Consulte docs/MIGRATIONS.md."
        );
        std::process::exit(1);
    }
}
