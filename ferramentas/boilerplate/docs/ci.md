# CI opcional no projeto gerado

A primeira implementação atende Express/TypeScript. Demais stacks rejeitam a opção antes da escrita; cada uma terá adapter próprio após validação. CI e criação de repositório no GitHub são escolhas independentes.

```sh
cargo run -p rust-boilerplate -- new minha-api \
  --type backend --backend express-ts --ci github-actions \
  --install no --git yes --github no --non-interactive
# Para omitir completamente:
cargo run -p rust-boilerplate -- plan outra-api \
  --type backend --backend express-ts --ci none
```

No assistente Express, a pergunta de CI tem padrão não. A receita salva `[infrastructure] ci = "github-actions"` ou `"none"`; receitas antigas sem esse campo continuam desativadas. Não há instalação de gh ou alteração remota por escolher CI.

Quando ativada, a raiz recebe .github/workflows/ci.yml e docs/CI.md. O job usa ubuntu-24.04, Node da receita, actions fixadas por SHA, contents:read, checkout sem credenciais persistentes e timeout. Executa npm ci e npm run check (lint, formatação, typecheck, testes e build). Não publica pacotes/imagens, não cria ambientes, não faz push nem deploy e não recebe segredos configurados pelo gerador.

Em monorepo fullstack, working-directory é apps/api e a primeira versão verifica somente a API. Em split, somente o repositório da API recebe workflow; a receita do frontend registra CI none. O job não provisiona banco/cache/Vault: testes de persistência condicionais podem ser pulados sem URLs de teste. Antes de exigir cobertura de persistência, adicione serviços descartáveis e preparação explícita, seguindo o guia de validação. Não aponte os testes para um banco pessoal ou de produção.

Execute localmente na API `npm ci && npm run check` e valide o YAML com actionlint antes de publicar. O workflow só executará remotamente quando for enviado ao GitHub. Required checks e regras de branch precisam ser configurados separadamente após observar o primeiro run. A permissão somente de leitura do token não torna scripts de dependências confiáveis: revise dependências e não forneça segredos a contribuições não confiáveis.
