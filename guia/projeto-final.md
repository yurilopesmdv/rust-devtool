# Projeto de consolidação: buscador de logs

[Voltar ao roteiro](README.md)

Objetivo: evoluir o laboratório da [etapa 09](09-arquivos-e-cli/README.md) para uma ferramenta pequena e previsível. Este arquivo é um roteiro de exercício; a nova ferramenta ainda não está implementada.

## Etapa 1 — Regras sem acesso ao terminal

Extraia a filtragem para uma função que receba `&str` e devolva as linhas correspondentes. Escolha referências quando fizer sentido evitar cópias. Teste texto vazio, nenhuma correspondência, múltiplas linhas e conteúdo com acentos.

Critério: você consegue testar a busca sem ler arquivos e explicar quem possui o texto retornado.

## Etapa 2 — Argumentos e erros

Modele uma configuração com termo, caminho e opção de ignorar maiúsculas. Separe interpretação de argumentos da execução; rejeite argumentos inválidos com uma mensagem acionável.

Critério: testes cobrem argumentos ausentes, excedentes, desconhecidos e válidos. A ajuda explica se o termo vazio é permitido e como caminhos com espaços devem ser passados entre aspas.

## Etapa 3 — Interface de terminal

Leia um arquivo, imprima correspondências em stdout e erros em stderr. Defina e documente códigos de saída para correspondência, ausência de resultados e falha. Avalie `PathBuf` para caminhos, sem convertê-los desnecessariamente em `String`.

Critério: verificações com um arquivo de teste confirmam resultado, arquivo inexistente e UTF-8 inválido. A ferramenta não sobrescreve a entrada.

## Etapa 4 — Organização como ferramenta

Quando a implementação estiver consolidada, crie um pacote em `ferramentas/busca-logs/` e adicione-o a `members` no workspace. Separe regras reutilizáveis em `src/lib.rs` e a integração com o terminal em `src/main.rs`; use `tests/` para os contratos públicos.

Critério: um README contém instalação, exemplos, limitações e comportamento de erros. O guia continua executável independentemente desse pacote.

## Etapa 5 — Evolução orientada por necessidade

Experimente leitura com `BufRead` para arquivos grandes. Só depois de medir, considere vários arquivos e um número limitado de workers. Documente se a ordem dos resultados é garantida. Como etapa posterior, avalie se uma biblioteca de parsing de argumentos simplifica a interface.

Critério: a complexidade adicional resolve uma necessidade demonstrável e mantém os testes dos contratos anteriores.

## Revisão final

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Revise suas decisões: quais dados são movidos ou emprestados? Quais erros são tratados? Quais entradas são aceitas? Quais suposições ainda podem causar panic? Com essas respostas, escolha o próximo utilitário de uso diário em [ferramentas/](../ferramentas/README.md).
