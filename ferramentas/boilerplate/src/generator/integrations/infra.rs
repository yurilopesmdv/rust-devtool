use super::*;
use crate::{catalog::Catalog, planner::Plan};
use serde_json::{Value, json};

fn encoded(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap() + "\n"
}
fn append(files: &mut Files, path: PathBuf, text: &str) {
    files.entry(path).or_default().push_str(text);
}
fn local(r: &Recipe, location: Option<Location>) -> bool {
    location == Some(Location::Local)
        && (r.infrastructure.containers == Some(Containers::Compose)
            || r.infrastructure.kubernetes == Some(Kubernetes::Manifests))
}
const PG: &str = "postgres:17.6-alpine";
const MONGO: &str = "mongo:8.0.13";
const REDIS: &str = "redis:7.4.5-alpine";
const PROM: &str = "prom/prometheus:v3.5.0";
const GRAFANA: &str = "grafana/grafana:12.1.1";
const VAULT: &str = "hashicorp/vault:1.20.2";
const EXPORTER: &str = "nginx/nginx-prometheus-exporter:1.4.2";

pub fn apply(plan: &Plan, catalog: &Catalog, root: &Path, files: &mut Files) -> Result<(), Error> {
    let r = &plan.configuration;
    if plan.integrations.is_empty() {
        return Ok(());
    }
    let mono = r.project.layout == Some(Layout::Monorepo);
    let apps: Vec<_> = plan
        .applications
        .iter()
        .filter(|a| mono || a.destination == root)
        .collect();
    let has_api = apps.iter().any(|a| {
        catalog
            .get(&a.template)
            .is_ok_and(|t| t.role == ProjectType::Backend)
    });
    let global_api = r.backend.template.is_some();
    let monitored = r.infrastructure.monitoring == Some(Monitoring::PrometheusGrafana)
        && (has_api || !global_api);
    let vault = r.infrastructure.secrets == Some(Secrets::Vault) && has_api;
    let db = if has_api {
        r.backend.database.unwrap_or(Database::None)
    } else {
        Database::None
    };
    let cache = has_api && r.backend.cache == Some(Cache::Redis);
    let db_local = db != Database::None && local(r, r.backend.database_location);
    let cache_local = cache && local(r, r.backend.cache_location);
    let compose = r.infrastructure.containers == Some(Containers::Compose);
    let k8s = r.infrastructure.kubernetes == Some(Kubernetes::Manifests);
    if !compose && !k8s && !monitored && !vault && db == Database::None && !cache {
        return Ok(());
    }
    let name = r.project.name.as_deref().unwrap();
    let namespace = r.infrastructure.namespace.as_deref().unwrap_or(name);
    let web_port = r
        .frontend
        .template
        .as_deref()
        .map(|id| catalog.get(id))
        .transpose()?
        .map(|t| t.port)
        .unwrap_or(5173);
    let mut env = String::new();
    insert(files, Path::new(""), "infra/.env.example", String::new());
    if db_local {
        env.push_str(if db == Database::Postgres {
            "POSTGRES_PASSWORD=replace-me\n"
        } else {
            "MONGO_PASSWORD=replace-me\n"
        });
    }
    if db != Database::None {
        let uri = if db == Database::Postgres {
            "postgresql://app:replace-me@database:5432/app"
        } else {
            "mongodb://app:replace-me@database:27017/app?authSource=admin"
        };
        let uri = if db_local {
            uri.to_string()
        } else {
            uri.replace("@database:", "@database.example.invalid:")
        };
        env.push_str(&format!("DATABASE_URL={uri}\n"));
    }
    if cache {
        if cache_local {
            env.push_str("REDIS_PASSWORD=replace-me\n");
        }
        env.push_str(if cache_local {
            "REDIS_URL=redis://:replace-me@cache:6379\n"
        } else {
            "REDIS_URL=rediss://:replace-me@cache.example.invalid:6379\n"
        });
    }
    if monitored {
        env.push_str("GRAFANA_ADMIN_PASSWORD=replace-me\n");
    }
    if vault {
        let address = r.infrastructure.vault_address.as_deref().unwrap_or(
            if r.infrastructure.vault_location == Some(Location::Local) {
                "http://vault:8200"
            } else {
                "https://vault.example.invalid:8200"
            },
        );
        env.push_str(&format!("VAULT_ADDR={address}\n"));
    }
    // Root env is for container/cluster endpoints. Application env uses host-accessible endpoints.
    if !env.is_empty() {
        if mono {
            append(files, ".env.example".into(), &env);
        } else {
            append(
                files,
                ".env.example".into(),
                "\n# Infraestrutura (DATABASE_URL/REDIS_URL acima usam localhost fora de containers):\n",
            );
            let without_urls = env
                .lines()
                .filter(|l| !l.starts_with("DATABASE_URL=") && !l.starts_with("REDIS_URL="))
                .collect::<Vec<_>>()
                .join("\n");
            append(files, ".env.example".into(), &(without_urls + "\n"));
        }
        insert(files, Path::new(""), "infra/.env.example", env.clone());
    }
    if has_api {
        let app = apps
            .iter()
            .find(|a| catalog.get(&a.template).unwrap().role == ProjectType::Backend)
            .unwrap();
        let prefix = app.destination.strip_prefix(root).unwrap();
        let mut extra = String::new();
        if db != Database::None {
            extra.push_str(if db == Database::Postgres {
                "DATABASE_URL=postgresql://app:replace-me@localhost:5432/app\n"
            } else {
                "DATABASE_URL=mongodb://app:replace-me@localhost:27017/app?authSource=admin\n"
            });
        }
        if r.backend.database_location == Some(Location::External) {
            extra = extra.replace("@localhost:", "@database.example.invalid:");
        }
        if cache {
            extra.push_str(if r.backend.cache_location == Some(Location::External) {
                "REDIS_URL=rediss://:replace-me@cache.example.invalid:6379\n"
            } else {
                "REDIS_URL=redis://:replace-me@localhost:6379\n"
            });
        }
        if vault {
            extra.push_str(
                "# Para Vault, remova a variável direta ao habilitar a variante _FILE:\n",
            );
            if db != Database::None {
                extra.push_str("# DATABASE_URL_FILE=/run/secrets/database-url\n");
            }
            if cache {
                extra.push_str("# REDIS_URL_FILE=/run/secrets/redis-url\n");
            }
        }
        append(files, prefix.join(".env.example"), &extra);
    }
    append(
        files,
        ".gitignore".into(),
        if vault {
            "\ninfra/.env\ninfra/vault/local/*\n"
        } else {
            "\ninfra/.env\n"
        },
    );
    if vault {
        for app in &apps {
            let ignore = app
                .destination
                .strip_prefix(root)
                .unwrap()
                .join(".dockerignore");
            if let Some(content) = files.get_mut(&ignore) {
                content.push_str(
                    "\ninfra/vault/local/\ninfra/.env\n**/.env\n**/.env.*\n!**/.env.example\n",
                );
            }
        }
    }
    if monitored {
        monitor_files(
            files,
            has_api,
            apps.iter()
                .any(|a| catalog.get(&a.template).unwrap().package_manager == "npm"),
        )?;
    }
    if vault {
        vault_files(files, r, name, db, cache)?;
    }
    if compose {
        let mut services = serde_json::Map::new();
        let mut volumes = serde_json::Map::new();
        for app in &apps {
            let t = catalog.get(&app.template)?;
            let web = t.role == ProjectType::Frontend;
            let role = if web { "web" } else { "api" };
            let prefix = app.destination.strip_prefix(root).unwrap();
            let context = if prefix.as_os_str().is_empty() {
                ".".into()
            } else {
                format!("./{}", prefix.display())
            };
            let port = if web { web_port } else { t.port };
            let mut service = json!({"build":{"context":context},"ports":[format!("127.0.0.1:{port}:{}",t.container_port.unwrap_or(port))],"init":true});
            if web {
                service["build"]["args"] = json!({"VITE_API_URL":r.infrastructure.api_url.as_deref().unwrap_or("http://localhost:3000"),"NEXT_PUBLIC_API_URL":r.infrastructure.api_url.as_deref().unwrap_or("http://localhost:3000")});
            }
            if !web {
                let mut environment = json!({"PORT":t.port.to_string(),"FRONTEND_ORIGIN":r.infrastructure.frontend_origin.clone().unwrap_or_else(||format!("http://localhost:{web_port}"))});
                if vault && db == Database::None && !cache {
                    environment["APP_SECRET_FILE"] = json!("/run/secrets/app-secret");
                }
                if db != Database::None {
                    environment[if vault {
                        "DATABASE_URL_FILE"
                    } else {
                        "DATABASE_URL"
                    }] = json!(if vault {
                        "/run/secrets/database-url"
                    } else {
                        "${DATABASE_URL:?configure DATABASE_URL no ambiente ou .env}"
                    });
                }
                if cache {
                    environment[if vault { "REDIS_URL_FILE" } else { "REDIS_URL" }] =
                        json!(if vault {
                            "/run/secrets/redis-url"
                        } else {
                            "${REDIS_URL:?configure REDIS_URL no ambiente ou .env}"
                        });
                }
                service["environment"] = environment;
                let mut depends = serde_json::Map::new();
                if db_local {
                    depends.insert("database".into(), json!({"condition":"service_healthy"}));
                }
                if vault {
                    depends.insert(
                        "vault-agent".into(),
                        json!({"condition":"service_completed_successfully"}),
                    );
                    service["volumes"] = json!(["app-secrets:/run/secrets:ro"]);
                }
                if !depends.is_empty() {
                    service["depends_on"] = Value::Object(depends);
                }
            } else if !has_api
                && monitored
                && (t.package_manager == "none" || t.id == "react-vite-ts")
            {
                static_config(files, prefix);
                service["volumes"] = json!([format!(
                    "./{}:/etc/nginx/conf.d/default.conf:ro",
                    prefix.join("infra/nginx/default.conf").display()
                )]);
            }
            services.insert(role.into(), service);
        }
        if db_local {
            let service = if db == Database::Postgres {
                json!({"image":PG,"environment":{"POSTGRES_USER":"app","POSTGRES_DB":"app","POSTGRES_PASSWORD":"${POSTGRES_PASSWORD:?configure POSTGRES_PASSWORD}"},"ports":["127.0.0.1:5432:5432"],"volumes":["database-data:/var/lib/postgresql/data"],"healthcheck":{"test":["CMD-SHELL","pg_isready -U app -d app"],"interval":"5s","timeout":"3s","retries":20}})
            } else {
                json!({"image":MONGO,"environment":{"MONGO_INITDB_ROOT_USERNAME":"app","MONGO_INITDB_ROOT_PASSWORD":"${MONGO_PASSWORD:?configure MONGO_PASSWORD}"},"ports":["127.0.0.1:27017:27017"],"volumes":["database-data:/data/db"],"healthcheck":{"test":["CMD","mongosh","--quiet","--eval","quit(db.adminCommand({ping:1}).ok ? 0 : 1)"],"interval":"5s","timeout":"3s","retries":20}})
            };
            services.insert("database".into(), service);
            volumes.insert("database-data".into(), json!({}));
        }
        if cache_local {
            services.insert("cache".into(),json!({"image":REDIS,"command":["redis-server","--appendonly","yes","--requirepass","${REDIS_PASSWORD:?configure REDIS_PASSWORD}"],"environment":{"REDIS_PASSWORD":"${REDIS_PASSWORD:?configure REDIS_PASSWORD}"},"ports":["127.0.0.1:6379:6379"],"volumes":["cache-data:/data"],"healthcheck":{"test":["CMD-SHELL","REDISCLI_AUTH=\"$$REDIS_PASSWORD\" redis-cli ping"],"interval":"5s","timeout":"3s","retries":10}}));
            volumes.insert("cache-data".into(), json!({}));
        }
        if monitored {
            let target = if has_api {
                "api:3000"
            } else if r.frontend.template.as_deref() == Some("next-ts") {
                "web:3001"
            } else {
                "web-exporter:9113"
            };
            insert(
                files,
                Path::new(""),
                "infra/monitoring/prometheus.compose.yml",
                encoded(&prometheus_config(target)),
            );
            services.insert("prometheus".into(),json!({"image":PROM,"command":["--config.file=/etc/prometheus/prometheus.yml","--storage.tsdb.retention.time=24h"],"ports":["127.0.0.1:9090:9090"],"volumes":["./infra/monitoring/prometheus.compose.yml:/etc/prometheus/prometheus.yml:ro","prometheus-data:/prometheus"]}));
            services.insert("grafana".into(),json!({"image":GRAFANA,"environment":{"GF_SECURITY_ADMIN_PASSWORD":"${GRAFANA_ADMIN_PASSWORD:?configure GRAFANA_ADMIN_PASSWORD}","GF_USERS_ALLOW_SIGN_UP":"false"},"ports":["127.0.0.1:3002:3000"],"volumes":["./infra/monitoring/grafana/provisioning:/etc/grafana/provisioning:ro","./infra/monitoring/grafana/dashboards:/var/lib/grafana/dashboards:ro","grafana-data:/var/lib/grafana"]}));
            if !has_api && r.frontend.template.as_deref() != Some("next-ts") {
                services.insert("web-exporter".into(),json!({"image":EXPORTER,"command":["--nginx.scrape-uri=http://web:8081/nginx_status"]}));
            }
            volumes.insert("prometheus-data".into(), json!({}));
            volumes.insert("grafana-data".into(), json!({}));
        }
        if vault {
            if r.infrastructure.vault_location == Some(Location::Local) {
                services.insert("vault".into(),json!({"image":VAULT,"command":["server"],"environment":{"SKIP_SETCAP":"true"},"ports":["127.0.0.1:8200:8200"],"volumes":["./infra/vault/server.hcl:/vault/config/server.hcl:ro","vault-data:/vault/file"]}));
                volumes.insert("vault-data".into(), json!({}));
            }
            services.insert("vault-agent".into(),json!({"image":VAULT,"user":"0:0","entrypoint":["vault"],"command":["agent","-config=/vault/config/agent.hcl"],"environment":{"VAULT_ADDR":"${VAULT_ADDR:?configure VAULT_ADDR}"},"volumes":["./infra/vault/agent.compose.hcl:/vault/config/agent.hcl:ro","./infra/vault/local:/vault/auth:ro","app-secrets:/run/secrets"]}));
            volumes.insert("app-secrets".into(), json!({}));
        }
        let mut output = json!({"services":services});
        if !volumes.is_empty() {
            output["volumes"] = Value::Object(volumes);
        }
        insert(files, Path::new(""), "compose.yaml", encoded(&output));
    }
    if k8s {
        kubernetes(
            plan,
            catalog,
            &apps,
            files,
            has_api,
            monitored,
            vault,
            db,
            cache,
            db_local,
            cache_local,
            namespace,
            name,
        )?;
    }
    for (path, content) in files
        .iter_mut()
        .filter(|(path, _)| path.file_name().is_some_and(|name| name == "README.md"))
    {
        let _ = path;
        *content = content
            .replace(
                "docker compose config",
                "docker compose --env-file infra/.env config",
            )
            .replace(
                "docker compose up",
                "docker compose --env-file infra/.env up",
            )
            .replace(
                "docker compose down",
                "docker compose --env-file infra/.env down",
            );
    }
    docs::write(r, files, has_api, monitored, vault, namespace, name);
    Ok(())
}

