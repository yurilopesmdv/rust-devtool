# Fontes e decisões técnicas

[Índice](README.md). Fontes oficiais consultadas em 24/10/2026. Links podem evoluir; as versões do catálogo serão fixadas durante a implementação, com validação das combinações.

## Stacks

- [Vite: Getting Started](https://vite.dev/guide/): base para o frontend React/Vite; o CLI oficial serve de referência para os templates mantidos.
- [Next: create-next-app](https://nextjs.org/docs/app/api-reference/cli/create-next-app): opções de criação, incluindo separação da instalação de pacotes com `--skip-install`.
- [NestJS](https://docs.nestjs.com/): referência para preservar módulos, controllers e providers ao aplicar as camadas padronizadas.
- [Express: application generator](https://expressjs.com/en/starter/generator/): referência de scaffolding; o template API TypeScript deste projeto será próprio e não será apresentado como saída idêntica ao gerador oficial.
- [Axum](https://docs.rs/axum/latest/axum/): routing, estado e integração com Tokio/Tower para o backend Rust.

## Infraestrutura

- [Docker Compose profiles](https://docs.docker.com/compose/how-tos/profiles/): ativação seletiva de serviços já presentes na configuração. Decisão local: opções desativadas no gerador não devem aparecer no arquivo, mesmo ocultas por profile.
- [Kubernetes/Kustomize](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/kustomization/): base para compor manifests e overlays declarativos.
- [Kubernetes Secrets](https://kubernetes.io/docs/concepts/configuration/secret/): requisitos de tratamento de valores secretos; base64 não substitui proteção de segredos.
- [Prometheus: primeiros passos](https://prometheus.io/docs/introduction/first_steps/): referência de coleta e validação de targets.
- [Grafana provisioning](https://grafana.com/docs/grafana/latest/administration/provisioning/): configuração de datasource e dashboards por arquivos versionados.
- [Vault: autenticação Kubernetes](https://developer.hashicorp.com/vault/docs/auth/kubernetes): autenticação do Agent initContainer, roles, ServiceAccounts e TokenReview.

## Referência local de experiência no terminal

Foram lidos os exemplos `git/change-branch`, `uteis/README.md` e `uteis/dicas/dicas.sh` do repositório local `tools`. Os padrões observados foram: comando no PATH, README por ferramenta, validação de dependências e seleção interativa quando aplicável. Alguns exemplos exigem Bash; não se deve presumir compatibilidade POSIX apenas por serem scripts.

A proposta para este repositório preserva essa experiência com núcleo Rust e scripts auxiliares que declaram corretamente o interpretador. Não importa os aliases ou as configurações pessoais do outro repositório.

## O que é decisão deste projeto

O catálogo de seis stacks, a arquitetura em camadas, os layouts de repositório, o formato da receita, as opções `none`, o contrato de arquivos de segredos, os limites de execução e a sequência de implementação são decisões/propostas locais. As fontes explicam os mecanismos utilizados; não validam automaticamente os templates ou suas combinações. Essa validação faz parte das próximas entregas.

## Criação, Node e GitHub

- [nvm: instalação e atualização](https://github.com/nvm-sh/nvm#installing-and-updating): função de shell, versões e carregamento; o gerador usa clone versionado sem alterar perfis.
- [Node: packages/type](https://nodejs.org/api/packages.html#type): interpretação ESM/CommonJS por package.json.
- [gh repo create](https://cli.github.com/manual/gh_repo_create): criação privada a partir do repositório local e push explícito.
- [rustix renameat_with](https://docs.rs/rustix/latest/rustix/fs/fn.renameat_with.html): publicação Linux com NOREPLACE, sem substituir o destino.

## Drivers e validação

- [node-postgres](https://node-postgres.com/features/connecting): conexão/pool PostgreSQL.
- [MongoDB Node driver](https://www.mongodb.com/docs/drivers/node/current/get-started/): cliente MongoDB.
- [node-redis](https://redis.io/docs/latest/develop/clients/nodejs/): cliente de cache.
- [Prometheus client Node](https://github.com/prometheus/client_js): registry e métricas HTTP/processo.
- [SQLx](https://docs.rs/sqlx/0.8.6/sqlx/), [MongoDB Rust](https://docs.rs/mongodb/3.9.1/mongodb/) e [redis-rs](https://docs.rs/redis/0.32.7/redis/): adapters Rust.
- [Entrypoint Vault 1.20.2](https://raw.githubusercontent.com/hashicorp/vault/v1.20.2/.release/docker/docker-entrypoint.sh): comportamento de usuário, volume /vault/file e configuração do container.
