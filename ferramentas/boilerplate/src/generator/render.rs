use super::writer::ProjectFiles;
use crate::{
    Error,
    catalog::{ASSETS, Catalog, Status},
    config::*,
    planner::Plan,
};
use std::{collections::BTreeMap, path::PathBuf};

const GITIGNORE: &str = "node_modules/\ndist/\nbuild/\n.next/\ntarget/\ncoverage/\n*.log\n*.tsbuildinfo\n.env\n.env.*\n!.env.example\n.DS_Store\n.vscode/\n.idea/\n";

fn put(files: &mut BTreeMap<PathBuf, String>, path: PathBuf, content: String) -> Result<(), Error> {
    if !super::writer::safe_path(&path) {
        return Err(Error::input("Caminho inválido no template"));
    }
    if files.insert(path.clone(), content).is_some() {
        return Err(Error::operation(format!(
            "Template duplicou arquivo: {}",
            path.display()
        )));
    }
    Ok(())
}

pub fn render(plan: &Plan, catalog: &Catalog) -> Result<Vec<ProjectFiles>, Error> {
    // Fail before any writes; a plan may describe future integrations, new never pretends to implement them.
    let unsupported: Vec<_> = plan
        .integrations
        .iter()
        .filter(|i| i.as_str() != "compose")
        .collect();
    if !unsupported.is_empty() {
        return Err(Error::input(format!(
            "Geração dessas integrações ainda indisponível: {unsupported:?}. Nenhum arquivo foi criado. Use plan para revisar o escopo futuro."
        )));
    }
    let r = &plan.configuration;
    let name = r.project.name.as_deref().unwrap();
    let full = r.project.kind == Some(ProjectType::Fullstack);
    let mono = r.project.layout == Some(Layout::Monorepo);
    let compose = r.infrastructure.containers == Some(Containers::Compose);
    let web_port = r
        .frontend
        .template
        .as_deref()
        .map(|id| catalog.get(id))
        .transpose()?
        .map(|t| t.port)
        .filter(|p| *p != 0)
        .unwrap_or(5173);
    let mut roots: BTreeMap<PathBuf, BTreeMap<PathBuf, String>> = BTreeMap::new();
    for app in &plan.applications {
        let t = catalog.get(&app.template)?;
        if t.status != Status::Ready {
            return Err(Error::input(format!(
                "Template {} ainda não está disponível para gerar",
                t.id
            )));
        }
        let system = if r.toolchain.module_system == ModuleSystem::Esm {
            "esm"
        } else {
            "commonjs"
        };
        if !t.module_systems.is_empty() && !t.module_systems.iter().any(|s| s == system) {
            return Err(Error::input(format!(
                "{} não suporta module-system {system}",
                t.id
            )));
        }
        let root = if mono {
            app.destination
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        } else {
            app.destination.clone()
        };
        let prefix = app.destination.strip_prefix(&root).unwrap();
        let files = roots.entry(root).or_default();
        let app_name = if full {
            format!(
                "{name}-{}",
                if t.role == ProjectType::Frontend {
                    "web"
                } else {
                    "api"
                }
            )
        } else {
            name.into()
        };
        let mut count = 0;
        for (_, asset_path, source) in ASSETS.iter().filter(|(id, _, _)| *id == app.template) {
            let asset_path = if let Some(rest) = asset_path.strip_prefix("@compose/") {
                if compose { rest } else { continue }
            } else {
                asset_path
            };
            let content = source
                .replace("{{PROJECT_NAME}}", &app_name)
                .replace("{{RUST_CRATE}}", &app_name.replace('-', "_"))
                .replace("{{NODE_VERSION}}", &r.toolchain.node_version)
                .replace("{{NODE_ENGINE}}", &node_engine(&r.toolchain.node_version))
                .replace(
                    "{{PACKAGE_TYPE}}",
                    if system == "esm" {
                        "module"
                    } else {
                        "commonjs"
                    },
                )
                .replace("{{TS_MODULE}}", "NodeNext")
                .replace("{{WEB_PORT}}", &web_port.to_string());
            if content.contains("{{") {
                return Err(Error::operation(format!(
                    "Placeholder não resolvido: {asset_path}"
                )));
            }
            put(files, prefix.join(asset_path), content)?;
            count += 1;
        }
        if count == 0 {
            return Err(Error::operation(format!("Template {} sem fontes", t.id)));
        }
        put(files, prefix.join(".gitignore"), GITIGNORE.into())?;
        if t.requirements.iter().any(|r| r.command == "node") {
            put(
                files,
                prefix.join(".nvmrc"),
                format!("{}\n", r.toolchain.node_version),
            )?;
            put(
                files,
                prefix.join(".npmrc"),
                "engine-strict=true\nsave-exact=true\n".into(),
            )?;
        }
        put(
            files,
            prefix.join("AGENTS.md"),
            agent_rules(&app_name, &t.package_manager),
        )?;
        put(
            files,
            prefix.join("README.md"),
            app_readme(&app_name, &t.id, &t.package_manager, web_port, compose),
        )?;
    }
    let mut projects = Vec::new();
    for (root, mut files) in roots {
        if mono {
            put(&mut files, PathBuf::from(".gitignore"), GITIGNORE.into())?;
            put(&mut files, PathBuf::from("AGENTS.md"), "# Estrutura do monorepo\n\nAplicações em apps/web e apps/api têm instruções próprias. Execute instalação/build/test no diretório de cada aplicação. Preserve a separação web/API e não exponha segredos ao browser. Não instale ferramentas globais ou publique repositórios sem autorização explícita.\n".into())?;
            put(
                &mut files,
                PathBuf::from("README.md"),
                format!(
                    "# {name}\n\nFrontend em [apps/web](apps/web/README.md) e API em [apps/api](apps/api/README.md).\n\nInstale e execute cada aplicação em seu diretório; nenhum serviço foi iniciado automaticamente. A API usa http://localhost:3000 e o frontend http://localhost:{web_port}. CORS e cliente HTTP já usam essas origens locais.\n\n{}",
                    if compose {
                        "Para containers, execute `docker compose config`, depois `docker compose up --build` nesta raiz. Para parar sem apagar dados: `docker compose down`.\n"
                    } else {
                        ""
                    }
                ),
            )?;
        }
        let mut recipe = r.clone();
        recipe.project.destination = None;
        if !mono && full {
            let app = plan
                .applications
                .iter()
                .find(|a| a.destination == root)
                .unwrap();
            let t = catalog.get(&app.template)?;
            recipe.project.kind = Some(t.role);
            recipe.project.name = Some(format!(
                "{name}-{}",
                if t.role == ProjectType::Frontend {
                    "web"
                } else {
                    "api"
                }
            ));
            recipe.project.layout = None;
            recipe.project.web_dir = None;
            recipe.project.api_dir = None;
            if t.role == ProjectType::Frontend {
                recipe.backend = Backend::default();
            } else {
                recipe.frontend = Frontend::default();
            }
        }
        put(
            &mut files,
            "boilerplate.toml".into(),
            toml::to_string_pretty(&recipe).map_err(|e| Error::operation(e.to_string()))?,
        )?;
        if compose {
            let mut yaml = String::from("services:\n");
            for app in plan
                .applications
                .iter()
                .filter(|app| mono || app.destination == root)
            {
                let t = catalog.get(&app.template)?;
                let web = t.role == ProjectType::Frontend;
                let service = if web { "web" } else { "api" };
                let context = if mono {
                    if web { "./apps/web" } else { "./apps/api" }
                } else {
                    "."
                };
                let port = if web { web_port } else { 3000 };
                let internal = if web && t.id != "next-ts" { 8080 } else { port };
                yaml.push_str(&format!("  {service}:\n    build:\n      context: {context}\n    ports:\n      - \"127.0.0.1:{port}:{internal}\"\n"));
                if !web {
                    yaml.push_str(&format!("    environment:\n      PORT: \"3000\"\n      FRONTEND_ORIGIN: http://localhost:{web_port}\n"));
                }
            }
            put(&mut files, "compose.yaml".into(), yaml)?;
        }
        projects.push(ProjectFiles {
            destination: root,
            files,
        });
    }
    Ok(projects)
}