fn prometheus_config(target: &str) -> Value {
    json!({"global":{"scrape_interval":"15s"},"scrape_configs":[{"job_name":"application","static_configs":[{"targets":[target]}]}]})
}
fn monitor_files(files: &mut Files, api: bool, node: bool) -> Result<(), Error> {
    insert(
        files,
        Path::new(""),
        "infra/monitoring/prometheus.local.yml",
        encoded(&prometheus_config(if api {
            "127.0.0.1:3000"
        } else if node {
            "127.0.0.1:3001"
        } else {
            "127.0.0.1:9113"
        })),
    );
    insert(
        files,
        Path::new(""),
        "infra/monitoring/grafana/provisioning/datasources/prometheus.yaml",
        encoded(
            &json!({"apiVersion":1,"datasources":[{"name":"Prometheus","type":"prometheus","uid":"boilerplate-prometheus","access":"proxy","url":"http://prometheus:9090","isDefault":true}]}),
        ),
    );
    insert(
        files,
        Path::new(""),
        "infra/monitoring/grafana/provisioning/dashboards/default.yaml",
        encoded(
            &json!({"apiVersion":1,"providers":[{"name":"Application","folder":"Application","type":"file","options":{"path":"/var/lib/grafana/dashboards"}}]}),
        ),
    );
    let mut panels = vec![panel(1, "Disponibilidade", "up{job=\"application\"}", 0)];
    if api {
        panels.push(panel(
            2,
            "Requisições/s",
            "sum(rate(http_requests_total[5m]))",
            8,
        ));
        panels.push(panel(
            3,
            "Latência p95",
            "histogram_quantile(0.95,sum(rate(http_request_duration_seconds_bucket[5m])) by (le))",
            16,
        ));
        panels.push(panel(
            4,
            "Erros 5xx/s",
            "sum(rate(http_requests_total{status=~\"5..\"}[5m]))",
            24,
        ));
    } else if node {
        panels.push(panel(2, "Memória Node", "process_resident_memory_bytes", 8));
    } else {
        panels.push(panel(
            2,
            "Requisições nginx/s",
            "rate(nginx_http_requests_total[5m])",
            8,
        ));
        panels.push(panel(3, "Conexões nginx", "nginx_connections_active", 16));
    }
    insert(
        files,
        Path::new(""),
        "infra/monitoring/grafana/dashboards/application.json",
        encoded(
            &json!({"title":"Application","uid":"boilerplate-application","schemaVersion":39,"version":1,"refresh":"15s","panels":panels}),
        ),
    );
    Ok(())
}
fn panel(id: i32, title: &str, expr: &str, y: i32) -> Value {
    json!({"id":id,"title":title,"type":"timeseries","gridPos":{"x":0,"y":y,"w":24,"h":8},"datasource":{"type":"prometheus","uid":"boilerplate-prometheus"},"targets":[{"expr":expr,"refId":"A"}]})
}
fn static_config(files: &mut Files, prefix: &Path) {
    insert(
        files,
        prefix,
        "infra/nginx/default.conf",
        "server { listen 8080; server_name _; root /usr/share/nginx/html; location / { try_files $uri $uri/ /index.html; } }\nserver { listen 8081; location = /nginx_status { stub_status; access_log off; } }\n",
    );
}

