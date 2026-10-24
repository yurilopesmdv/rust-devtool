# 06 — Coleções, closures e iteradores

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example colecoes
```

Transformar listas e agrupar dados sem perder de vista empréstimos e consumo.

- `Vec<T>` é uma coleção contígua de tamanho variável. `.get(indice)` e `.first()` retornam `Option`, enquanto `[indice]` pode causar panic se o índice for inválido.
- `HashMap<K, V>` associa chaves a valores. `entry(...).or_insert(...)` permite inicializar e atualizar uma entrada. O `*` do exemplo acessa o valor por trás da referência mutável.
- Closures usam `|parametro| expressao` e podem capturar valores do ambiente.
- `.iter()` empresta itens; `.iter_mut()` empresta itens para alteração; `.into_iter()` consome o receptor. Chamado sobre um `Vec<T>` possuído, move seus itens.
- `map` e `filter` criam adaptadores preguiçosos. `collect`, `sum` ou um laço consomem o iterador para realizar o trabalho.
- `collect()` frequentemente precisa que o tipo de destino seja informado. Um `Vec<&str>` pode conter referências ao texto de um `Vec<String>` enquanto os donos permanecem válidos.

### Comparação com TypeScript

```ts
const fontes = arquivos.filter(nome => nome.endsWith(".rs"));
const maiusculos = arquivos.map(nome => nome.toUpperCase());
```

```rust
let fontes: Vec<_> = arquivos.iter()
    .filter(|nome| nome.ends_with(".rs"))
    .collect();
```

Operações de array em JS normalmente produzem os resultados imediatamente; adaptadores de iteradores Rust só executam ao serem consumidos. As duas cadeias podem parecer semelhantes, mas ownership e avaliação diferem.

### Experimente

1. Adicione `.ts` à coleção e conte cada extensão com `HashMap`.
2. Use `.iter_mut()` para adicionar um prefixo a cada `String`, antes de criar referências aos nomes.
3. Substitua uma cadeia por um `for` e compare legibilidade e tipos.

Avance quando conseguir explicar por que `arquivos` não pode ser usado depois de `into_iter()`. A saída original contém dois arquivos `.rs` e a frequência de `info` igual a 2; a ordenação explícita deixa a apresentação previsível.

Próxima etapa: [07 — Traits, generics e lifetimes](../07-traits-generics-lifetimes/README.md).
