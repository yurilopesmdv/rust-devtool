# 04 — Structs, enums e Option

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example modelagem
```

Representar os dados e estados de uma ferramenta de maneira explícita.

- `struct` agrupa campos com tipos definidos. `impl` declara funções associadas e métodos. `Self` representa o tipo da implementação.
- `Configuracao::nova()` é uma função associada. `config.exibir()` é um método com `&self`, portanto consulta a configuração por empréstimo. `&mut self` modifica; `self` consome o valor.
- `enum` define variantes, que podem carregar dados diferentes. Um `match` decompõe esses dados e cobre as variantes.
- `Option<T>` representa presença: `Some(T)` ou `None`. Não existe `null` como alternativa implícita de todo tipo.
- `if let` trata um padrão específico; `match` é adequado quando todos os casos importam. `unwrap_or` fornece um valor padrão sem causar panic por ausência.

### Comparação com TypeScript

```ts
type Comando =
  | { tipo: "ajuda" }
  | { tipo: "hash"; caminho: string };
const arquivo: string | undefined = undefined;
```

```rust
enum Comando {
    Ajuda,
    Hash { caminho: String },
}
let arquivo: Option<&str> = None;
```

Enums com dados lembram uniões discriminadas, mas as variantes pertencem ao tipo. `struct` + `impl` cobre muitos usos de dados e métodos; não há herança de classes como em TS.

### Experimente

1. Adicione `Json { caminho: String }` e atualize `executar`.
2. Crie um método com `&mut self` para ativar o modo silencioso.
3. Faça a configuração silenciosa parar de imprimir o nome, mantendo o restante do exemplo compreensível.

Avance quando conseguir modelar um comando sem combinar campos opcionais que permitem estados inválidos. A saída original mostra a ajuda, o caminho `Cargo.toml` e a mensagem `nenhum arquivo` para o argumento ausente.

Próxima etapa: [05 — Result e tratamento de erros](../05-tratamento-de-erros/README.md).
