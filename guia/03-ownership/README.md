# 03 — Ownership, borrowing e strings

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example ownership
```

Entender a principal mudança de mentalidade para quem vem de JavaScript: quem possui um valor e por quanto tempo ele pode ser acessado.

- Cada valor tem um dono; ao sair do escopo, o recurso é liberado por `Drop` quando aplicável. Rust não depende de um coletor de lixo para isso.
- Atribuir uma `String` ou passá-la por valor normalmente transfere a posse (*move*). `Copy`, implementado por tipos como `u32`, permite copiar implicitamente.
- `&T` empresta acesso compartilhado; `&mut T` empresta acesso exclusivo. Enquanto esses empréstimos estão ativos, não se pode combiná-los de forma incompatível. O compilador pode encerrar um empréstimo depois do último uso.
- `String` possui texto UTF-8 e pode crescer. `&str` é uma visão emprestada de texto UTF-8. Parâmetros `&str` aceitam tanto literais quanto empréstimos de `String`.
- `.clone()` cria outra instância; em `String`, copia os dados. Escolha copiar quando precisar de independência, e emprestar quando precisar apenas consultar.
- Slices (`&dados[inicio..fim]`) são visões de parte dos dados. Strings não aceitam `texto[0]`: seus caracteres não têm tamanho fixo em bytes. `.len()` conta bytes, `.chars().count()` conta valores escalares Unicode; nenhum deles conta necessariamente caracteres percebidos pelo usuário.

### Comparação com TypeScript

```ts
const original = { nome: "DevTool" };
const outro = original;
outro.nome = "CLI"; // Ambos referenciam o mesmo objeto.
```

```rust
let original = String::from("DevTool");
let outro = original; // A posse mudou; original deixou de ser utilizável.
println!("{outro}");
```

Um `&mut String` não equivale a uma referência livremente compartilhada de um objeto JS: Rust exige exclusividade durante o empréstimo.

### Leia um erro de propósito

Este trecho é intencionalmente inválido; não faz parte do exemplo compilável:

```rust,compile_fail
let original = String::from("DevTool");
let outro = original;
println!("{original}"); // E0382: uso depois de move.
```

Experimente em um arquivo temporário ou no [Rust Playground](https://play.rust-lang.org/). Corrija emprestando `&original` ou clonando, e explique a diferença de custo e intenção.

### Experimente

1. Troque o nome por `ação` e compare bytes e `.chars().count()`; ajuste ou remova o slice fixo antes de executar.
2. Crie uma função que receba `&str` e imprima o nome sem consumi-lo.
3. Tente usar uma referência compartilhada depois de criar e usar uma referência mutável ao mesmo valor; leia o diagnóstico e reorganize os últimos usos.

A saída original inclui `DevTool CLI`. O exemplo anterior do repositório continua em [emprestimo_original.rs](emprestimo_original.rs): execute com `cargo run -p guia-rust --example emprestimo_original`.

Avance quando conseguir escolher entre `String`, `&str`, `&mut String` e `clone()` sem apenas tentar silenciar o compilador.

Próxima etapa: [04 — Structs, enums e Option](../04-structs-e-enums/README.md).
