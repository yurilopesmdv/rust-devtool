# Ponte TypeScript → Rust

[Voltar ao roteiro](../README.md)

As comparações ajudam a reconhecer ideias. Elas não são traduções automáticas: procure entender o modelo de execução e de memória de Rust.

| Em TypeScript / Node.js | Em Rust | Diferença para observar |
| --- | --- | --- |
| `const` e `let` | `let`, `let mut`, `const` | Imutabilidade de binding e mutação de objetos em TS não equivalem às regras de empréstimo |
| `number` | `u32`, `i64`, `usize`, `f64` etc. | Tamanho, sinal e operações importam; conversões não são todas implícitas |
| `string` | `String` e `&str` | Texto possuído e texto emprestado; índices de string JS lidam com unidades UTF-16, Rust usa UTF-8 |
| `T[]` | `[T; N]` ou `Vec<T>` | Array fixo e vetor dinâmico têm tipos diferentes |
| `Map<K, V>` | `HashMap<K, V>` | HashMap não preserva ordem de inserção |
| Objeto tipado | `struct` | Campos e métodos via `impl`, sem herança de classes |
| União discriminada | `enum` com dados | `match` permite decompor e verificar cobertura dos casos |
| `T | undefined` | `Option<T>` | Presença explícita com `Some` e `None` |
| União de sucesso/falha | `Result<T, E>` | Falha aparece no retorno e pode ser propagada com `?` |
| `throw` / `try` / `catch` | `Result`, `match`, `?` | `?` retorna cedo; não cria uma exceção que sobe pela pilha |
| `interface` | `trait` | Rust exige implementação do trait; não basta ter o mesmo formato |
| `<T extends X>` | `<T: X>` | Generics Rust podem gerar código especializado, enquanto tipos TS são apagados |
| Arrow function | Closure `|x| ...` | A captura pode emprestar ou mover valores |
| `.map()` / `.filter()` de arrays | Adaptadores de iteradores | Cadeias de iteradores são preguiçosas até serem consumidas |
| Referências de objetos + GC | Ownership, borrowing e Drop | Referências não dão posse livremente compartilhada de qualquer dado |
| `import` / `export` | `mod`, `use`, `pub` | Declarar a árvore de módulos e importar um nome são ações diferentes |
| `package.json` e lockfile | `Cargo.toml` e `Cargo.lock` | Cargo gerencia pacotes, compilação, exemplos, testes e documentação |
| `process.argv.slice(2)` | `env::args_os().skip(1)` | O primeiro argumento Rust é o executável; caminhos podem não ser Unicode |
| `Promise` / `async` | `Future` / `async` | Future normalmente precisa ser polled por um executor para progredir |
| `worker_threads` | `std::thread` | Threads Rust têm restrições verificadas sobre os dados enviados |

## Três hábitos para adaptar

**Comece perguntando quem é o dono.** Em vez de adicionar `clone()` a cada erro, decida se a função precisa consumir o valor, consultá-lo por `&T` ou alterá-lo por `&mut T`.

**Modele ausência e falha.** Um argumento opcional pode ser `Option`; uma configuração inválida pode ser `Result`. Não use panic para erros comuns de entrada.

**Use o compilador como material de estudo.** Diagnósticos de tipos e empréstimos apontam relações que no runtime JavaScript ficariam implícitas. Resolver a relação é mais útil do que contornar o diagnóstico.

Consulte a [etapa 03](../03-ownership/README.md), a [etapa 05](../05-tratamento-de-erros/README.md) e a [etapa 07](../07-traits-generics-lifetimes/README.md) quando essas diferenças aparecerem.
