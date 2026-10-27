# Go: API com net/http

A base go-http usa Go 1.27.3 no Linux/WSL. [Downloads oficiais e checksums](https://go.dev/dl/) permitem instalar a toolchain sem um CLI global de framework. gofmt acompanha Go e precisa estar no PATH antes da geração, pois os fontes são formatados antes da publicação. Use `GOTOOLCHAIN=local` para exigir a toolchain local, sem download automático de outra versão.

```sh
cargo run -p rust-boilerplate -- new minha-api-go \
  --type backend --backend go-http \
  --install no --git yes --github no --non-interactive
cd "$HOME/projects/minha-api-go"
go version
go mod download
go test -race ./...
go vet ./...
test -z "$(gofmt -l .)"
go run ./cmd/api
```

O assistente também oferece go-http como backend de fullstack, com vanilla/React/Next, tanto em monorepo como split. A API responde em 3000; FRONTEND_ORIGIN define a origem permitida. PORT aceita 1–65535; a configuração rejeita URLs públicas com credenciais. Variáveis vêm do ambiente do processo: .env.example não é carregado implicitamente. Exemplo: `PORT=3010 FRONTEND_ORIGIN=http://localhost:5173 go run ./cmd/api`.

cmd/api é a composição e lifecycle. internal/httpapi registra rotas e CORS. internal/modules/example contém controller HTTP e service de domínio. /health mede liveness; /ready usa uma função de disponibilidade injetada e dispensa recursos externos na base mínima. A aplicação trata INT/TERM com shutdown limitado; timeouts HTTP e de readiness são explícitos. Erros internos não são enviados ao cliente.

A primeira entrega usa somente biblioteca padrão: não gera driver, repository, configuração de banco ou go.sum vazio. go.mod é versionado; quando houver dependências, go.sum também deve ser versionado e validado com go mod verify. A versão mínima está declarada em go.mod; mantenha toolchain e CI alinhadas ao atualizar.

Integrações não declaradas no catálogo são rejeitadas antes da escrita. PostgreSQL, Compose e manifests Kubernetes são opcionais; MongoDB/Redis/Vault/monitoria não são anunciados para Go. Mobile e Java permanecem contratos futuros.

Para validar a aplicação gerada em diretório temporário:

```sh
python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal --stacks go-http
```

Esse runner executa go mod download/verify, gofmt, testes com race, vet e build. O build temporário desativa somente VCS stamping (`-buildvcs=false`), pois os fixtures não possuem repositório Git próprio. Os comandos normais da aplicação mantêm o comportamento padrão do Go.

## PostgreSQL opcional

```sh
cargo run -p rust-boilerplate -- new minha-api-go-db \
  --type backend --backend go-http --database postgres --database-location local \
  --containers compose --kubernetes manifests \
  --install no --git yes --github no --non-interactive
```

O gerador não inicia serviços. Configure infra/.env a partir do exemplo e inicie o Compose explicitamente quando desejar; os comandos estão em docs/OPERACAO.md do projeto. Para uso local, exporte DATABASE_URL com URI própria (senha com percent-encoding) e execute `go run ./cmd/prepare` antes de `go run ./cmd/api`. Em Compose, a preparação é `docker compose --env-file infra/.env run --rm api /app/prepare`; em Kubernetes há um Job explícito. Não faça preparação em cada startup HTTP.

O pool pgx v5.11.0 é criado/pingado no startup, limitado a dez conexões e fechado depois do shutdown HTTP. Cada readiness tem limite de dois segundos. Queries e upsert parametrizados pertencem ao repository; infraestrutura somente cria pool e prepara schema. Services dependem de interface, controllers tratam HTTP e validam 1–200 pontos de código Unicode. URLs e erros internos não aparecem nas respostas HTTP.

`go test -race ./...` habilita persistência quando DATABASE_URL está definida: use somente banco descartável, pois prepara a tabela e escreve/restaura o exemplo. Sem URL, o teste é pulado explicitamente. Ele verifica HTTP, Unicode e readiness 503/liveness 200 após fechar o pool. A base sem banco continua sem driver/repository/go.sum.

Os snapshots `integrations/go/postgres.mod` e `postgres.sum` distribuem o grafo fixado. Para atualizar, gere um projeto PostgreSQL temporário, altere a versão direta, execute `go mod tidy`, `go mod download`, `go mod verify` e os testes com banco real; copie manifesto/somas após revisar o diff, substituindo apenas o módulo raiz por `{{PROJECT_NAME}}` no snapshot .mod. Não normalize integridades/versões. O runner recusa mutação de go.mod/go.sum durante validação. A manutenção Go pode ganhar um updater dedicado após novos adapters; hoje há um único grafo opcional.

Compose/Kubernetes passaram verificação estrutural. O Dockerfile compila API e prepare somente quando necessários, usa usuário 10001 e CA certificates; a imagem mínima passou no smoke real em runner descartável. A variante Docker com PostgreSQL/prepare ainda não foi executada; seus fontes e configuração foram verificados pelos testes e pelo build nativo.
