# Fluxo e comandos propostos

[Índice](README.md). `templates list/show`, `plan`, `doctor` e `new` já existem. O assistente atual cobre bases, destinos, módulos, Node, Compose, instalação e Git/GitHub. Dados/cache/Kubernetes/monitoria/Vault também são oferecidos para os tipos compatíveis. Consulte os exemplos atuais no [guia de uso](../README.md).

## Fluxo final especificado

`boilerplate new` abre o fluxo no terminal. O assistente completa escolhas ausentes e confirma o resumo antes de escrever. Para alterar opções, use flags/receita ou cancele antes da confirmação.

1. Tipo: frontend, backend ou fullstack.
2. Nome e destino: um diretório novo; fullstack permite monorepo ou dois diretórios para repositórios separados.
3. Frontend, quando aplicável: vanilla JS, React/Vite/TS ou Next.js/TS.
4. Backend, quando aplicável: Express/TS, NestJS/TS ou Rust/Axum. Exibir a arquitetura padronizada; não oferecer arquiteturas sem templates validados.
5. Dados do backend: nenhum, PostgreSQL ou MongoDB. Se presente, escolher ambiente local de desenvolvimento ou serviço externo, com endereço configurável. Cache: nenhum ou Redis, com a mesma escolha de localização.
6. Containers: nenhum ou Dockerfile + Compose. Em fullstack, definir por aplicação e consolidar os serviços selecionados em um Compose no monorepo; no layout separado, gerar um Compose por diretório e instruções de conexão.
7. Kubernetes: nenhum ou manifests. Se selecionado, exigir imagem existente ou Dockerfile para construir a imagem. Perguntar namespace e referência das imagens; não pressupor registry ou cluster configurado.
8. Monitoria: nenhuma ou Prometheus + Grafana. Só oferecer instrumentação HTTP em aplicações com servidor; frontend estático usa métricas do servidor web quando houver container.
9. Segredos: variáveis de ambiente ou Vault. Vault permite instância externa; ambiente de laboratório local só é oferecido com Docker/Compose ou manifests Kubernetes.
10. Integração web/API: portas e URLs locais; CORS com origem explícita quando aplicável; destinos de desenvolvimento e Kubernetes separados. O resumo mostra placeholders de domínio que o usuário ainda precisa preencher.
11. Revisar plano: arquivos, dependências, serviços, opções omitidas, downloads previstos e pendências de ambiente. Confirmar a geração no assistente.
12. Gerar, validar os arquivos e perguntar se deve instalar dependências das aplicações. Se não, imprimir os comandos. Não iniciar aplicações ou infraestrutura.

Perguntas de banco/cache/Vault ficam fora do fluxo de frontend puro. Next.js pode ser o frontend de um backend separado: credenciais continuam no servidor; não são expostas ao bundle web. Uma modalidade Next-only fullstack integrada fica para uma evolução específica.

## Interface por flags

O executável é `boilerplate`, no pacote Cargo `rust-boilerplate`. `rust-devtool` continua independente. Abaixo há comandos atuais de consulta/plano e comandos de geração:

```sh
# Consultar catálogo e requisitos, sem criar um projeto.
boilerplate templates list
boilerplate templates show express-ts
boilerplate doctor --template express-ts

# Assistente atual para bases e Compose.
boilerplate new

# Backend mínimo com geração disponível.
boilerplate new minha-api --type backend --backend express-ts \
  --database none --cache none --containers none --kubernetes none \
  --monitoring none --secrets env --install no --non-interactive

# Plano completo antes de gerar.
boilerplate plan plataforma --type fullstack --layout monorepo \
  --frontend react-vite-ts --backend express-ts \
  --database postgres --database-location local \
  --cache redis --cache-location local --containers compose \
  --kubernetes manifests \
  --image-web registry.example.com/plataforma-web:0.1.0 \
  --image-api registry.example.com/plataforma-api:0.1.0 \
  --monitoring prometheus-grafana \
  --secrets vault --vault-location local --non-interactive

# O mesmo contrato de opções será aceito por new.
# Geração a partir de uma receita.
boilerplate new minha-api --config ferramentas/boilerplate/recipes/backend.toml --install no --non-interactive
```

