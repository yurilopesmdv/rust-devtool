# Trilha: de TypeScript a ferramentas em Rust

O objetivo é conhecer a linguagem e aplicá-la na construção de ferramentas de desenvolvimento. A experiência com TypeScript ajuda com funções, tipos e modelagem; ownership, referências, tipos numéricos e erros explícitos exigem uma nova base.

Cada etapa tem um README, código executável, comparação com TS e exercícios. Os exemplos usam apenas a biblioteca padrão, sem depender da implementação da CLI existente.

## Preparação

Tenha Rust stable e Cargo instalados conforme o [README da raiz](../README.md). Para os comandos de qualidade, instale os componentes se ainda não estiverem disponíveis:

```sh
rustup component add rustfmt clippy
rustup doc --book
cargo run -p guia-rust --example fundamentos
```

`rustup doc --book` abre o livro oficial localmente. Os exemplos usam edição 2024. Você não precisa instalar a CLI no terminal para estudar.

## Roteiro

Siga a ordem: cada etapa usa conceitos das anteriores. Reserve uma ou mais sessões por etapa; avance pelo entendimento e pelos exercícios, sem prazo fixo.

| Etapa | Foco | Exemplo Cargo | Aplicação em ferramentas |
| --- | --- | --- | --- |
| [01 — Fundamentos](01-fundamentos/README.md) | Tipos, variáveis, funções e expressões | `fundamentos` | Configurações e cálculos |
| [02 — Controle de fluxo](02-controle-de-fluxo/README.md) | Condições, loops e padrões | `controle_de_fluxo` | Seleção de comandos |
| [03 — Ownership](03-ownership/README.md) | Move, Copy, borrowing, strings e slices | `ownership` | Reutilizar dados sem cópias desnecessárias |
| [04 — Modelagem](04-structs-e-enums/README.md) | Struct, impl, enum e Option | `modelagem` | Estados e argumentos válidos |
| [05 — Erros](05-tratamento-de-erros/README.md) | Result, ?, erros próprios e panic | `erros` | Falhas de entrada e diagnósticos |
| [06 — Coleções](06-colecoes-e-iteradores/README.md) | Vec, HashMap, closures e iteradores | `colecoes` | Filtrar e resumir dados |
| [07 — Abstrações](07-traits-generics-lifetimes/README.md) | Traits, generics e lifetimes | `traits` | APIs reutilizáveis e referências |
| [08 — Organização](08-modulos-e-testes/README.md) | Módulos, visibilidade e testes | `modulos` | Separar regras da interface |
| [09 — Arquivos e CLI](09-arquivos-e-cli/README.md) | I/O, argumentos e códigos de saída | `busca` | Busca literal em um arquivo |
| [10 — Concorrência](10-concorrencia/README.md) | Threads, move e canais | `concorrencia` | Distribuir tarefas com segurança |

Os nomes da coluna “Exemplo Cargo” são usados assim, a partir da raiz:

```sh
cargo run -p guia-rust --example ownership
cargo run -p guia-rust --example busca -- erro guia/09-arquivos-e-cli/exemplo.log
```

O exemplo `emprestimo_original` preserva o código de ownership anterior à reorganização.

## Como estudar cada etapa

1. Leia o README e preveja a saída antes de executar.
2. Execute o exemplo e acompanhe o caminho dos valores pelo código.
3. Faça os exercícios alterando o exemplo; não precisa criar outro pacote a cada exercício.
4. Quando o compilador rejeitar algo, leia o diagnóstico e consulte `rustc --explain E0382`, trocando o código pelo apresentado no erro.
5. Explique a decisão em termos de Rust: quem possui os dados, quem empresta, que tipo representa ausência ou falha.
6. Rode a formatação e a verificação antes de registrar suas mudanças.

```sh
cargo fmt --all -- --check
cargo check -p guia-rust --all-targets
cargo test -p guia-rust --all-targets
cargo clippy -p guia-rust --all-targets -- -D warnings
```

`cargo test --all-targets` executa os testes das etapas 05 e 08 e compila os demais alvos em modo de teste. Ele não executa o `main` de cada exemplo: use os comandos das aulas para observar o comportamento.

## Consolidar e continuar

Ao terminar, faça o [projeto de consolidação](projeto-final.md): um plano incremental para transformar o laboratório de busca em uma ferramenta confiável. Sua implementação fica como exercício futuro.

Use a [ponte TypeScript → Rust](referencias/typescript-para-rust.md) para revisar diferenças e a [lista de referências](referencias/README.md) para aprofundar. Depois da base, explore serialização, leitura em streaming, testes de integração, distribuição de CLI, smart pointers e programação assíncrona.

A pasta [ferramentas/](../ferramentas/README.md) já está pronta para receber os utilitários. Os comandos existentes permanecem em um pacote separado para que a trilha possa evoluir sem exigir alterações neles.
