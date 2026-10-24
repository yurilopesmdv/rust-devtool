# Boilerplate web — CLI Rust para WSL/Linux

Pacote `rust-boilerplate`, executável `boilerplate`. Este pacote é independente de `ferramentas/devtool`: utilitários de hash/JSON continuam lá; catálogo, planejamento e futuras integrações de projetos ficam aqui.

**Disponível agora:** catálogo, receitas, planos, diagnóstico e `new` com assistente, geração das seis bases, instalação local de dependências, seleção de Node/nvm, Git local e GitHub privado opcional. Dockerfile/.dockerignore/Compose podem ser gerados. As bases estão `ready`; as capacidades avançadas do catálogo continuam sendo intenções, não adapters entregues.

**Ainda no plano:** geração de PostgreSQL/MongoDB/Redis, Kubernetes, Prometheus/Grafana e Vault. `new` rejeita essas integrações antes de criar arquivos; `plan` continua permitindo estudar as combinações. Nenhum serviço é iniciado automaticamente.

## Requisitos e instalação

Execute dentro do WSL/Linux com Rust stable/Cargo e acesso ao crates.io para a primeira compilação. Não há instalador ou suporte para Windows nativo. Para instalar Rust, siga as [instruções oficiais](https://www.rust-lang.org/tools/install) no ambiente Linux.

Da raiz do repositório:

```sh
cargo build -p rust-boilerplate --locked
cargo run -p rust-boilerplate -- --help
```

Para disponibilizar o comando no PATH, opcionalmente:

```sh
cargo install --path ferramentas/boilerplate --locked
boilerplate --help
```

Cargo normalmente instala em `$HOME/.cargo/bin`. Se necessário, carregue o ambiente do Rust na sessão com `. "$HOME/.cargo/env"`. Não é necessário instalar como root nem modificar configurações do Windows. O binário pode ser chamado por Bash, Zsh ou POSIX sh; não execute o binário ELF como `sh boilerplate`.

Os exemplos a seguir usam `cargo run` para funcionar sem instalação global. O workspace continua usando a devtool como membro padrão: `cargo run -- help` mantém seu comportamento anterior.

## Criar um projeto

Para o assistente, na raiz do repositório:

```sh
cargo run -p rust-boilerplate -- new
```

Ele pede nome, tipo, layout fullstack, destino(s), stack(s), versão Node quando aplicável e opção Compose. O destino precisa ser um diretório novo. Para Express sozinho, permite escolher ESM/CommonJS; ESM é o padrão. Mostra o resumo, confirma a criação, inicializa Git local e pergunta sobre dependências, GitHub privado e push separadamente. Quando os argumentos/receita já fornecem as escolhas, o assistente só pergunta o que falta.

Exemplo reproduzível sem perguntas e sem publicar no GitHub:

```sh
cargo run -p rust-boilerplate -- new minha-api \
  --type backend --backend express-ts \
  --destination ./projetos/minha-api \
  --node-version 24 --module-system esm \
  --install yes --setup-node no --github no --non-interactive
```

O caminho é relativo ao diretório onde você chama a CLI. Nesse exemplo, os arquivos são criados em `./projetos/minha-api`, incluindo `.git`, `package.json`, lockfile, `src/`, testes, `.gitignore`, `.nvmrc`, `.npmrc`, README, AGENTS.md e receita `boilerplate.toml`. `--git no` desativa Git local. Sem instalar agora, use `--install no` e os comandos de instalação ficarão no README.

Sem Node compatível, esse exemplo preserva os fontes e informa como instalar. `--setup-node yes` autoriza preparar nvm/Node quando necessário. Em modo interativo, a preparação é oferecida e exige resposta afirmativa. A CLI verifica Node/npm e versões já instaladas no nvm antes de instalar; não reinstala frameworks, TypeScript, Vite, Nest ou Next globalmente.

Mais detalhes: [criação e instalação](docs/criacao-e-instalacao.md), [Git/GitHub](docs/git-e-github.md) e [manutenção por agentes](docs/manutencao-por-agentes.md).

## Consultar o catálogo

```sh
cargo run -p rust-boilerplate -- templates list
cargo run -p rust-boilerplate -- templates show express-ts
cargo run -p rust-boilerplate -- templates list --json
```

Bases: `vanilla-js`, `react-vite-ts`, `next-ts`, `express-ts`, `nest-ts` e `rust-axum`. O catálogo é embutido no binário e não exige um clone no diretório atual nem conexão à internet na execução.

## Planejar sem criar arquivos

```sh
cargo run -p rust-boilerplate -- plan minha-api \
  --type backend --backend express-ts

cargo run -p rust-boilerplate -- plan plataforma \
  --type fullstack --layout monorepo \
  --frontend react-vite-ts --backend rust-axum --json

cargo run -p rust-boilerplate -- plan plataforma \
  --type fullstack --layout split \
  --frontend next-ts --backend nest-ts \
  --web-dir ./plataforma-web --api-dir ./plataforma-api
```

A saída mostra aplicações, destinos absolutos, caminhos pretendidos, comandos futuros de instalação, integrações selecionadas e omitidas. Esses comandos são apenas exibidos. Os caminhos são uma proposta de estrutura, não uma lista final de todos os arquivos de um template gerável.

O plano exige nome e tipo, stack de cada papel selecionado e layout para fullstack. Sem opções, usa `none` para banco/cache/containers/Kubernetes/monitoria e `env` para segredos. O plano não acessa rede, não verifica runtimes e não cria diretórios; lê receitas e verifica destinos/ancestrais existentes para rejeitar conflitos.

Nomes usam até 63 caracteres: letra minúscula inicial, a-z, números e hífen, sem hífen final. Destinos relativos são resolvidos pelo diretório de execução, inclusive quando vêm de uma receita. O destino precisa estar ausente; `..`, destinos sobrepostos, symlinks de destino e ancestrais inválidos são rejeitados.

## Dados, cache e infraestrutura

```sh
cargo run -p rust-boilerplate -- plan api-completa \
  --type backend --backend express-ts \
  --database postgres --database-location local \
  --cache redis --cache-location local \
  --containers compose --kubernetes manifests \
  --image registry.example.com/api:0.1.0 --namespace api-dev \
  --monitoring prometheus-grafana \
  --secrets vault --vault-location local --json
```

O endereço de imagem acima é um exemplo a substituir antes de uma futura implantação. Nesta etapa, referências aceitam tags explícitas diferentes de `latest`; digests não estão implementados. Sem Dockerfile planejado, Kubernetes exige referência de imagem para cada aplicação (`--image` ou `--image-web`/`--image-api`).

| Opção | Valores |
| --- | --- |
| `--database` | `none`, `postgres`, `mongodb` |
| `--cache` | `none`, `redis` |
| `--database-location`, `--cache-location`, `--vault-location` | `local`, `external` quando a integração correspondente estiver ativa |
| `--containers` | `none`, `compose` |
| `--kubernetes` | `none`, `manifests` |
| `--monitoring` | `none`, `prometheus-grafana` |
| `--secrets` | `env`, `vault` |

`local` significa que a infraestrutura fará parte dos arquivos planejados e exige Compose ou manifests. Uma instância instalada/administrada separadamente usa `external`. `none` não cria pastas de integração nem recursos de infraestrutura. Frontend puro rejeita banco/cache/Vault. Monitoria de frontend estático exige o servidor de container planejado; em fullstack, a instrumentação HTTP proposta inicialmente pertence à API.

O plano não sobe Docker nem aplica Kubernetes. Os exemplos de geração usam API em 3000, React/vanilla em 5173 e Next em 3001, com cliente HTTP e CORS coerentes. Frameworks e lockfiles das bases já estão fixados. Drivers e manifests avançados ainda não são gerados.

## Receitas TOML e flags

Há [uma receita mínima](recipes/backend.toml) e [uma receita fullstack](recipes/fullstack.toml), sem credenciais:

```sh
cargo run -p rust-boilerplate -- plan \
  --config ferramentas/boilerplate/recipes/backend.toml --json

cargo run -p rust-boilerplate -- plan outro-nome \
  --config ferramentas/boilerplate/recipes/backend.toml \
  --backend rust-axum --database none --json
```

A receita exige `schema_version = 1` e `catalog_version = "1"`. Campos desconhecidos e valores inválidos são erros. Flags explícitas prevalecem; `--database none`, `--cache none` e `--secrets env` também limpam a localização herdada da opção desativada, salvo localização explícita na mesma chamada (que será validada).

`plan` sempre é não interativo; `--non-interactive` é aceito para scripts. Não há leitura de stdin ou espera por respostas. `plan` não aceita `--install`: não tem efeitos de instalação. `new` grava `boilerplate.toml` sem credenciais no projeto. A saída de new em JSON exige `--non-interactive` e `--install yes|no`; progresso dos processos externos vai para stderr.

## Diagnóstico de ferramentas

```sh
cargo run -p rust-boilerplate -- doctor --template express-ts
cargo run -p rust-boilerplate -- doctor --template rust-axum --json
cargo run -p rust-boilerplate -- doctor --template express-ts \
  --config ferramentas/boilerplate/recipes/fullstack.toml
cargo run -p rust-boilerplate -- doctor --template express-ts --git --github
```

Doctor executa somente consultas de versão dos executáveis previstos para a stack: Node/npm, Rust/Cargo ou nenhum runtime obrigatório para vanilla. Com receita, acrescenta Docker/Compose e kubectl somente quando selecionados. `--template` precisa fazer parte da receita. Em fullstack, execute doctor para cada template se quiser conferir os dois runtimes.

Node é verificado contra a versão solicitada (padrão 24 ou valor da receita), usando uma versão já instalada no nvm quando necessário. Outros requisitos mostram presença/versão; não há certificação de todas as combinações de runtime. Docker é consultado com `--version` e `compose version`; kubectl com `version --client=true`. Não é verificado acesso ao daemon, cluster, bancos, Vault ou serviços de monitoria. Para planejar, nenhuma dessas ferramentas é obrigatória.

Códigos: 0 sucesso; 1 falha operacional ou ferramenta ausente/falhando; 2 argumentos/configuração inválidos. Em JSON, doctor mantém os resultados em stdout e diagnósticos em stderr. `new` cancela com código 130 quando o usuário recusa a confirmação ou a entrada interativa encerra. Falhas de instalação/Git/GitHub preservam os projetos já publicados.

## Desenvolvimento e extensão

```sh
cargo fmt --all -- --check
cargo test -p rust-boilerplate --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Veja [como adicionar uma linguagem](docs/adicionar-stack.md), a [arquitetura atual](docs/arquitetura-do-pacote.md) e a [especificação completa](docs/README.md). As próximas etapas implementam os adapters de dados/cache, Kubernetes, monitoria e Vault. A geração de arquivos dessas opções continua indisponível até suas validações específicas. Dockerfiles/Compose foram gerados e o Compose fullstack passou em `docker compose config --quiet`; a construção/execução das imagens ainda não foi validada nesta revisão.
