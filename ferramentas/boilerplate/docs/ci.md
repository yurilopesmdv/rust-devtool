# CI opcional no projeto gerado

As oito stacks web oferecem `--ci github-actions`: Vanilla, React/Vite, Next.js, Express, NestJS, Rust/Axum, Python/FastAPI e Go. CI e criação de repositório no GitHub são escolhas independentes. O padrão é `none` e não gera arquivos de CI.

```sh
cargo run -p rust-boilerplate -- new minha-api \
  --type backend --backend express-ts --ci github-actions \
  --install no --git yes --github no --non-interactive
# Apenas planejar, sem criar arquivos:
cargo run -p rust-boilerplate -- plan meu-front \
  --type frontend --frontend react-vite-ts --ci github-actions
```

O assistente pergunta sobre CI para qualquer stack compatível. A receita registra `[infrastructure] ci = "github-actions"` ou `"none"`; receitas antigas sem o campo continuam desativadas. Escolher CI não instala gh nem cria recursos remotos.

A raiz recebe `.github/workflows/ci.yml` e `docs/CI.md`. Cada aplicação recebe `.ci/check.sh`. Execute dentro da aplicação:

```sh
sh .ci/check.sh
```

O workflow executa esse mesmo script. Prepare antes o runtime descrito no README; o script instala as dependências locais com locks, mas não instala runtimes globalmente.

| Gerenciador | Ambiente do workflow | Checks |
| --- | --- | --- |
| npm | Node da receita, padrão 24 | npm ci e npm run check: lint, formato, tipos quando aplicável, testes e build |
| uv | uv 0.13.5, Python 3.13 | uv sync --locked, Ruff lint/formato, pytest |
| Cargo | Rust stable, rustfmt e Clippy | formato, testes --locked, Clippy com warnings como erros |
| Go | Go 1.27.3 | download/verificação de módulos, gofmt, testes com race detector, vet e build |

No monorepo, há dois jobs independentes com diretórios `apps/web` e `apps/api`. No split, cada repositório recebe workflow próprio, script e receita com CI ativada. Um projeto isolado recebe um job. Os adapters são selecionados pelo gerenciador declarado no catálogo; uma nova linguagem deve implementar seu adapter de checks e declarar a capability `github-actions` após validação.

Os jobs usam ubuntu-24.04, actions fixadas por SHA, `contents: read`, checkout sem persistir credenciais e timeout. Não há publicação, deploy, push, provisionamento de serviços ou aplicação automática de migrations. Rust stable é um canal atualizado; os locks fixam dependências, não o compilador. Para exigir compilador exato, mantenha um rust-toolchain.toml e ajuste o workflow do projeto.

Banco, Redis, Vault e monitoria não são iniciados. Testes de persistência condicionados a URLs podem ser pulados. Para exigi-los, configure serviços descartáveis, URLs de teste e preparação/migrations explícitas antes dos checks, conforme o [guia de validação](validacao.md). Não use bancos pessoais ou de produção.

O workflow executa remotamente depois de enviado ao GitHub. Configure required checks e regras de branch após observar o primeiro run. Atualizações de actions/runtimes exigem revisão. A validação deste gerador executa os scripts reais das oito stacks e verifica os workflows dos projetos isolados e das 30 combinações fullstack com actionlint; isso não configura políticas do repositório gerado.
