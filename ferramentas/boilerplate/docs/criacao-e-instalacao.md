# Criação, destino e instalação

[Guia de uso](../README.md) · [Git/GitHub](git-e-github.md)

## O que a CLI faz

`boilerplate new` cria o projeto no destino informado, sem mesclar ou sobrescrever diretórios existentes. O assistente pede o caminho; em automação use `--destination`, ou `--web-dir` e `--api-dir` para split. Se a opção de destino for omitida em automação, usa o nome do projeto relativo ao cwd. Não usa a localização do executável ou da receita como base.

Os arquivos são renderizados e validados antes de publicar. A publicação usa staging no mesmo filesystem e rename sem substituição no WSL/Linux. Se um destino surgir entre a verificação e a publicação, a operação falha sem sobrescrevê-lo. Em split, os dois projetos são renderizados primeiro; se a segunda publicação falhar, o primeiro fica preservado e o erro informa o que já foi publicado. Diretórios pais ausentes podem ser criados.

A etapa seguinte inicializa Git local, oferece instalação de dependências e, por último, trata GitHub/push. Um erro depois da publicação não remove os fontes, lockfiles ou dados de um projeto gerado.

## Stacks e módulos

Bases geráveis: vanilla HTML/CSS/JS, React/Vite/TS, Next/TS, Express/TS, Nest/TS e Rust/Axum. Fontes JS/TS usam import/export. `--module-system esm` é padrão; Express também aceita `commonjs`, com `package.json` e saída TypeScript coerentes. As outras bases Node rejeitam CommonJS nesta revisão.

As dependências Node usam versões exatas e package-lock distribuído com o template; a instalação usa `npm ci`. Rust inclui Cargo.lock e usa `cargo fetch --locked`. Frameworks e CLIs são dependências do projeto, não instalações globais.

## Node e nvm

Padrão Node 24. `--node-version` aceita um major ou versão numérica, por exemplo `24` ou `24.15.0`; aliases e texto de shell são rejeitados. A base exige Node 22.12 ou posterior; um major `22` é resolvido pelo nvm e a versão concreta também é verificada. O template grava `.nvmrc`, `engines.node` e `.npmrc` com engine-strict/save-exact.

Antes de instalar dependências, a CLI tenta o Node/npm no PATH. Se não corresponderem à seleção, tenta encontrar a versão já instalada no nvm. O diretório do nvm vem de `NVM_DIR`, da configuração XDG existente ou de `$HOME/.nvm`. `nvm` é função de shell, portanto não é tratado como executável global.

Se não encontrar versão compatível, o assistente pergunta se pode preparar nvm/Node. Em CI, isso só é permitido com `--setup-node yes`; `no` impede a preparação. Se não há nvm, a CLI clona o repositório oficial na versão v0.40.8 sem sobrescrever uma instalação existente e sem editar .bashrc/.zshrc. Depois usa nvm para instalar a versão solicitada. É necessário Git e acesso à rede; o procedimento não usa sudo.

Os comandos usam scripts fixos e argumentos separados para versão/caminhos. O PATH é ajustado somente nos subprocessos da instalação; uma CLI não consegue alterar o shell que a chamou. Para usar nvm na próxima sessão, siga o README gerado e carregue seu nvm normalmente. Veja as [instruções oficiais do nvm](https://github.com/nvm-sh/nvm#installing-and-updating) e a [semântica de módulos Node](https://nodejs.org/api/packages.html#type).

## Instalar ou pular

- Em TTY, após criar os arquivos, perguntar se deve instalar as dependências; Enter aceita a instalação.
- Em automação, `--install yes` ou `--install no` é obrigatório. A ausência retorna erro antes de escrever.
- `--install no` ainda gera fontes, lockfiles e guias; não testa nem instala runtimes.
- Vanilla não possui gerenciador de pacotes; o README explica como servir seus módulos por HTTP.
- Uma falha de npm/Cargo mantém o projeto e retorna código 1. Recupere rodando `npm ci` ou `cargo fetch --locked` no diretório da aplicação após corrigir runtime/rede.

## Fullstack

```sh
# Monorepo: dois projetos e um Git na raiz.
boilerplate new plataforma --type fullstack --layout monorepo \
  --frontend react-vite-ts --backend express-ts \
  --destination ./plataforma --containers compose \
  --install yes --setup-node no --github no --non-interactive

# Duas raízes, com Git e receita independentes.
boilerplate new plataforma --type fullstack --layout split \
  --frontend next-ts --backend rust-axum \
  --web-dir ./plataforma-web --api-dir ./plataforma-api \
  --install no --github no --non-interactive
```

Instale a CLI conforme o [README](../README.md), ou substitua `boilerplate` por `cargo run -p rust-boilerplate --` na raiz deste repositório.

API local em 3000; frontend React/vanilla em 5173 ou Next em 3001. O browser recebe URL pública local, nunca o nome interno do serviço Compose. Cada aplicação tem seus scripts e validações; o monorepo não instala gerenciadores de workspace ou um runtime Node no backend Rust.

## Ignore e containers

Sempre gerar `.gitignore` para dependências, builds, arquivos .env privados e dados locais; manter `.env.example` e lockfiles versionáveis. Ao escolher `--containers compose`, gerar também Dockerfile/.dockerignore por aplicação e Compose na raiz do repositório. Em split, cada raiz tem seu Compose.

A CLI verifica se Docker responde a `--version` e informa quando estiver ausente, mas não instala Docker, não consulta o daemon e não executa Compose. Instale/configure Docker no WSL se quiser executar os arquivos. Para revisão: `docker compose --env-file infra/.env config`; para iniciar manualmente: `docker compose --env-file infra/.env up --build`. O Compose fullstack passou em `docker compose --env-file infra/.env config --quiet`; a execução das imagens ainda não foi validada nesta entrega.

Dados/cache/Kubernetes/monitoria/Vault são geráveis, com passos em docs/OPERACAO.md. A [receita fullstack](../recipes/fullstack.toml) funciona em plan ou new. Veja [integrações](integracoes.md) para escolher serviços locais/externos e configurar os endpoints.
