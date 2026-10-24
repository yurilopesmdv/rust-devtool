# Lint e formatação

[Guia de uso](../README.md) · [Validação](validacao.md)

Os projetos distribuem configuração de qualidade, ferramentas locais fixadas e `.editorconfig` (LF/UTF-8, espaços). O gerador formata os fontes JS/TS compostos antes de salvar, inclusive as integrações, sem instalar Node para essa etapa. Fontes Rust passam por rustfmt; Python distribui fontes/fragmentos formatados e suas combinações passam pelo Ruff na validação do catálogo.

| Linguagem | Lint | Formatação | Indentação |
| --- | --- | --- | --- |
| JavaScript/TypeScript/JSX/TSX | ESLint + typescript-eslint | dprint TypeScript embutido e CLI local | 2 espaços |
| Python | Ruff, incluindo imports | Ruff formatter | 4 espaços |
| Rust | Clippy com warnings como erro | rustfmt | 4 espaços |

HTML/CSS dos templates também estão indentados. Os checks de lint da tabela são para as linguagens de programação; não equivalem a validação semântica completa de HTML, acessibilidade ou CSS.

## Node e vanilla

Na pasta de cada aplicação:

```sh
npm ci
npm run lint
npm run format:check
npm run format
npm run check
```

`lint` não altera arquivos; `format:check` verifica, e `format` aplica a formatação. `check` reúne lint, formatação, typecheck, testes e build. Em vanilla, typecheck/test/build verificam a sintaxe dos módulos; não há bundler ou testes de navegador simulados. Node/npm são necessários para as ferramentas de desenvolvimento, enquanto o site pode ser servido por qualquer servidor estático.

ESLint usa configuração flat ESM em eslint.config.mjs, inclusive quando a saída de Express é CommonJS. dprint.json usa o plugin WASM instalado em node_modules, com a mesma versão do formatador embutido no gerador; não baixa plugin remoto durante a formatação. Não use npx com versão flutuante para substituir esses comandos.

TypeScript está fixado em 6.0.3: o parser typescript-eslint 8.72.0 anuncia suporte até versões inferiores a 6.1. A versão 7 não é forçada com legacy-peer-deps. Atualize compilador/parser em conjunto, regenere os locks base e superset e valide builds/lint antes de mudar essa combinação.

## Python

```sh
uv sync --locked
uv run --locked ruff check .
uv run --locked ruff format --check .
uv run --locked ruff format .
uv run --locked pytest
```

As regras estão no pyproject.toml, com imports ordenados, checagens de erros, atualização de sintaxe e práticas comuns. A exceção B008 permite Depends nas assinaturas de FastAPI. Não use pip global.

## Rust

```sh
rustup component add rustfmt clippy
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Para aplicar formatação, use `cargo fmt`. Gerar a stack Rust requer rustfmt no PATH, mesmo com --install no; se estiver ausente, a geração falha antes de publicar fontes. JS/TS e Python não precisam de formatadores globais para gerar arquivos.

## Evoluir os templates

Formatação não deve reescrever contratos de integração silenciosamente. Depois de alterar templates/fragmentos/adapters, rode os testes do pacote e `scripts/validate-generated.py`; use --minimal para as bases e --with-services com bancos de teste dedicados para persistência real. O script verifica formatação sem corrigi-la para esconder problemas.
