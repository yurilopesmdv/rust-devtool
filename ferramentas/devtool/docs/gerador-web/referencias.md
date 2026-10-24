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
- [Vault Agent Injector](https://developer.hashicorp.com/vault/docs/deploy/kubernetes/injector): referência para entrega de segredos em arquivos no Kubernetes. O adapter da aplicação precisa consumir esses arquivos.

## Referência local de experiência no terminal

Foram lidos os exemplos `git/change-branch`, `uteis/README.md` e `uteis/dicas/dicas.sh` do repositório local `tools`. Os padrões observados foram: comando no PATH, README por ferramenta, validação de dependências e seleção interativa quando aplicável. Alguns exemplos exigem Bash; não se deve presumir compatibilidade POSIX apenas por serem scripts.

A proposta para este repositório preserva essa experiência com núcleo Rust e scripts auxiliares que declaram corretamente o interpretador. Não importa os aliases ou as configurações pessoais do outro repositório.

## O que é decisão deste projeto

O catálogo de seis stacks, a arquitetura em camadas, os layouts de repositório, o formato da receita, as opções `none`, o contrato de arquivos de segredos, os limites de execução e a sequência de implementação são decisões/propostas locais. As fontes explicam os mecanismos utilizados; não validam automaticamente os templates ou suas combinações. Essa validação faz parte das próximas entregas.