fn node_engine(version: &str) -> String {
    let major = version.split('.').next().unwrap();
    format!(
        ">={version} <{}.0.0",
        major.parse::<u32>().unwrap_or(24) + 1
    )
}

fn agent_rules(name: &str, manager: &str) -> String {
    let validation = match manager {
        "npm" => "npm run typecheck && npm test && npm run build",
        "cargo" => "cargo fmt --check && cargo test && cargo clippy -- -D warnings",
        _ => "Servir por HTTP, verificar carregamento e a integração com a API no navegador",
    };
    format!(
        "# {name}: instruções de manutenção\n\nLeia README.md e boilerplate.toml quando presente. Preserve as convenções existentes e a separação de responsabilidades.\n\n- Validação: `{validation}`.\n- Instale dependências localmente; mantenha o lockfile e não adicione CLIs globais.\n- Use import/export nos fontes JS/TS; module-system está no package.json.\n- Não versione .env, tokens, credenciais, node_modules, builds ou target.\n- Ao alterar rotas, configuração ou dependências, atualize documentação e testes de comportamento.\n- Dados/cache/Vault não fazem parte da base sem uma integração implementada.\n- Não publique, faça push ou altere configurações globais sem instrução explícita.\n"
    )
}

fn app_readme(name: &str, template: &str, manager: &str, web_port: u16, compose: bool) -> String {
    let commands = match manager {
        "npm" => {
            "```sh\n# Se necessário, carregue o nvm: . \"$HOME/.nvm/nvm.sh\"\nnvm install\nnvm use\nnpm ci\nnpm run typecheck\nnpm test\nnpm run build\nnpm run dev\n```\n\nPara ambiente com lockfile, prefira `npm ci`. O runtime local já é verificado pela engine e .nvmrc. As dependências ficam em node_modules, sem instalar framework/TypeScript globalmente.\n"
        }
        "cargo" => {
            "```sh\ncargo fetch\ncargo fmt --check\ncargo test\ncargo run\n```\n\nVersione Cargo.lock após a instalação.\n"
        }
        _ => {
            "```sh\npython3 -m http.server 5173 --bind 127.0.0.1\n```\n\nAbra http://localhost:5173. Python é apenas uma opção de servidor HTTP; não há dependências npm. Não abra diretamente por file://, pois os módulos precisam de HTTP.\n"
        }
    };
    format!(
        "# {name}\n\nBase `{template}` criada pelo boilerplate para WSL/Linux. Execute os comandos nesta pasta.\n\n## Instalar, validar e executar\n\n{commands}\n## Configuração e integração\n\nAPI: http://localhost:3000, `/health` e `/api/v1/example`. Frontend: http://localhost:{web_port}. O cliente usa URL acessível ao navegador, e não DNS de container. `VITE_API_URL` e `NEXT_PUBLIC_API_URL` são públicas e aplicadas no build; nunca coloque segredos nelas.\n\nQuando .env.example existir, copie para .env antes de personalizar. Não versione .env. Os scripts Node não carregam .env automaticamente: exporte as variáveis no shell (ou use `node --env-file=.env` no comando de execução após compilar). Vite/Next carregam seus próprios arquivos .env. O backend Rust também lê o ambiente do processo.\n\nEstrutura: controllers recebem HTTP; services concentram regras; repositories só entram com persistência implementada. O backend demonstra chamadas stateless e não simula um banco. Os fontes TypeScript usam import/export, com ESM por padrão; uma opção CommonJS muda a saída compilada, não o estilo dos fontes.\n\n{}\n## Manutenção\n\nConsulte AGENTS.md. boilerplate.toml registra opções sem credenciais. Ao modificar dependências, atualize o lockfile e rode os checks acima. Um projeto criado não implica que GitHub, Docker ou serviços tenham sido iniciados.\n",
        if compose {
            "## Containers\n\nDockerfile e .dockerignore acompanham a aplicação. Execute `docker --version` e `docker compose version` antes de usar. O Compose fica na raiz do projeto/repositório. Revise com `docker compose config`, execute `docker compose up --build` e pare com `docker compose down`. Um projeto separado usa URLs do host para conectar browser/API. Nenhum desses comandos é executado pelo gerador.\n"
        } else {
            ""
        }
    )
}
