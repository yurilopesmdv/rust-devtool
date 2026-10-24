# Integrações web disponíveis

[Guia de uso](../README.md) · [Validação](validacao.md)

O assistente e as flags oferecem as mesmas escolhas. Nenhum banco, cache, runtime de infraestrutura ou recurso de nuvem é instalado/iniciado por `new`. O comando gera fontes/configuração e pode instalar as bibliotecas da aplicação quando autorizado.

## Matriz

| Aplicação | PostgreSQL/MongoDB | Redis | Prometheus | Vault |
| --- | --- | --- | --- | --- |
| Express/TS | pg ou driver MongoDB | node-redis | HTTP + processo | Arquivos de configuração |
| Nest/TS | Mesmos drivers, mantendo controllers/providers | node-redis | HTTP + processo | Arquivos de configuração |
| Rust/Axum | SQLx ou driver MongoDB | redis-rs | HTTP | Arquivos de configuração |
| Next standalone | Na API separada | Na API separada | Processo Node em /metrics | Na API separada |
| React/Vite e vanilla | Na API separada | Na API separada | nginx + exporter em containers | Na API separada |

Fullstack monitora a API; frontend estático isolado mede tráfego/conexões do nginx. Não são gerados painéis de latência/status quando o servidor não fornece essas métricas. Dados/cache/Vault não entram no bundle do browser.

## Gerar uma base completa

Na raiz deste repositório:

```sh
cargo run -p rust-boilerplate -- new plataforma \
  --type fullstack --layout monorepo \
  --frontend react-vite-ts --backend express-ts \
  --destination "$HOME/projects/plataforma" \
  --database postgres --database-location local \
  --cache redis --cache-location local \
  --containers compose --kubernetes manifests \
  --monitoring prometheus-grafana \
  --secrets vault --vault-location local \
  --install yes --setup-node no --github no --non-interactive
```

Troque `express-ts` por `nest-ts` ou `rust-axum`; `mongodb` substitui `postgres`. `--install no` permite apenas gerar. Para dois repositórios, use `--layout split --web-dir ... --api-dir ...`, omitindo `--destination`.

O README e `docs/OPERACAO.md` gerados explicam os passos exatos daquela seleção. É necessário preencher valores próprios em arquivos ignorados antes de executar a infraestrutura. O bootstrap de Vault é explícito: não existe root token pronto no boilerplate.

## Banco/cache e a escolha none

`--database none` não gera driver, conexão, repository, migration nem serviço de banco. `--cache none` não gera Redis ou variáveis de cache. O mesmo vale para monitoria e Vault desativados. As pastas comuns da arquitetura continuam presentes quando fazem parte da base.

Para dados relacionais/transações, PostgreSQL é uma base prática. MongoDB oferece um modelo de documentos; escolha considerando consultas e invariantes dos dados. O wizard não exige ORM: os drivers por stack são fixados, e as regras chamam repositories.

`local` gera os recursos de desenvolvimento no Compose/Kubernetes selecionado. `external` gera apenas o cliente/contrato e exemplos de configuração: não cria uma segunda instância. URLs com credenciais ficam em .env ignorado ou arquivos de segredos, não na receita TOML.

O exemplo persistente expõe GET e POST em `/api/v1/example`; a gravação valida `message` entre 1 e 200 caracteres. A preparação é explícita e idempotente para a tabela/coleção de exemplo:

- Node: na API, `npm run build && npm run db:prepare`.
- Rust: na API, `cargo run --locked --example prepare`.
- Kubernetes: Job em `infra/kubernetes/jobs/prepare.yaml`, aplicado manualmente.

Readiness consulta o banco obrigatório. Redis usa TTL de 30s e chave com prefixo do projeto; indisponibilidade do cache permite seguir pela origem. Isso é cache-aside com consistência eventual, não uma transação distribuída.

## Endereços e imagens

`--api-url https://api.example` define a URL pública usada pelo browser; `--frontend-origin https://web.example` configura CORS. As URLs aceitam origem HTTP(S), sem credenciais/caminho. Vite/Next aplicam configurações públicas no build, portanto reconstrua a imagem depois de alterá-las.

Em Kubernetes sem Compose, forneça `--image` ou `--image-web`/`--image-api`. Referências aceitam tag explícita diferente de latest ou digest sha256 de 64 dígitos. O cluster/registry e a imagem construída são pré-requisitos externos. O fluxo local usa port-forward, sem instalar ingress controller ou assumir um domínio.

## Compose, Kubernetes e segredos

Os arquivos Compose são JSON válido, aceito como YAML pelo Docker Compose; os manifests e kustomizations seguem a mesma abordagem determinística. Na raiz do projeto gerado:

```sh
cp infra/.env.example infra/.env
# Edite os placeholders e credenciais antes de continuar.
docker compose --env-file infra/.env config --quiet
# Após o bootstrap necessário, iniciar é uma ação manual:
docker compose --env-file infra/.env up --build -d
```

O arquivo `infra/.env` usa endereços internos. O `.env.example` da aplicação é voltado ao runtime local. Não misture os dois. O gerador protege ambos e os arquivos de bootstrap de Vault em .gitignore/.dockerignore.

Kubernetes usa Kustomize, Deployments/Services para aplicações, StatefulSets/PVCs para dados locais e referências a Secrets criados pelo operador. Não há credenciais reais em YAML. O overlay dev de monitoria usa armazenamento efêmero; bancos locais precisam de StorageClass. Ajuste capacidade, persistência, TLS e backup conforme o ambiente.

Vault usa AppRole no Agent Compose e autenticação Kubernetes em initContainer no cluster, com policy limitada ao caminho do projeto. Não requer Injector/Operator. `--vault-address https://vault.example:8200` registra um endereço público não secreto para Agents externos. O leitor suporta DATABASE_URL_FILE/REDIS_URL_FILE; exemplo sem banco/cache usa APP_SECRET_FILE apenas como demonstração de configuração, sem implementar autenticação HTTP.

Segredos são lidos no startup; rotação exige reiniciar a aplicação. O guia gerado detalha inicialização/unseal, policy, role, entrega dos arquivos, operações e diagnóstico.
