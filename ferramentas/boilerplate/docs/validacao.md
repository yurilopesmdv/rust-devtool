# Validação e reprodução dos testes

[Guia de uso](../README.md) · [Integrações](integracoes.md)

## Suíte do repositório

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Os testes incluem uma matriz de 72 combinações de backend/dados/cache/monitoria/Vault, verificando ausência de módulos desativados, lockfiles, serviços locais/externos, referências a segredos, layouts e URLs. Não iniciam bancos, containers ou clusters.

## Compilar as aplicações geradas

O script opcional requer Python 3.8+, Node 24 ativo e Rust/Cargo. Cria projetos em diretório temporário, instala dependências locais e executa typecheck/build/test. Docker CLI e kubectl, quando presentes, validam Compose e Kustomize sem daemon/cluster.

```sh
nvm use 24
python3 ferramentas/boilerplate/scripts/validate-generated.py
```

Ele não instala ferramentas globais, não inicia serviços e não publica no GitHub. `--keep` preserva os fontes para revisão. `--offline` exige caches preenchidos; `npm_config_cache` e `CARGO_TARGET_DIR` podem indicar caches explícitos para reutilizar downloads/compilações.

## Persistência real, com serviços de teste

Use bancos/cache dedicados, pois a suíte prepara a tabela/coleção `examples` e grava a entrada de demonstração. Configure POSTGRES_TEST_URL, MONGODB_TEST_URL e REDIS_TEST_URL no ambiente sem versionar ou expor credenciais. Depois:

```sh
python3 ferramentas/boilerplate/scripts/validate-generated.py --with-services --keep
```

Use `--commonjs-only` para validar especificamente Express/CommonJS com os dois bancos. O script não provisiona esses serviços. Ele testa Express/Nest/Rust com ambos os bancos, faz preparação explícita, executa os testes HTTP de leitura/escrita/validação/métricas das APIs Node e os testes de persistência Rust. Sem as variáveis, os testes dependentes de banco são explicitamente pulados; não se deve confundir isso com validação de persistência.

## Evidência desta entrega

Ambiente de validação: WSL/Ubuntu 20.04, Node 24.15.0 e Rust 1.99.1.

- Quatro combinações Node ESM (Express/Nest × PostgreSQL/MongoDB) e duas Express/CommonJS: npm ci, typecheck, build e testes, incluindo execução com serviços autenticados.
- Duas combinações Rust: Cargo.lock, testes, Clippy e testes de persistência reais com --ignored. Casos cache sem banco e Vault sem dados também foram compilados/testados.
- PostgreSQL 17.6 e Redis 7.4.5 compilados somente em /tmp; MongoDB 7.0.14 compatível com o host para o teste de protocolo/CRUD. A imagem gerada de MongoDB é 8.0.13 e não foi executada neste ambiente.
- Vault 1.20.2 Linux oficial, com SHA256 verificado: Agent real autenticou por AppRole e renderizou os arquivos de credenciais a partir da policy gerada. O teste usou servidor isolado e dados fictícios.
- Promtool 3.5.0 oficial validou configurações Prometheus e as métricas HTTP de aplicações Node/Rust em processos locais.
- Compose/Kustomize e manifests foram renderizados e conferidos contra o OpenAPI oficial Kubernetes 1.34. As tags de imagens dos serviços foram verificadas no registry.

O daemon Docker não estava acessível e não havia cluster de teste. Por isso não foram executadas imagens Docker, deploys, autenticação Vault/Kubernetes ou interface Grafana em container. As configurações são geradas e validadas, mas essa etapa operacional precisa ser confirmada no ambiente de destino. Nenhum serviço pessoal foi usado; processos temporários foram encerrados.
