# 01 — Fundamentos e sintaxe

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example fundamentos
```

Entender como um programa Rust é compilado e como declarar dados e funções.

- `fn main()` é o ponto de entrada de um binário. `println!` é uma macro; o `!` faz parte da chamada.
- `let` cria um binding imutável por padrão. `let mut` permite reatribuir ou modificar o valor, dentro das regras de empréstimo.
- Shadowing (`let porta = ...` novamente) cria outro binding e pode mudar seu tipo. `const` exige tipo explícito e uma expressão constante.
- Inteiros têm tamanho e sinal: `u32`, `i64`, `usize`. `usize` é usado em índices e tamanhos. `f64`, `bool` e `char` são outros tipos básicos; `char` representa um valor escalar Unicode.
- Uma tupla pode misturar tipos; um array `[T; N]` tem tamanho fixo. `Vec<T>`, na etapa 06, pode crescer.
- Funções declaram os tipos dos parâmetros e do retorno. A última expressão sem `;` produz o retorno. Com `;`, ela vira uma instrução; um bloco sem valor retorna `()`.
- `{valor}` formata com `Display`; `{valor:?}` usa `Debug`. `#[derive(Debug)]` pode gerar essa implementação para seus tipos.

### Comparação com TypeScript

```ts
const somar = (a: number, b: number): number => a + b;
let execucoes: number = 1;
execucoes += 1;
```

```rust
fn somar(a: u32, b: u32) -> u32 { a + b }
let mut execucoes: u32 = 1;
execucoes += 1;
```

`number` não equivale a todos os tipos numéricos de Rust. Uma divisão entre inteiros, como `5 / 2`, resulta em `2`. `const` em TS impede reatribuir o binding, mas permite modificar objetos; ele não traduz diretamente o `const` de Rust.

### Experimente

1. Adicione uma função `dobrar(u32) -> u32` e use seu resultado na mensagem.
2. Remova `mut` de `execucoes` e leia o diagnóstico; depois restaure.
3. Coloque `;` depois de `a + b` e explique o erro de retorno.

Avance quando conseguir explicar a diferença entre expressão, instrução, mutabilidade e shadowing. A saída original deve conter `porta 8080` e `Execuções: 3, limite: 10`.

Próxima etapa: [02 — Controle de fluxo e padrões](../02-controle-de-fluxo/README.md).
