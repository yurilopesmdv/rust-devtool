# Rust DevTool

Aprender Rust na prática, partindo de uma base em TypeScript e avançando até a construção de ferramentas de desenvolvimento para o terminal.

O repositório é um workspace Cargo com três espaços:

- [guia/](guia/README.md): roteiro, explicações e exemplos executáveis da linguagem.
- [ferramentas/boilerplate/](ferramentas/boilerplate/README.md): geração, instalação e planejamento de projetos web, separados dos utilitários.
- [ferramentas/](ferramentas/README.md): utilitários de desenvolvimento; a CLI existente está em [ferramentas/devtool/](ferramentas/devtool/README.md).

## Começar

Instale a toolchain stable de Rust com as [instruções oficiais](https://www.rust-lang.org/tools/install). O projeto usa a edição 2024; utilize uma versão stable atual que a suporte.

```sh
rustc --version
cargo --version
cargo run -p guia-rust --example fundamentos
cargo run -p rust-devtool -- help
```

Execute os comandos deste repositório a partir da raiz, salvo indicação diferente.
O compilador produz os binários em `target/`; o antigo executável avulso `ownership`, se presente localmente, não faz parte dos fontes.

## Percurso de aprendizado

Siga o [roteiro completo](guia/README.md): fundamentos → controle de fluxo → ownership → structs e enums → erros → coleções → traits e lifetimes → módulos e testes → arquivos e CLI → concorrência.

Cada pasta tem explicação em português, código executável, comparação com TypeScript e exercícios. Ao final, use o [projeto de consolidação](guia/projeto-final.md) para planejar sua primeira ferramenta. Consulte também a [ponte TypeScript → Rust](guia/referencias/typescript-para-rust.md).

## Criar e planejar projetos web

O pacote [boilerplate](ferramentas/boilerplate/README.md) cria as seis bases web, recebe nome/destino, instala dependências quando solicitado e integra Git/GitHub opcionalmente. Executa em WSL/Linux, com ESM e Node 24 por padrão nas stacks Node. Banco/cache, Docker/Kubernetes, monitoria e Vault são opcionais e têm [guias de operação](ferramentas/boilerplate/docs/integracoes.md).

```sh
cargo run -p rust-boilerplate -- new
cargo run -p rust-boilerplate -- plan minha-api --type backend --backend express-ts
```

## Verificar o workspace

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Os pacotes compartilham `Cargo.lock` e `target/`. Use `-p guia-rust` para o material didático, `-p rust-devtool` para os utilitários ou `-p rust-boilerplate` para o planejador. O membro padrão é a CLI existente, preservando `cargo run -- help` na raiz. Veja a [referência de workspaces do Cargo](https://doc.rust-lang.org/cargo/reference/workspaces.html).
