# Validação e reprodução dos testes

[Guia de uso](../README.md) · [Integrações](integracoes.md)

## Suíte do repositório

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Os 59 testes do workspace incluem uma matriz de 72 combinações de backend/dados/cache/monitoria/Vault, mais 24 combinações Python e verificações das doze duplas fullstack, conferindo ausência de módulos desativados, lockfiles, serviços locais/externos, referências a segredos, layouts e URLs. Não iniciam bancos, containers ou clusters.

## Compilar as aplicações geradas

O script opcional requer Python 3.8+ para orquestração e Rust/Cargo. Os checks das aplicações usam Node 24 para stacks npm, rustfmt/Clippy para Rust e uv para Python 3.13. Cria projetos em diretório temporário, instala dependências locais e executa typecheck/build/test. Docker CLI e kubectl, quando presentes, validam Compose e Kustomize sem daemon/cluster.

```sh
nvm use 24
uv --version
python3 ferramentas/boilerplate/scripts/validate-generated.py
# Somente bases sem integrações:
python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal
# Seleção específica:
python3 ferramentas/boilerplate/scripts/validate-generated.py --stacks python-fastapi nest-ts
```

Ele não instala ferramentas globais, não inicia serviços e não publica no GitHub. `--keep` preserva os fontes para revisão. `--offline` exige caches preenchidos; `npm_config_cache` e `CARGO_TARGET_DIR` podem indicar caches explícitos para reutilizar downloads/compilações.

## Persistência real, com serviços de teste

Use bancos/cache dedicados, pois a suíte prepara a tabela/coleção `examples` e grava a entrada de demonstração. Configure POSTGRES_TEST_URL, MONGODB_TEST_URL e REDIS_TEST_URL no ambiente sem versionar ou expor credenciais. Depois:

```sh
python3 ferramentas/boilerplate/scripts/validate-generated.py --with-services --keep
```

Use `--commonjs-only` para validar especificamente Express/CommonJS com os dois bancos. O script não provisiona esses serviços. Ele testa Express/Nest/Rust/Python com ambos os bancos, faz preparação explícita, executa os testes HTTP de leitura/escrita/validação/métricas das APIs Node os testes de persistência Rust e os testes HTTP de Python. Sem as variáveis, os testes dependentes de banco são explicitamente pulados; não se deve confundir isso com validação de persistência.

## Evidência desta entrega

Ambiente de validação: WSL/Ubuntu 20.04, Node 24.15.0 e Rust 1.99.1.

- Quatro combinações Node ESM (Express/Nest × PostgreSQL/MongoDB) e duas Express/CommonJS: npm ci, typecheck, build e testes, incluindo execução com serviços autenticados.
- Duas combinações Rust: Cargo.lock, testes, Clippy e testes de persistência reais com --ignored. Casos cache sem banco e Vault sem dados também foram compilados/testados.
- PostgreSQL 17.6 e Redis 7.4.5 compilados somente em /tmp; MongoDB 7.0.14 compatível com o host para o teste de protocolo/CRUD. A imagem gerada de MongoDB é 8.0.13 e não foi executada neste ambiente.
- Vault 1.20.2 Linux oficial, com SHA256 verificado: Agent real autenticou por AppRole e renderizou os arquivos de credenciais a partir da policy gerada. O teste usou servidor isolado e dados fictícios.
- Promtool 3.5.0 oficial validou configurações Prometheus e as métricas HTTP de aplicações Node/Rust em processos locais.
- Compose/Kustomize e manifests foram renderizados e conferidos contra o OpenAPI oficial Kubernetes 1.34. As tags de imagens dos serviços foram verificadas no registry.

O daemon Docker não estava acessível e não havia cluster de teste. Por isso não foram executadas imagens Docker, deploys, autenticação Vault/Kubernetes ou interface Grafana em container. As configurações são geradas e validadas, mas essa etapa operacional precisa ser confirmada no ambiente de destino. Nenhum serviço pessoal foi usado; processos temporários foram encerrados.

## Revisão de arquitetura, Python e qualidade

A revisão das camadas foi validada com Express/Nest/Rust/Python × PostgreSQL/MongoDB: oito combinações com serviços reais dedicados e autenticados, preparação, escrita/leitura e invalidação de cache. Os processos foram encerrados ao final. MongoDB local continuou na versão 7 compatível com o host; não é execução da imagem 8 distribuída.

ESLint/dprint e typecheck/build/test passaram nas bases Node, inclusive Next/Vite/vanilla; Ruff check/format e pytest passaram nas 24 combinações de Python (banco/cache/métricas/Vault). Express/CommonJS também passou com ambos os bancos, agora com TypeScript 6 e os novos checks. As APIs Rust passaram em cargo fmt --check, testes, Clippy e persistência real após mover consultas para repositories. O teste de service Nest verifica a dependência injetada sem abrir conexão. Os manifests de Python foram renderizados por Compose/Kustomize.

