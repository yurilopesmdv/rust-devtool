# Manutenção por pessoas e agentes

[Guia de uso](../README.md) · [Adicionar stack](adicionar-stack.md)

## Contratos estáveis

Use receita TOML e flags para automação, sem simular teclado. `new --non-interactive --install yes|no --github yes|no --push yes|no` expressa os efeitos permitidos. `--json` publica o resumo em stdout; progresso de instalações/Git vai para stderr. `plan --json` continua sem escrita/execução de ferramentas.

Os projetos gerados recebem `AGENTS.md` com comandos de validação, separação de camadas, tratamento de segredos e regras para manter dependências locais. O arquivo não depende de um produto específico e pode ser adaptado pelo responsável do projeto. `boilerplate.toml`, README e lockfiles tornam as escolhas verificáveis.

## Onde alterar o gerador

- `catalog/*.toml`: IDs, papéis, linguagem, comandos, portas e capacidades. IDs são consultados no catálogo; não criar switches de linguagens na CLI.
- `templates/<id>/`: fontes e lockfiles embutidos. `@compose/` só entra quando Compose é selecionado. `build.rs` descobre os arquivos automaticamente.
- `src/generator/render.rs`: substituições, arquivos comuns e composição por layout. Não executar ferramentas aqui.
- `writer.rs`: staging e publicação sem sobrescrita. Não apagar projetos já publicados em uma falha posterior.
- `runtime.rs`: detecção Node/nvm e execução da instalação solicitada. Não instalar frameworks globalmente.
- `vcs.rs`: Git/GitHub e checagens; nenhuma publicação sem escolha explícita.
- `wizard.rs`: perguntas, convertidas na mesma Recipe das flags/TOML.
- `cli.rs`/`config.rs`/`planner.rs`: interface, schema e validação compartilhada.

Placeholders dos assets: `{{PROJECT_NAME}}`, `{{RUST_CRATE}}`, `{{NODE_VERSION}}`, `{{NODE_ENGINE}}`, `{{PACKAGE_TYPE}}`, `{{TS_MODULE}}` e `{{WEB_PORT}}`. Reserve `{{` para esses tokens. Variáveis são substituídas como dados em arquivos; os comandos externos usam programa/argumentos separados.

## Antes de anunciar uma opção disponível

Testar geração, build e comportamento; verificar a ausência quando desativada. Não aceitar uma flag só para ignorar seu efeito. Os adapters atuais estão em src/generator/integrations; uma nova stack ainda precisa implementar seus contratos antes de anunciar capacidades.

Ao alterar dependências, instale em um projeto temporário gerado, rode checks/build reais, atualize o lockfile do template e normalize o nome/engine com placeholders. Não use `latest` para resolver uma dependência da base em cada geração. A primeira instalação do binário pode acessar crates.io; a geração com `--install no` não baixa scaffolding.

Comandos na raiz do repositório:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Para Node, validar `npm ci`, typecheck, testes e build nas bases geradas. Para Rust, `cargo test --locked`, formatação e Clippy. Verificar ESM e a opção CommonJS do Express, monorepo/split, destino existente e falha de instalação. Testes GitHub usam executáveis simulados; não criar repositórios reais como parte da suíte.

A revisão atual validou Node 24.15.0, Rust 1.99.1 e os lockfiles distribuídos. Novas linguagens continuam usando metadados e assets próprios; seu provisionamento de runtime e adapters exigem validação adicional antes de serem disponibilizados.

A composição Node poda lockfiles de apoio por dependências alcançáveis, removendo drivers não escolhidos. Rust distribui snapshots de Cargo.lock por combinação; ao alterar dependências/flags de features, atualize essas combinações e execute validações com --locked. Para validação reproduzível, veja [validacao.md](validacao.md).
