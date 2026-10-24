# 07 — Traits, generics e lifetimes

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example traits
```

Descrever comportamentos reutilizáveis e relações entre referências.

- Um trait define um contrato de comportamento. `impl Resumo for Tarefa` implementa esse contrato para o tipo; ele não é satisfeito apenas por um objeto ter métodos com os mesmos nomes.
- `T: Resumo` é um limite sobre um parâmetro genérico. O compilador gera implementações especializadas conforme necessário, com despacho estático.
- `&dyn Resumo` usa despacho dinâmico através de um objeto de trait. Nem todo trait pode ser usado dessa forma; há regras de compatibilidade com `dyn`.
- Lifetimes descrevem a validade das referências. Não fazem valores viverem mais tempo e não mudam a duração da memória em execução.
- Em `primeiro_campo<'a>(linha: &'a str, separador: &str) -> &'a str`, a saída está ligada ao empréstimo de `linha`, e não ao separador. A função devolve uma fatia do texto original.
- Muitas assinaturas permitem omitir lifetimes pelas regras de elisão. Anote quando for necessário esclarecer as relações; não acrescente `'static` como correção automática.

### Comparação com TypeScript

```ts
interface Resumo { resumir(): string }
function imprimir<T extends Resumo>(item: T): void {
  console.log(item.resumir());
}
```

TS usa principalmente compatibilidade estrutural; Rust exige a implementação do trait. Tipos TS são apagados na execução; generics Rust podem gerar código especializado. TS não tem um equivalente direto às garantias de lifetime.

### Experimente

1. Implemente `Resumo` para um segundo tipo e use as duas funções de impressão.
2. Crie um `Vec<&dyn Resumo>` com referências aos dois tipos e percorra-o.
3. Tente retornar `&str` apontando para uma `String` criada dentro de uma função. Leia o erro e corrija retornando `String`.

Avance quando conseguir explicar quem possui `linha` e por que `campo` não pode sobreviver a esse texto. O exemplo imprime a tarefa duas vezes e o primeiro campo `erro`.

Próxima etapa: [08 — Módulos, visibilidade e testes](../08-modulos-e-testes/README.md).