O gerador não depende de CLIs globais de JavaScript/Python para gerar arquivos. Gerar Rust requer rustfmt. A suíte agora verifica formatação sem alterá-la. Para complementar lint estático com verificação de dados reais, use --with-services e serviços descartáveis dedicados; não use bancos pessoais ou de produção.

O middleware Express também foi verificado em ESM/CommonJS com requisições reais de JSON inválido (400) e payload grande (413), sem depender de banco para esses casos.

O workflow Quality agora separa o workspace, bases mínimas npm/Python/Rust e persistência com serviços descartáveis. A validação local da T006 também verifica readiness após fechar a conexão (503) e liveness preservada (200) nas dez combinações de backend/banco, contando Express/CommonJS. A primeira execução dos jobs/imagens no GitHub ainda precisa ser observada; veja docs/desenvolvimento/tasks/T006.md na raiz do repositório.

## Limites e interrupção

No Linux/WSL, instalações executadas pelo gerador têm limite de 900 segundos por comando. Ajuste com `BOILERPLATE_COMMAND_TIMEOUT_SECONDS` (inteiro positivo). Timeout retorna 124; Ctrl-C retorna 130; SIGTERM retorna 143. O grupo do comando recebe TERM e, após uma tolerância curta, KILL. Os arquivos publicados permanecem: entre na aplicação e retome `npm ci`, `uv sync --locked` ou `cargo fetch --locked`, conforme seu README. Nenhum serviço é iniciado por essa etapa.

O runner validate-generated.py limita cada comando a 900 segundos. A manutenção de locks limita cada resolução/instalação a 300 segundos. Timeout e Ctrl-C encerram o grupo Linux, incluindo filhos; comandos destacados voluntariamente em outra sessão não pertencem ao grupo supervisionado. Os testes usam processos falsos lentos e verificam que os filhos deixam de executar.

## Smoke Docker opcional

Use somente um contexto Docker Linux descartável dedicado a testes. O comando verifica o daemon existente; não instala/inicia Docker e não participa de new:

```sh
python3 ferramentas/boilerplate/scripts/docker_smoke.py --disposable-daemon --stack express-ts
```

As oito bases web são aceitas por --stack. O teste gera sem instalar no host/Git/GitHub, constrói a imagem mínima, verifica UID não root e arquivo principal, publica uma porta somente no loopback e aguarda /ready (APIs) ou / (frontends). Depois envia stop e rejeita OOM/SIGKILL. Nomes e labels são exclusivos por execução; finally remove somente o container e tag próprios, sem prune ou remoção de volumes. Imagens base e cache de build podem permanecer no daemon de teste. Não execute contra um daemon de produção.

No workflow Quality, a opção manual docker_smoke é false por padrão e habilita uma matriz separada em runners descartáveis. Builds precisam de acesso aos registries, rede e espaço; o limite é 35 minutos por stack. A validação local do runner cobre cleanup simulado em sucesso/falha; a execução real das oito imagens mínimas foi aprovada em runners descartáveis do GitHub; o ambiente local continua sem daemon disponível.

## Assistente interativo em PTY

Após `cargo build -p rust-boilerplate --locked`, execute:

```sh
python3 ferramentas/boilerplate/scripts/wizard_pty.py --binary target/debug/boilerplate
```

O teste Linux/WSL abre um terminal real, usa HOME temporário e PATH sem instaladores, verifica recuperação de escolhas inválidas, aceita o destino HOME/projects e cria vanilla sem instalação/Git. Uma segunda sessão recusa a confirmação: código 130, nenhum destino ou staging publicado. Prompts e término têm limites para evitar travar a suíte. A CI roda esse teste depois dos checks do workspace.

### Executar a validação antes de integrar na main

O workflow também aceita push para branches `validation/**`. As branches `validation/docker-*` habilitam explicitamente a matriz de smoke Docker junto dos demais checks, em runners descartáveis. Isso permite verificar um workflow novo antes de ele existir na branch padrão. Branches de validação comuns executam os demais checks e mantêm o smoke desativado. O gerador não publica essas branches; o envio é uma ação separada do mantenedor.

O runner tolera resets de conexão durante o startup até atingir o prazo. Cleanup tenta remover container e imagem mesmo se uma remoção falhar; se já houver erro de readiness/build, ele permanece como causa principal e falhas de limpeza aparecem em stderr.

## Evidência remota concluída

[Quality 40527937252](https://github.com/yurilopesmdv/rust-devtool/actions/runs/40527937252), commit c3df5a0: 26 jobs aprovados em 2026-10-27. Inclui workspace, bases, persistência com serviços descartáveis, locks, documentação e oito smoke tests reais. O primeiro run detectou /ready ausente em Express/Nest/Rust mínimos; a correção passou nos testes HTTP e nas imagens. Não houve aplicação de manifests em cluster nem smoke da composição completa com Vault/monitoria; esses testes não equivalem à validação de um ambiente de produção.
