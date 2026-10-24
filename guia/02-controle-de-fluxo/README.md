# 02 — Controle de fluxo e padrões

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example controle_de_fluxo
```

Escolher caminhos e percorrer dados sem abandonar as garantias de tipos.

- `if` é uma expressão: os braços precisam produzir tipos compatíveis. A condição precisa ser `bool`; não existe coerção por truthiness.
- `match` compara padrões e precisa cobrir todos os casos possíveis. `_` é um padrão que aceita os demais casos.
- `for` percorre um iterável; `enumerate()` adiciona índices começando em zero. `1..3` exclui 3; `1..=3` inclui 3.
- `while` repete enquanto uma condição for verdadeira. `loop` repete até um `break` e permite `break valor`.
- `continue` passa à próxima iteração. `break` encerra o laço.

### Comparação com TypeScript

```ts
const descricao = comando === "hash" ? "resumo de bytes" : "outro";
for (const [indice, nome] of comandos.entries()) console.log(indice, nome);
```

```rust
let descricao = match comando {
    "hash" => "resumo de bytes",
    _ => "outro",
};
```

`match` lembra um `switch`, mas combina padrões, produz valores e não tem fallthrough entre braços. Na etapa 04 ele também decompõe dados de enums.

### Experimente

1. Adicione o comando `version` em `descrever`.
2. Faça o laço ignorar comandos desconhecidos usando `continue`.
3. Troque `1..=3` por `1..3` e compare a saída.

Avance quando conseguir usar `match` para selecionar um comportamento e explicar por que o contador termina em zero. O exemplo original imprime os passos 1 e 3.

Próxima etapa: [03 — Ownership, borrowing e strings](../03-ownership/README.md).
