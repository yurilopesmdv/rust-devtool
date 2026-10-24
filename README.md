# Rust DevTool

Aprender Rust na prática, partindo de uma base em TypeScript e avançando até a construção de ferramentas de desenvolvimento para o terminal.

O repositório é um workspace Cargo com dois espaços:

- [guia/](guia/README.md): roteiro, explicações e exemplos executáveis da linguagem.
- [ferramentas/](ferramentas/README.md): utilitários de desenvolvimento; a CLI existente está em [ferramentas/devtool/](ferramentas/devtool/README.md).

## Começar

Instale a toolchain stable de Rust com as [instruções oficiais](https://www.rust-lang.org/tools/install). O projeto usa a edição 2024; utilize uma versão stable atual que a suporte.

```sh
rustc --version
cargo --version
cargo run -p guia-rust --example emprestimo_original
cargo run -p rust-devtool -- help
```

Execute os comandos deste repositório a partir da raiz, salvo indicação diferente.
O compilador produz os binários em `target/`; o antigo executável avulso `ownership`, se presente localmente, não faz parte dos fontes.

## Verificar o workspace

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Os pacotes compartilham `Cargo.lock` e `target/`. Use `-p guia-rust` para o material didático ou `-p rust-devtool` para a CLI. O membro padrão é a CLI existente, preservando `cargo run -- help` na raiz. Veja a [referência de workspaces do Cargo](https://doc.rust-lang.org/cargo/reference/workspaces.html).
