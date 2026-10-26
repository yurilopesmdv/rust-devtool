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

Nesta microentrega, integrações não declaradas no catálogo são rejeitadas antes da escrita. A expansão PostgreSQL fica na tarefa seguinte; MongoDB/Redis/Vault/monitoria não são anunciados para Go. Mobile e Java permanecem contratos futuros.

Para validar a aplicação gerada em diretório temporário:

```sh
python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal --stacks go-http
```

Esse runner executa go mod download/verify, gofmt, testes com race, vet e build. O build temporário desativa somente VCS stamping (`-buildvcs=false`), pois os fixtures não possuem repositório Git próprio. Os comandos normais da aplicação mantêm o comportamento padrão do Go.
