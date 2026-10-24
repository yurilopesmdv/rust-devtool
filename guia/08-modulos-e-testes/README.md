# 08 — Módulos, visibilidade e testes

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example modulos
```

Separar regras de interpretação da interface do programa e proteger seus contratos com testes.

- Um pacote Cargo tem `Cargo.toml` e pode conter crates de biblioteca, binários e exemplos. Crate é uma unidade de compilação; módulo organiza itens dentro dela.
- `mod comando;` declara um módulo, carregando [comando.rs](comando.rs). `use` traz nomes para o escopo; não baixa dependências.
- Itens são privados por padrão. `pub` expõe itens a quem pode acessar seu caminho; `pub(crate)` limita ao crate.
- O parser usa padrões de slices para exigir a quantidade correta de argumentos. O caminho retornado é emprestado, sem cópia de texto.
- `#[cfg(test)]` inclui o módulo em compilações de teste; `#[test]` marca funções de teste. `assert_eq!` compara valores, `assert!` verifica condições.
- Testes unitários podem morar ao lado das regras. Em pacotes com biblioteca, `tests/` abriga testes de integração usando a API pública; exemplos em documentação da biblioteca também podem ser testados com `cargo test --doc`.

### Comparação com TypeScript

```ts
export type Comando = { tipo: "ajuda" } | { tipo: "hash"; caminho: string };
// import { interpretar } from "./comando";
```

Arquivos TS costumam ser módulos quando importam ou exportam itens. Em Rust, a árvore de módulos precisa ser declarada; criar um `.rs` sozinho não o inclui. `Cargo.toml` gerencia dependências de pacotes, enquanto `mod` organiza código.

### Experimente

1. Execute `cargo test -p guia-rust --example modulos`.
2. Adicione uma variante `Version` e um teste antes de implementá-la no parser.
3. Remova `pub` de `interpretar`, observe o erro na compilação e restaure.

Avance quando conseguir adicionar um comando sem misturar leitura de terminal, regras de argumentos e execução. Os testes cobrem ajuda, caminho válido, caminho ausente, vazio, argumentos excedentes e comando desconhecido.

Próxima etapa: [09 — Arquivos, argumentos e uma pequena CLI](../09-arquivos-e-cli/README.md).
