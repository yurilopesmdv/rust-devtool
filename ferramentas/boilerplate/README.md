# Boilerplate web — CLI Rust para WSL/Linux

Pacote `rust-boilerplate`, executável `boilerplate`. Este pacote é independente de `ferramentas/devtool`: utilitários de hash/JSON continuam lá; catálogo, planejamento e futuras integrações de projetos ficam aqui.

**Disponível agora:** catálogo, receitas TOML, validação, plano em texto/JSON e diagnóstico de ferramentas. **Ainda indisponível:** `new`, assistente, geração de fontes, instalação de dependências e arquivos de infraestrutura. As seis stacks estão com status `planned`; capacidades do catálogo descrevem intenções, não adapters já funcionais.

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

O plano não sobe Docker nem aplica Kubernetes. URLs, portas, CORS, versões exatas de frameworks/drivers e manifests finais serão adicionados com a geração dos templates.

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

`plan` sempre é não interativo; `--non-interactive` é aceito para scripts. Não há leitura de stdin ou espera por respostas. `--install` não é aceito nesta etapa. Exportação de receita TOML também fica para a etapa de geração; JSON é a saída estruturada atual.

## Diagnóstico de ferramentas

```sh
cargo run -p rust-boilerplate -- doctor --template express-ts
cargo run -p rust-boilerplate -- doctor --template rust-axum --json
cargo run -p rust-boilerplate -- doctor --template express-ts \
  --config ferramentas/boilerplate/recipes/fullstack.toml
```

Doctor executa somente consultas de versão dos executáveis previstos para a stack: Node/npm, Rust/Cargo ou nenhum runtime obrigatório para vanilla. Com receita, acrescenta Docker/Compose e kubectl somente quando selecionados. `--template` precisa fazer parte da receita. Em fullstack, execute doctor para cada template se quiser conferir os dois runtimes.

A versão mostrada é informativa: compatibilidade mínima de runtime/framework ainda não foi certificada. Docker é consultado com `--version` e `compose version`; kubectl com `version --client=true`. Não é verificado acesso ao daemon, cluster, bancos, Vault ou serviços de monitoria. Para planejar, nenhuma dessas ferramentas é obrigatória.

Códigos: 0 sucesso; 1 falha operacional ou ferramenta ausente/falhando; 2 argumentos/configuração inválidos. Em JSON, doctor mantém os resultados em stdout e diagnósticos em stderr. `new` ainda não existe e é rejeitado como comando desconhecido.

## Desenvolvimento e extensão

```sh
cargo fmt --all -- --check
cargo test -p rust-boilerplate --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Veja [como adicionar uma linguagem](docs/adicionar-stack.md), a [arquitetura atual](docs/arquitetura-do-pacote.md) e a [especificação completa](docs/README.md). O caminho seguinte é implementar renderização/escrita e assistente, começando pelas bases Express/TS e React/Vite. Geração, dependências e integrações terão testes próprios antes de serem anunciadas como disponíveis.