fn agent_templates(name: &str, db: Database, cache: bool, base: &str) -> String {
    let mut result = String::new();
    for (enabled, key, file) in [
        (db != Database::None, "database_url", "database-url"),
        (cache, "redis_url", "redis-url"),
    ] {
        if enabled {
            result.push_str(&format!("template {{\n contents = \"{{{{ with secret \\\"kv/data/{name}\\\" }}}}{{{{ .Data.data.{key} }}}}{{{{ end }}}}\"\n destination = \"{base}/{file}\"\n perms = 0644\n error_on_missing_key = true\n}}\n"));
        }
    }
    // A Vault-only project still demonstrates a file delivery contract without adding unrelated services.
    if db == Database::None && !cache {
        result.push_str(&format!("template {{\n contents = \"{{{{ with secret \\\"kv/data/{name}\\\" }}}}{{{{ .Data.data.app_secret }}}}{{{{ end }}}}\"\n destination = \"{base}/app-secret\"\n perms = 0644\n error_on_missing_key = true\n}}\n"));
    }
    result
}
fn vault_files(
    files: &mut Files,
    r: &Recipe,
    name: &str,
    db: Database,
    cache: bool,
) -> Result<(), Error> {
    if r.infrastructure.vault_location == Some(Location::Local) {
        insert(
            files,
            Path::new(""),
            "infra/vault/server.hcl",
            "ui = true\ndisable_mlock = true\nstorage \"file\" { path = \"/vault/file\" }\nlistener \"tcp\" {\n address = \"0.0.0.0:8200\"\n tls_disable = true\n}\n# Laboratório local: use TLS e armazenamento/HA adequados fora dele.\n",
        );
    }
    let policy = format!("path \"kv/data/{name}\" {{ capabilities = [\"read\"] }}\n");
    insert(files, Path::new(""), "infra/vault/app-policy.hcl", policy);
    let mut compose = String::from(
        "exit_after_auth = true\nauto_auth {\n method \"approle\" {\n  config = { role_id_file_path = \"/vault/auth/role-id\", secret_id_file_path = \"/vault/auth/secret-id\", remove_secret_id_file_after_reading = false }\n }\n}\n",
    );
    compose.push_str(&agent_templates(name, db, cache, "/run/secrets"));
    insert(
        files,
        Path::new(""),
        "infra/vault/agent.compose.hcl",
        compose,
    );
    let mut k8s = format!(
        "exit_after_auth = true\nvault {{ address = \"{}\" }}\nauto_auth {{\n method \"kubernetes\" {{ config = {{ role = \"{name}-api\" }} }}\n}}\n",
        r.infrastructure.vault_address.as_deref().unwrap_or(
            if r.infrastructure.vault_location == Some(Location::Local) {
                "http://vault:8200"
            } else {
                "https://vault.example.invalid:8200"
            }
        )
    );
    k8s.push_str(&agent_templates(name, db, cache, "/vault/secrets"));
    insert(
        files,
        Path::new(""),
        "infra/vault/agent.kubernetes.hcl",
        k8s,
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn kubernetes(
    plan: &Plan,
    catalog: &Catalog,
    apps: &[&crate::planner::Application],
    files: &mut Files,
    has_api: bool,
    monitored: bool,
    vault: bool,
    db: Database,
    cache: bool,
    db_local: bool,
    cache_local: bool,
    namespace: &str,
    name: &str,
) -> Result<(), Error> {
    let r = &plan.configuration;
    let mono = r.project.layout == Some(Layout::Monorepo);
    let mut resources = vec!["namespace.yaml".to_string()];
    insert(
        files,
        Path::new(""),
        "infra/kubernetes/base/namespace.yaml",
        encoded(&json!({"apiVersion":"v1","kind":"Namespace","metadata":{"name":namespace}})),
    );
    for app in apps {
        let t = catalog.get(&app.template)?;
        let web = t.role == ProjectType::Frontend;
        let role = if web { "web" } else { "api" };
        let port = t.container_port.unwrap_or(t.port);
        let image = if r.project.kind == Some(ProjectType::Fullstack) {
            if web {
                r.infrastructure.image_web.as_ref()
            } else {
                r.infrastructure.image_api.as_ref()
            }
        } else {
            r.infrastructure.image.as_ref()
        }
        .cloned()
        .unwrap_or_else(|| format!("{name}-{role}:0.1.0"));
        let mut container = json!({"name":role,"image":image,"imagePullPolicy":"IfNotPresent","ports":[{"containerPort":port}],"resources":{"requests":{"cpu":"100m","memory":"128Mi"},"limits":{"cpu":"1","memory":"512Mi"}},"securityContext":{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]}},"livenessProbe":{"httpGet":{"path":if web{"/"}else{"/health"},"port":port},"initialDelaySeconds":10},"readinessProbe":{"httpGet":{"path":if web{"/"}else if has_api&&(!plan.integrations.is_empty()){"/ready"}else{"/health"},"port":port},"initialDelaySeconds":5}});
        let mut env = vec![];
        if !web {
            env.push(json!({"name":"PORT","value":t.port.to_string()}));
            env.push(json!({"name":"FRONTEND_ORIGIN","value":r.infrastructure.frontend_origin.clone().unwrap_or_else(||format!("http://localhost:{}",r.frontend.template.as_deref().map(|id|catalog.get(id).unwrap().port).unwrap_or(5173)))}));
            if vault && db == Database::None && !cache {
                env.push(json!({"name":"APP_SECRET_FILE","value":"/vault/secrets/app-secret"}));
            }
            if db != Database::None {
                env.push(if vault {
                    json!({"name":"DATABASE_URL_FILE","value":"/vault/secrets/database-url"})
                } else {
                    secret_env("DATABASE_URL")
                });
            }
            if cache {
                env.push(if vault {
                    json!({"name":"REDIS_URL_FILE","value":"/vault/secrets/redis-url"})
                } else {
                    secret_env("REDIS_URL")
                });
            }
        }
        container["env"] = json!(env);
        let mut spec = json!({"containers":[container],"securityContext":{"runAsNonRoot":true,"runAsUser":1000,"fsGroup":1000}});
        if !web && vault {
            vault_pod(&mut spec, files)?;
        }
        if web && !has_api && monitored && t.id != "next-ts" {
            let prefix = if mono {
                PathBuf::from("apps/web")
            } else {
                PathBuf::new()
            };
            static_config(files, &prefix);
            let content = files[&prefix.join("infra/nginx/default.conf")].clone();
            let cm = json!({"apiVersion":"v1","kind":"ConfigMap","metadata":{"name":"web-nginx"},"data":{"default.conf":content}});
            resource(files, &mut resources, "web-nginx.yaml", cm);
            spec["volumes"] = json!([{"name":"nginx","configMap":{"name":"web-nginx"}}]);
            spec["containers"][0]["volumeMounts"] = json!([{"name":"nginx","mountPath":"/etc/nginx/conf.d/default.conf","subPath":"default.conf","readOnly":true}]);
            spec["containers"].as_array_mut().unwrap().push(json!({"name":"exporter","image":EXPORTER,"args":["--nginx.scrape-uri=http://127.0.0.1:8081/nginx_status"],"ports":[{"containerPort":9113}]}));
            resource(
                files,
                &mut resources,
                "web-metrics.yaml",
                service("web-metrics", "web", 9113, 9113),
            );
        }
        resource(
            files,
            &mut resources,
            &format!("{role}-deployment.yaml"),
            json!({"apiVersion":"apps/v1","kind":"Deployment","metadata":{"name":role},"spec":{"replicas":1,"selector":{"matchLabels":{"app":role}},"template":{"metadata":{"labels":{"app":role}},"spec":spec.clone()}}}),
        );
        resource(
            files,
            &mut resources,
            &format!("{role}-service.yaml"),
            service(role, role, if web { 80 } else { t.port }, port),
        );
        if !web && db != Database::None {
            let mut job_spec = spec.clone();
            job_spec["restartPolicy"] = json!("Never");
            let c = &mut job_spec["containers"][0];
            c.as_object_mut().unwrap().remove("livenessProbe");
            c.as_object_mut().unwrap().remove("readinessProbe");
            c["command"] = if t.package_manager == "npm" {
                json!(["node", "dist/infrastructure/database/prepare.js"])
            } else {
                json!(["/app/prepare"])
            };
            insert(
                files,
                Path::new(""),
                "infra/kubernetes/jobs/prepare.yaml",
                encoded(
                    &json!({"apiVersion":"batch/v1","kind":"Job","metadata":{"name":format!("{name}-prepare"),"namespace":namespace},"spec":{"backoffLimit":2,"template":{"metadata":{"labels":{"app":"prepare"}},"spec":job_spec}}}),
                ),
            );
        }
    }
    if db_local {
        let (image, port, path, env) = if db == Database::Postgres {
            (
                PG,
                5432,
                "/var/lib/postgresql/data",
                json!([{"name":"POSTGRES_USER","value":"app"},{"name":"POSTGRES_DB","value":"app"},secret_env("POSTGRES_PASSWORD")]),
            )
        } else {
            (
                MONGO,
                27017,
                "/data/db",
                json!([{"name":"MONGO_INITDB_ROOT_USERNAME","value":"app"},{"name":"MONGO_INITDB_ROOT_PASSWORD","valueFrom":{"secretKeyRef":{"name":"app-secrets","key":"MONGO_PASSWORD"}}}]),
            )
        };
        stateful(
            files,
            &mut resources,
            "database",
            image,
            port,
            path,
            env,
            None,
            false,
        );
    }
    if cache_local {
        stateful(
            files,
            &mut resources,
            "cache",
            REDIS,
            6379,
            "/data",
            json!([secret_env("REDIS_PASSWORD")]),
            Some(json!([
                "redis-server",
                "--appendonly",
                "yes",
                "--requirepass",
                "$(REDIS_PASSWORD)"
            ])),
            false,
        );
    }
    if monitored {
        let target = if has_api {
            "api:3000"
        } else if r.frontend.template.as_deref() == Some("next-ts") {
            "web:80"
        } else {
            "web-metrics:9113"
        };
        let cfg = encoded(&prometheus_config(target));
        resource(
            files,
            &mut resources,
            "prometheus-config.yaml",
            configmap("prometheus", json!({"prometheus.yml":cfg})),
        );
        let mut prom = workload("prometheus", PROM, 9090);
        prom["spec"]["template"]["spec"]["volumes"] = json!([{"name":"config","configMap":{"name":"prometheus"}},{"name":"data","emptyDir":{}}]);
        let c = &mut prom["spec"]["template"]["spec"]["containers"][0];
        c["args"] = json!([
            "--config.file=/etc/prometheus/prometheus.yml",
            "--storage.tsdb.retention.time=24h"
        ]);
        c["volumeMounts"] = json!([{"name":"config","mountPath":"/etc/prometheus","readOnly":true},{"name":"data","mountPath":"/prometheus"}]);
        resource(files, &mut resources, "prometheus.yaml", prom);
        resource(
            files,
            &mut resources,
            "prometheus-service.yaml",
            service("prometheus", "prometheus", 9090, 9090),
        );
        let ds = files
            [&PathBuf::from("infra/monitoring/grafana/provisioning/datasources/prometheus.yaml")]
            .clone();
        let provider = files
            [&PathBuf::from("infra/monitoring/grafana/provisioning/dashboards/default.yaml")]
            .clone();
        let dash =
            files[&PathBuf::from("infra/monitoring/grafana/dashboards/application.json")].clone();
        resource(
            files,
            &mut resources,
            "grafana-config.yaml",
            configmap(
                "grafana",
                json!({"datasource.yaml":ds,"provider.yaml":provider,"application.json":dash}),
            ),
        );
        let mut grafana = workload("grafana", GRAFANA, 3000);
        let spec = &mut grafana["spec"]["template"]["spec"];
        spec["volumes"] =
            json!([{"name":"config","configMap":{"name":"grafana"}},{"name":"data","emptyDir":{}}]);
        spec["containers"][0]["env"] = json!([{"name":"GF_SECURITY_ADMIN_PASSWORD","valueFrom":{"secretKeyRef":{"name":"app-secrets","key":"GRAFANA_ADMIN_PASSWORD"}}},{"name":"GF_USERS_ALLOW_SIGN_UP","value":"false"}]);
        spec["containers"][0]["volumeMounts"] = json!([{"name":"config","mountPath":"/etc/grafana/provisioning/datasources/prometheus.yaml","subPath":"datasource.yaml","readOnly":true},{"name":"config","mountPath":"/etc/grafana/provisioning/dashboards/default.yaml","subPath":"provider.yaml","readOnly":true},{"name":"config","mountPath":"/var/lib/grafana/dashboards/application.json","subPath":"application.json","readOnly":true},{"name":"data","mountPath":"/var/lib/grafana"}]);
        resource(files, &mut resources, "grafana.yaml", grafana);
        resource(
            files,
            &mut resources,
            "grafana-service.yaml",
            service("grafana", "grafana", 3000, 3000),
        );
    }
    if vault {
        let agent = files[&PathBuf::from("infra/vault/agent.kubernetes.hcl")].clone();
        resource(
            files,
            &mut resources,
            "vault-agent.yaml",
            configmap("vault-agent", json!({"agent.hcl":agent})),
        );
        resource(
            files,
            &mut resources,
            "api-account.yaml",
            json!({"apiVersion":"v1","kind":"ServiceAccount","metadata":{"name":"api"}}),
        );
        if r.infrastructure.vault_location == Some(Location::Local) {
            let server = files[&PathBuf::from("infra/vault/server.hcl")].clone();
            resource(
                files,
                &mut resources,
                "vault-config.yaml",
                configmap("vault-config", json!({"server.hcl":server})),
            );
            stateful(
                files,
                &mut resources,
                "vault",
                VAULT,
                8200,
                "/vault/file",
                json!([]),
                Some(json!([
                    "vault",
                    "server",
                    "-config=/vault/config/server.hcl"
                ])),
                true,
            );
            resource(
                files,
                &mut resources,
                "vault-account.yaml",
                json!({"apiVersion":"v1","kind":"ServiceAccount","metadata":{"name":"vault"}}),
            );
            resource(
                files,
                &mut resources,
                "vault-review.yaml",
                json!({"apiVersion":"rbac.authorization.k8s.io/v1","kind":"ClusterRoleBinding","metadata":{"name":format!("{namespace}-vault-tokenreview")},"subjects":[{"kind":"ServiceAccount","name":"vault","namespace":namespace}],"roleRef":{"apiGroup":"rbac.authorization.k8s.io","kind":"ClusterRole","name":"system:auth-delegator"}}),
            );
        }
    }
    insert(
        files,
        Path::new(""),
        "infra/kubernetes/base/kustomization.yaml",
        encoded(
            &json!({"apiVersion":"kustomize.config.k8s.io/v1beta1","kind":"Kustomization","namespace":namespace,"resources":resources}),
        ),
    );
    insert(
        files,
        Path::new(""),
        "infra/kubernetes/overlays/dev/kustomization.yaml",
        encoded(
            &json!({"apiVersion":"kustomize.config.k8s.io/v1beta1","kind":"Kustomization","resources":["../../base"]}),
        ),
    );
    Ok(())
}
fn secret_env(key: &str) -> Value {
    json!({"name":key,"valueFrom":{"secretKeyRef":{"name":"app-secrets","key":key}}})
}
fn service(name: &str, app: &str, port: u16, target: u16) -> Value {
    json!({"apiVersion":"v1","kind":"Service","metadata":{"name":name},"spec":{"selector":{"app":app},"ports":[{"port":port,"targetPort":target}]}})
}
fn configmap(name: &str, data: Value) -> Value {
    json!({"apiVersion":"v1","kind":"ConfigMap","metadata":{"name":name},"data":data})
}
fn resource(files: &mut Files, resources: &mut Vec<String>, name: &str, value: Value) {
    resources.push(name.into());
    insert(
        files,
        Path::new(""),
        &format!("infra/kubernetes/base/{name}"),
        encoded(&value),
    );
}
fn workload(name: &str, image: &str, port: u16) -> Value {
    json!({"apiVersion":"apps/v1","kind":"Deployment","metadata":{"name":name},"spec":{"replicas":1,"selector":{"matchLabels":{"app":name}},"template":{"metadata":{"labels":{"app":name}},"spec":{"securityContext":{"runAsNonRoot":true,"runAsUser":1000,"fsGroup":1000},"containers":[{"name":name,"image":image,"ports":[{"containerPort":port}],"resources":{"requests":{"cpu":"100m","memory":"128Mi"},"limits":{"cpu":"1","memory":"512Mi"}}}]}}}})
}
#[allow(clippy::too_many_arguments)]
fn stateful(
    files: &mut Files,
    resources: &mut Vec<String>,
    name: &str,
    image: &str,
    port: u16,
    path: &str,
    env: Value,
    command: Option<Value>,
    vault: bool,
) {
    let mut container = json!({"name":name,"image":image,"ports":[{"containerPort":port}],"env":env,"volumeMounts":[{"name":"data","mountPath":path}],"resources":{"requests":{"cpu":"100m","memory":"256Mi"},"limits":{"cpu":"1","memory":"1Gi"}}});
    if let Some(cmd) = command {
        container["command"] = cmd;
    }
    let mut spec = json!({"containers":[container]});
    if vault {
        spec["serviceAccountName"] = json!("vault");
        spec["securityContext"] = json!({"runAsUser":1000,"fsGroup":1000});
        spec["volumes"] = json!([{"name":"config","configMap":{"name":"vault-config"}}]);
        spec["containers"][0]["volumeMounts"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":"config","mountPath":"/vault/config","readOnly":true}));
    }
    let mut svc = service(name, name, port, port);
    svc["spec"]["clusterIP"] = json!("None");
    resource(files, resources, &format!("{name}-service.yaml"), svc);
    resource(
        files,
        resources,
        &format!("{name}.yaml"),
        json!({"apiVersion":"apps/v1","kind":"StatefulSet","metadata":{"name":name},"spec":{"replicas":1,"serviceName":name,"selector":{"matchLabels":{"app":name}},"template":{"metadata":{"labels":{"app":name}},"spec":spec},"volumeClaimTemplates":[{"metadata":{"name":"data"},"spec":{"accessModes":["ReadWriteOnce"],"resources":{"requests":{"storage":"1Gi"}}}}]}}),
    );
}
fn vault_pod(spec: &mut Value, _files: &Files) -> Result<(), Error> {
    spec["serviceAccountName"] = json!("api");
    spec["volumes"] = json!([{"name":"secrets","emptyDir":{"medium":"Memory"}},{"name":"vault-config","configMap":{"name":"vault-agent"}}]);
    spec["initContainers"] = json!([{"name":"vault-agent","image":VAULT,"command":["vault","agent","-config=/vault/config/agent.hcl"],"volumeMounts":[{"name":"secrets","mountPath":"/vault/secrets"},{"name":"vault-config","mountPath":"/vault/config","readOnly":true}],"securityContext":{"runAsNonRoot":true,"runAsUser":1000,"allowPrivilegeEscalation":false}}]);
    spec["containers"][0]["volumeMounts"] =
        json!([{"name":"secrets","mountPath":"/vault/secrets","readOnly":true}]);
    Ok(())
}
