# 09 — Arquivos, argumentos e uma pequena CLI

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example busca -- erro guia/09-arquivos-e-cli/exemplo.log
```

Combinar o que foi aprendido em um programa que filtra linhas de um arquivo. Este é um laboratório didático, separado da CLI existente.

- `env::args_os()` lê os argumentos do processo; `skip(1)` ignora o nome do executável. `args()` exige argumentos Unicode; `args_os()` permite preservar caminhos não Unicode.
- A conversão do termo para `String` é validada porque a busca trabalha com texto UTF-8. O caminho permanece `OsString`.
- `fs::read_to_string` lê um arquivo inteiro e falha se ele não existir, não puder ser lido ou contiver texto UTF-8 inválido.
- `.lines().enumerate()` percorre linhas com índices; `contains` faz busca literal sensível a maiúsculas.
- `Box<dyn Error>` reúne diferentes tipos de erro em uma interface comum. É conveniente neste laboratório; erros de domínio explícitos, como na etapa 05, podem ser melhores em bibliotecas.
- `println!` escreve resultados em stdout; `eprintln!` escreve diagnósticos em stderr. `ExitCode` comunica sucesso ou falha ao shell.

### Executar com entrada real

```sh
cargo run -p guia-rust --example busca -- erro guia/09-arquivos-e-cli/exemplo.log
```

Resultado esperado:

```text
2:erro: configuração ausente
```

O `--` separa opções do Cargo dos argumentos da aplicação. O arquivo é lido sem ser modificado. Sem correspondências, a saída é vazia e o código é de sucesso; essa escolha didática difere de utilitários como `grep`. Termo vazio encontra todas as linhas.

### Comparação com TypeScript / Node.js

```ts
import { readFileSync } from "node:fs";
const [termo, arquivo] = process.argv.slice(2);
const texto = readFileSync(arquivo, "utf8");
```

A versão Rust usa `Result` para propagar falhas; a API síncrona acima de Node normalmente lança exceções. Ler todo o conteúdo é simples, mas usa memória proporcional ao arquivo em ambas as versões.

### Experimente

1. Execute sem argumentos, com caminho inexistente e com um argumento extra. Verifique stderr e código de saída (`echo $?` no shell logo após o comando).
2. Adicione uma opção para ignorar diferenças entre maiúsculas e minúsculas.
3. Separe a filtragem em uma função e teste correspondência, ausência e texto vazio.
4. Como extensão, estude `BufRead` e substitua a leitura integral por leitura de linhas.

Avance quando conseguir explicar cada conversão de `Option` para `Result` e a propagação por `?`. O programa deve retornar falha de processo para argumentos inválidos ou arquivo inexistente.

Próxima etapa: [10 — Ownership entre threads](../10-concorrencia/README.md).