`plan` nunca escreve arquivos de projeto, instala pacotes ou consulta daemons. Pode produzir o plano em JSON para automação. `new` imprime o mesmo plano antes de aplicá-lo.

## Receita e reprodução

A configuração será TOML, com `schema_version` e versão do catálogo. Campos desconhecidos são erros, não opções ignoradas. Flags explícitas prevalecem sobre a receita; o resultado é validado depois da mesclagem. No modo interativo, perguntar somente o que falta; no não interativo, usar defaults documentados para opções opcionais e rejeitar campos essenciais ausentes.

Defaults propostos: TypeScript nas stacks JS com framework; sem banco/cache/containers/Kubernetes/monitoria; segredos por ambiente; instalação desativada no modo não interativo. Para fullstack, a omissão de layout ou stacks essenciais exige uma resposta, não uma inferência.

A receita exportada guarda escolhas, versões e nomes, nunca tokens, senhas ou conteúdo de `.env`. Valores locais ficam em arquivo ignorado. Repetir a receita em diretório novo deve produzir a mesma estrutura e versões declaradas; reprodutibilidade de instalações exige lockfiles validados por template, sem depender de `latest`.

### Exemplo de receita mínima proposta

```toml
schema_version = 1
catalog_version = "1" # Versão ilustrativa; será publicada com os templates.

[project]
type = "backend"
name = "minha-api"
destination = "/home/seu-usuario/projects/minha-api"

[backend]
template = "express-ts"
database = "none"
cache = "none"

[infrastructure]
containers = "none"
kubernetes = "none"
monitoring = "none"
secrets = "env"
```

O schema 1 já é validado pelo núcleo; `plan` não aceita `--install` porque nunca executa instalação. Não colocar URLs com credenciais na receita. A escolha de instalar pertence à execução (`--install`), não à receita do projeto. `new` aceita `--node-version`, `--module-system`, `--setup-node`, `--git`, `--github` e `--push` conforme o [guia de uso](../README.md).

No layout `split`, a interface deve aceitar `--web-dir` e `--api-dir` para destinos independentes. As flags `--image-web`/`--image-api` definem referências de imagem; para projetos individuais, aceitar `--image`. Os endereços `registry.example.com` são placeholders e precisam ser substituídos antes de construir/publicar/aplicar recursos.

## Contrato de execução

- Linux e WSL primeiro, com chamadas utilizáveis em Bash, Zsh e POSIX sh. O binário não exige shell para gerar arquivos.
- Scripts opcionais usam `#!/bin/sh` se forem POSIX; scripts que exigirem Bash declaram essa dependência. Não executar um script Bash com `sh`.
- Sem argumentos suficientes e sem TTY: erro explicativo; nunca aguardar uma resposta invisível em CI.
- `--non-interactive` não inicia prompts; `new` exige `--install yes|no` nesse modo, sem implicar execução de serviços.
- Proposta de códigos de saída: 0 sucesso; 1 falha de geração/instalação; 2 entrada incompatível ou inválida; 130 cancelamento.
- Rejeitar destino existente na primeira versão, inclusive diretório vazio ou symlink. Validar nomes, destinos sobrepostos no layout separado e caminhos dos templates antes de escrever.
- Falha de renderização limpa apenas a área temporária desta execução. Falha de instalação preserva os fontes gerados e informa como repetir a instalação.
- Git local é padrão e pode ser desativado com `--git no`, conforme a decisão revisada. GitHub privado e push continuam escolhas explícitas e separadas. Layout separado cria duas raízes com repositórios independentes.
- Usar argumentos estruturados de processo em Rust, sem construir comandos com `sh -c` e dados fornecidos pelo usuário.
