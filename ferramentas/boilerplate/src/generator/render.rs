use super::writer::ProjectFiles;
use crate::{
    Error,
    catalog::{ASSETS, Catalog, Status},
    config::*,
    planner::Plan,
};
use std::{collections::BTreeMap, path::PathBuf};

const GITIGNORE: &str = "node_modules/\ndist/\nbuild/\n.next/\ntarget/\ncoverage/\nbin/\n*.log\n*.tsbuildinfo\n.venv/\n__pycache__/\n*.py[cod]\n.pytest_cache/\n.ruff_cache/\n.env\n.env.*\n!.env.example\n.DS_Store\n.vscode/\n.idea/\n";

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
    render_assets(plan, catalog, ASSETS)
}

fn render_assets(
    plan: &Plan,
    catalog: &Catalog,
    assets: &[(&str, &str, &str)],
) -> Result<Vec<ProjectFiles>, Error> {
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
    let default_origin = format!("http://localhost:{web_port}");
    let frontend_origin = r
        .infrastructure
        .frontend_origin
        .as_deref()
        .unwrap_or(&default_origin);
    let api_url = r
        .infrastructure
        .api_url
        .as_deref()
        .unwrap_or("http://localhost:3000");
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
        for (_, asset_path, source) in assets.iter().filter(|(id, _, _)| *id == app.template) {
            let asset_path = if let Some(rest) = asset_path.strip_prefix("@compose/") {
                if compose { rest } else { continue }
            } else {
                asset_path
            };
            let content = source
                .replace("http://localhost:{{WEB_PORT}}", frontend_origin)
                .replace("http://localhost:3000", api_url)
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
        super::integrations::backend(r, t, files, prefix)?;
        super::integrations::frontend(r, t, files, prefix)?;
        if t.role == ProjectType::Backend {
            put(
                files,
                prefix.join("docs/ARQUITETURA.md"),
                super::architecture::guide(t),
            )?;
        }
        // Preserve the declared architecture even where a folder has no source yet.
        for directory in t.paths.iter().filter(|path| path.ends_with('/')) {
            let directory = prefix.join(directory);
            if !files.keys().any(|path| path.starts_with(&directory)) {
                put(files, directory.join(".gitkeep"), String::new())?;
            }
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
                    "# {name}\n\nFrontend em [apps/web](apps/web/README.md) e API em [apps/api](apps/api/README.md).\n\nInstale e execute cada aplicação em seu diretório; nenhum serviço foi iniciado automaticamente. A API usa http://localhost:3000 e o frontend http://localhost:{web_port} por padrão. Confira as URLs efetivas nos READMEs das aplicações e em boilerplate.toml.\n\n{}",
                    if compose {
                        "Para containers, copie infra/.env.example para infra/.env e ajuste os valores. Execute `docker compose --env-file infra/.env config`, depois `docker compose --env-file infra/.env up --build` nesta raiz. Para parar sem apagar dados: `docker compose --env-file infra/.env down`.\n"
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
            recipe.infrastructure.image = if t.role == ProjectType::Frontend {
                recipe.infrastructure.image_web.take()
            } else {
                recipe.infrastructure.image_api.take()
            };
            if recipe.infrastructure.kubernetes == Some(Kubernetes::Manifests)
                && recipe.infrastructure.image.is_none()
            {
                recipe.infrastructure.image = Some(format!(
                    "{name}-{}:0.1.0",
                    if t.role == ProjectType::Frontend {
                        "web"
                    } else {
                        "api"
                    }
                ));
            }
            recipe.infrastructure.image_web = None;
            recipe.infrastructure.image_api = None;
            recipe.infrastructure.frontend_origin = Some(frontend_origin.to_string());
            recipe.infrastructure.api_url = Some(api_url.to_string());
            if t.role == ProjectType::Frontend {
                recipe.backend = Backend::default();
                recipe.infrastructure.secrets = Some(Secrets::Env);
                recipe.infrastructure.vault_location = None;
                recipe.infrastructure.vault_address = None;
                recipe.infrastructure.monitoring = Some(Monitoring::None);
            } else {
                recipe.frontend = Frontend::default();
            }
        }
        put(
            &mut files,
            "boilerplate.toml".into(),
            toml::to_string_pretty(&recipe).map_err(|e| Error::operation(e.to_string()))?,
        )?;
        super::integrations::infra::apply(plan, catalog, &root, &mut files)?;
        super::format::apply(&mut files)?;
        projects.push(ProjectFiles {
            destination: root,
            files,
        });
    }
    Ok(projects)
}

pub fn node_engine(version: &str) -> String {
    let major = version.split('.').next().unwrap();
    format!(
        ">={version} <{}.0.0",
        major.parse::<u32>().unwrap_or(24) + 1
    )
}

fn agent_rules(name: &str, manager: &str) -> String {
    let validation = match manager {
        "npm" => "npm run check",
        "uv" => {
            "uv run --locked ruff check . && uv run --locked ruff format --check . && uv run --locked pytest"
        }
        "go" => "go test ./... && go vet ./... && test -z \"$(gofmt -l .)\"",
        "cargo" => {
            "cargo fmt --check && cargo test --locked && cargo clippy --all-targets -- -D warnings"
        }
        _ => "Servir por HTTP, verificar carregamento e a integração com a API no navegador",
    };
    format!(
        "# {name}: instruções de manutenção\n\nLeia README.md e boilerplate.toml quando presente. Preserve as convenções existentes e a separação de responsabilidades.\n\n- Validação: `{validation}`.\n- Instale dependências localmente; mantenha o lockfile e não adicione CLIs globais.\n- Use import/export nos fontes JS/TS; module-system está no package.json.\n- Não versione .env, tokens, credenciais, node_modules, builds ou target.\n- Ao alterar rotas, configuração ou dependências, atualize documentação e testes de comportamento.\n- Consulte docs/OPERACAO.md na raiz para os módulos selecionados; não acrescente serviços quando a receita os omite.\n- Não publique, faça push ou altere configurações globais sem instrução explícita.\n"
    )
}

fn app_readme(name: &str, template: &str, manager: &str, web_port: u16, compose: bool) -> String {
    let commands = match manager {
        "npm" => {
            "```sh\n# Se necessário, carregue o nvm: . \"$HOME/.nvm/nvm.sh\"\nnvm install\nnvm use\nnpm ci\nnpm run check\nnpm run dev\n```\n\nPara ambiente com lockfile, prefira `npm ci`. O runtime local já é verificado pela engine e .nvmrc. As dependências ficam em node_modules, sem instalar framework/TypeScript globalmente.\n"
        }
        "uv" => {
            "```sh\nuv sync --locked\ncp .env.example .env\nuv run --locked ruff check .\nuv run --locked ruff format --check .\nuv run --locked pytest\nuv run --locked --env-file .env uvicorn app.main:create_app --factory --host 0.0.0.0 --port 3000 --reload\n```\n\nRequer uv (veja https://docs.astral.sh/uv/getting-started/installation/). uv verifica/obtém Python 3.13 e cria .venv local, sem instalar bibliotecas no Python do sistema. Use uv run --locked ruff format . para formatar. /docs oferece OpenAPI/Swagger.\n"
        }
        "go" => {
            "```sh\ngo version\ngo mod download\ngo test ./...\ngo vet ./...\ntest -z \"$(gofmt -l .)\"\ngo run ./cmd/api\n```\n\nRequer Go 1.27.3 (Linux/WSL). A base usa apenas a biblioteca padrão; sem dependências externas não há go.sum. Quando houver dependências, versione go.mod/go.sum e use go mod verify. Variáveis são lidas do ambiente do processo; .env.example é referência, não há carregador de .env implícito.\n"
        }
        "cargo" => {
            "```sh\ncargo fetch\ncargo fmt\ncargo test\ncargo run\n```\n\nVersione Cargo.lock após a instalação.\n"
        }
        _ => {
            "```sh\npython3 -m http.server 5173 --bind 127.0.0.1\n```\n\nAbra http://localhost:5173. Python é apenas uma opção de servidor HTTP; não há dependências npm. Não abra diretamente por file://, pois os módulos precisam de HTTP.\n"
        }
    };
    format!(
        "# {name}\n\nBase `{template}` criada pelo boilerplate para WSL/Linux. Execute os comandos nesta pasta.\n\n## Instalar, validar e executar\n\n{commands}\n## Configuração e integração\n\nAPI: http://localhost:3000, `/health` e `/api/v1/example`. Frontend: http://localhost:{web_port}. O cliente usa URL acessível ao navegador, e não DNS de container. `VITE_API_URL` e `NEXT_PUBLIC_API_URL` são públicas e aplicadas no build; nunca coloque segredos nelas.\n\nQuando .env.example existir, copie para .env antes de personalizar. Não versione .env. Os scripts dev/start dos backends Node carregam .env quando presente; Vite/Next usam seus carregadores próprios. O backend Rust lê o ambiente do processo: exporte as variáveis no shell.\n\nEstrutura: controllers recebem HTTP; services concentram regras; repositories só entram com persistência implementada. APIs incluem docs/ARQUITETURA.md com o mapa de camadas e o fluxo de dependências. Sem persistência selecionada, o backend é stateless; com banco, veja docs/OPERACAO.md na raiz para preparação e exemplos de leitura/escrita. Os fontes TypeScript usam import/export, com ESM por padrão; uma opção CommonJS muda a saída compilada, não o estilo dos fontes.\n\n{}\n## Manutenção\n\nConsulte AGENTS.md. boilerplate.toml registra opções sem credenciais. Ao modificar dependências, atualize o lockfile e rode os checks acima. Um projeto criado não implica que GitHub, Docker ou serviços tenham sido iniciados.\n",
        if compose {
            "## Containers\n\nDockerfile e .dockerignore acompanham a aplicação. Execute `docker --version` e `docker compose version` antes de usar. O Compose fica na raiz do projeto/repositório. Copie infra/.env.example para infra/.env e ajuste os valores nessa raiz. Revise com `docker compose --env-file infra/.env config`, execute `docker compose --env-file infra/.env up --build` e pare com `docker compose --env-file infra/.env down`. Um projeto separado usa URLs do host para conectar browser/API. Nenhum desses comandos é executado pelo gerador.\n"
        } else {
            ""
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn changed_templates_fail_before_any_destination_is_published() {
        let root = std::env::temp_dir().join(format!(
            "boilerplate-anchor-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let scratch = Scratch(root);
        let catalog = Catalog::embedded().unwrap();
        for (stack, file, defect) in [
            ("express-ts", "src/app.ts", "absent"),
            ("express-ts", "src/app.ts", "duplicated"),
            ("express-ts", "src/app.ts", "missing-source"),
            (
                "nest-ts",
                "src/modules/example/example.module.ts",
                "missing-source",
            ),
        ] {
            let destination = scratch.0.join(format!("{stack}-{defect}"));
            let mut recipe = Recipe::default();
            recipe.project.name = Some("broken-template".into());
            recipe.project.kind = Some(ProjectType::Backend);
            recipe.project.destination = Some(destination.clone());
            recipe.backend.template = Some(stack.into());
            recipe.backend.database = Some(Database::Postgres);
            recipe.backend.database_location = Some(Location::External);
            let plan = crate::planner::build(recipe, &catalog, &scratch.0).unwrap();
            let original = ASSETS
                .iter()
                .find(|(id, path, _)| *id == stack && *path == file)
                .unwrap()
                .2;
            let anchor = "app.disable('x-powered-by');";
            let damaged = if defect == "duplicated" {
                format!("{original}\n{anchor}\n")
            } else {
                original.replace(anchor, "// anchor removed")
            };
            let assets: Vec<_> = ASSETS
                .iter()
                .copied()
                .filter_map(|(id, path, source)| {
                    if id == stack && path == file {
                        if defect == "missing-source" {
                            None
                        } else {
                            Some((id, path, damaged.as_str()))
                        }
                    } else {
                        Some((id, path, source))
                    }
                })
                .collect();
            let error = render_assets(&plan, &catalog, &assets).unwrap_err();
            assert_eq!(error.code, 1);
            assert!(error.message.contains(file), "{}", error.message);
            assert!(!destination.exists());
            assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 0);
        }
    }
}
