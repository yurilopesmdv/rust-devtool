# 05 — Result e tratamento de erros

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example erros
```

Separar ausência, falha esperada e quebra de uma suposição do programa.

- `Option<T>` representa ausência. `Result<T, E>` representa sucesso `Ok(T)` ou falha `Err(E)` com informação.
- `?` devolve antecipadamente um erro para quem chamou, fazendo conversão compatível quando necessário. Ele não lança uma exceção de JavaScript.
- `map_err` transforma o tipo de erro. O exemplo distingue erro de conversão de um número fora do intervalo aceito.
- `Display` fornece a mensagem ao usuário; `Debug` ajuda a inspecionar dados; `std::error::Error` integra um erro ao ecossistema.
- `unwrap()` e `expect()` causam panic em `Err` ou `None`. Reserve-os para suposições garantidas, exemplos fixos e testes, explicando a suposição. Entrada de usuário inválida deve ser tratada.
- `panic!` interrompe o fluxo normal; não é o mecanismo rotineiro para um arquivo inexistente ou argumento inválido.

### Comparação com TypeScript

```ts
type Resultado<T> = { ok: true; valor: T } | { ok: false; erro: string };
// Uma união discriminada ajuda a entender Result melhor que um try/catch.
```

```rust
fn ler_numero(texto: &str) -> Result<u32, std::num::ParseIntError> {
    let numero = texto.parse::<u32>()?;
    Ok(numero)
}
```

As assinaturas comunicam a falha esperada. Quem chama precisa decidir se trata, transforma ou propaga o erro.

### Experimente

1. Execute `cargo test -p guia-rust --example erros`.
2. Adicione uma variante para entrada vazia e um teste que a diferencie de `abc`.
3. Faça uma função que chame `ler_limite` e propague o erro usando `?`.

Avance quando conseguir explicar o comportamento para entrada ausente, `1`, `100`, `0`, `101` e texto inválido. O exemplo imprime limites 10 e 25 e duas mensagens de erro; a ordem relativa de stdout e stderr pode variar.

Próxima etapa: [06 — Coleções, closures e iteradores](../06-colecoes-e-iteradores/README.md).
