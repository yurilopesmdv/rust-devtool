# DevTool: experimentos existentes

Pacote `rust-devtool`, preservado para estudar a aplicação de Rust em uma CLI.
Todos os comandos abaixo partem da raiz do repositório:

```sh
cargo run -p rust-devtool -- help
cargo run -p rust-devtool -- hello
cargo run -p rust-devtool -- hash ferramentas/devtool/fixtures/exemplo.txt
cargo run -p rust-devtool -- json format ferramentas/devtool/fixtures/exemplo.json
cargo test -p rust-devtool
```

O `cargo run -- help` da raiz também funciona: este pacote é o membro padrão do workspace. O binário continua se chamando `rust-devtool`; o texto interno de ajuda ainda usa `devtool`, como no experimento original.

- `src/main.rs`: argumentos e despacho dos comandos.
- `src/commands/hash.rs`: leitura de bytes e SHA-256.
- `src/commands/json.rs`: leitura de texto e formatação com `serde_json`.
- `fixtures/`: os arquivos de exemplo que antes ficavam na raiz.

Os caminhos passados à CLI são relativos ao diretório de onde você executa o comando. Os exemplos acima imprimem o resultado e não sobrescrevem os arquivos de entrada.

A evolução funcional desta CLI fica para depois da [trilha de aprendizado](../../guia/README.md).
