# Python/FastAPI

[Guia de uso](../README.md) · [Arquitetura](revisao-de-arquitetura.md) · [Qualidade](qualidade.md)

O ID `python-fastapi` cria APIs Python 3.13 com FastAPI, uv, pytest, Pydantic, Ruff e uv.lock. Pode ser usado sozinho ou com qualquer frontend nos layouts monorepo/split. Os arquivos são gerados sem uv instalado; a instalação de dependências requer uv e é escolhida ao final.

```sh
cargo run -p rust-boilerplate -- new minha-api-python \
  --type backend --backend python-fastapi
```

O assistente sugere `$HOME/projects/minha-api-python`. Para automação sem instalar nem publicar:

```sh
cargo run -p rust-boilerplate -- new minha-api-python \
  --type backend --backend python-fastapi \
  --install no --github no --non-interactive
```

## Instalar e executar

Instale uv no WSL seguindo a [documentação oficial](https://docs.astral.sh/uv/getting-started/installation/), confira `uv --version` e entre na API criada. Não é necessário substituir o Python do sistema nem usar pip global.

```sh
uv sync --locked
cp .env.example .env
uv run --locked ruff check .
uv run --locked ruff format --check .
uv run --locked pytest
uv run --locked --env-file .env uvicorn app.main:create_app \
  --factory --host 0.0.0.0 --port 3000 --reload
```

uv seleciona/obtém Python 3.13 e mantém as dependências em `.venv`; as versões transitivas estão em uv.lock. `--locked` recusa um manifesto incompatível com o lock. Versione as alterações de pyproject.toml/uv.lock em conjunto. Os testes comuns não precisam iniciar serviços; testes de persistência só executam com URLs de teste configuradas. `.venv`, caches e .env são ignorados.

Endpoints: `/health`, `/ready`, `/api/v1/example`, `/docs` e `/openapi.json`. FastAPI valida o payload com Pydantic e retorna 422 quando inválido. `/metrics` só existe quando monitoria foi selecionada.

## Organização e integrações

A API usa `app/main.py` como fábrica de aplicação. Cada funcionalidade tem `router.py`, `schemas.py`, `service.py` e, com banco, `repository.py`. Routers usam Depends; services recebem dependências, e os repositories fazem consultas parametrizadas/driver. Configuração não executa consultas de domínio.

O lifespan abre os recursos selecionados e fecha-os no shutdown. Conexões pertencem a `app.state`, sem pool global compartilhado entre aplicações. PostgreSQL usa psycopg com pool async; MongoDB usa PyMongo async; Redis usa redis-py async, timeout e TTL de 30s com fallback para a origem. Não há ORM obrigatório.

Com PostgreSQL/MongoDB, preencha .env e prepare o exemplo explicitamente:

```sh
uv run --locked --env-file .env python -m app.infrastructure.prepare
uv run --locked --env-file .env pytest
```

A preparação é uma base demonstrativa idempotente; não é um sistema completo de migrações para schemas de produção. A readiness consulta o banco obrigatório; falhas retornam 503. Redis não determina readiness. A gravação invalida cache após persistir.

Docker/Compose, Kubernetes, Prometheus/Grafana e Vault usam as mesmas opções das outras APIs. O Dockerfile instala somente dependências de execução com uv sync --locked --no-dev. O Job Kubernetes chama o módulo Python de preparação. O runtime em container usa porta 3000; ajuste Dockerfile/manifests em conjunto se trocar esse contrato.

Vault fornece DATABASE_URL_FILE/REDIS_URL_FILE, com rejeição de configuração direta e arquivo simultâneos. Sem dados/cache, APP_SECRET_FILE é apenas demonstração de configuração. Bootstrap/autenticação/rotação continuam explícitos no `docs/OPERACAO.md` gerado; segredos são lidos no startup.
