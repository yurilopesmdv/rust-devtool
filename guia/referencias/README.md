# Referências para aprofundamento

[Voltar ao roteiro](../README.md) · [Comparações com TypeScript](typescript-para-rust.md)

Use as fontes oficiais para aprofundar os assuntos do guia. A trilha local traz uma ordem de prática, enquanto essas referências explicam a linguagem e as ferramentas com mais abrangência.

- [The Rust Programming Language](https://doc.rust-lang.org/book/): leitura principal para fundamentos, ownership, enums, erros, generics, testes e concorrência. Pode ser aberto localmente com `rustup doc --book`.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/): exemplos adicionais por assunto; útil para testar variações de sintaxe.
- [Biblioteca padrão](https://doc.rust-lang.org/std/): consulte métodos de `String`, `str`, `Vec`, `Option`, `Result`, iteradores, arquivos e threads. `rustup doc --std` abre a documentação local.
- [Cargo Book](https://doc.rust-lang.org/cargo/): pacotes, alvos, dependências e comandos. A [seção de workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) explica a estrutura deste repositório.
- [Rust Reference](https://doc.rust-lang.org/reference/): consulta detalhada das regras da linguagem; não precisa ser sua primeira leitura.
- [Rust Playground](https://play.rust-lang.org/): experimentos pequenos sem criar um projeto local.

Para revisar erros, use `rustc --explain <codigo>`, por exemplo `rustc --explain E0382` para uso de um valor depois de move.

## Depois da trilha

1. Smart pointers: `Box`, `Rc`, `RefCell`, `Arc` e `Mutex`, com atenção a quando cada um é necessário.
2. Macros: diferença entre chamar macros, usar derives e escrever suas próprias macros.
3. Serialização e dependências: estudar a API de `serde_json` já usada na CLI existente.
4. I/O em streaming e testes de integração com arquivos temporários e processos.
5. Futures, `async`/`await` e executores, depois de compreender ownership e concorrência básica.
6. Distribuição: documentação de comandos, instalação local, builds de release e compatibilidade entre sistemas.

`unsafe` e FFI podem ser explorados quando um problema concreto exigir; não são pré-requisitos para as primeiras ferramentas deste repositório.
