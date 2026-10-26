# Como adicionar uma linguagem ou framework

[Guia de uso](../README.md) · [Arquitetura do pacote](arquitetura-do-pacote.md)

O catálogo é orientado por metadados, sem enum de linguagens. Python e Go já estão implementados como referências com convenções próprias. Java poderá seguir o mesmo processo. Adicionar uma linguagem ao planejador é diferente de entregar um boilerplate gerável: o primeiro passo cadastra a intenção; o segundo exige fontes, adapters, dependências fixadas e testes reais.

## 1. Adicionar metadados

Crie `catalog/go-chi.toml`, por exemplo. O arquivo abaixo é uma proposta de cadastro, não um template já entregue:

```toml
id = "go-chi"
role = "backend"
language = "Go"
framework = "Chi"
port = 3000
container_port = 3000
architecture = "layered"
status = "planned"
package_manager = "go"
paths = ["cmd/api/", "internal/services/", "tests/", "docs/"]
capabilities = ["postgres"]
install = ["go", "mod", "download"]

[[requirements]]
command = "go"
args = ["version"]
purpose = "Instalar dependências; não é necessário para planejar"

[conditional_paths]
postgres = ["internal/repositories/", "internal/database/"]
```

Ao recompilar, `build.rs` inclui o arquivo automaticamente; não é necessário alterar um dispatcher ou registrar o ID no código da CLI. `language`, `framework` e `package_manager` são metadados textuais. Campos desconhecidos são rejeitados. IDs precisam ser únicos; caminhos absolutos, com `..` ou barras invertidas são rejeitados.

Os comandos são arrays de argumentos, não strings de shell. Os requisitos executam consultas locais de versão no doctor; revise cada programa e argumento antes de adicionar ao catálogo, sem operações de instalação ou acesso a serviços. `install` é somente informativo em plan; new o executa quando a instalação é escolhida. Novos gerenciadores também precisam de seus checks documentados e de suporte no script de validação.

## 2. Declarar capacidades e caminhos condicionais

Capacidades atuais: `postgres`, `mongodb`, `redis`, `compose`, `kubernetes`, `prometheus-grafana`, `vault`, `github-actions` (primeiro adapter de CI em Express). Frontend estático pode declarar `prometheus-grafana-container` para permitir monitoria via servidor de container quando Compose estiver selecionado.

Omitir uma capacidade faz o planner rejeitar uma escolha incompatível. Não copie a lista inteira sem prever e validar cada integração. `conditional_paths` contém os caminhos que só aparecem quando aquela integração é escolhida; a estrutura pode ser `app/` para Python, `internal/` para Go ou a árvore apropriada de Java, sem condicionais por linguagem no planejador.

`paths` contém a estrutura base; dados/cache/segredos não devem aparecer nela. Os nomes das integrações são contratos compartilhados, não nomes de linguagens. Novas linguagens reutilizam esses contratos; um novo tipo de integração, como outro banco, exigirá evolução explícita do schema/validador.

## 3. Validar o cadastro no ambiente WSL/Linux

Depois de adicionar o arquivo, na raiz:

```sh
cargo run -p rust-boilerplate -- templates show go-chi
cargo run -p rust-boilerplate -- plan minha-api --type backend --backend go-chi
cargo run -p rust-boilerplate -- doctor --template go-chi
cargo test -p rust-boilerplate --locked
```

O doctor exige `go` apenas nesse exemplo; o planner continua funcionando sem ele. Os comandos dependem de criar esses metadados: go-chi é apenas o exemplo hipotético desta página. go-http já está pronto e possui catalog/go-http.toml, templates/go-http e src/generator/integrations/go.rs; Python oferece outra referência completa.

Inclua testes com a nova estrutura e com uma opção incompatível. Existe um teste que registra uma stack em memória e demonstra a extensão sem alterar CLI ou caminhos internos do planner.

## 4. Entregar a geração depois do cadastro

Acrescente fontes, configuração de runtime, comandos de build/test/lint e documentação. Implemente adapters por stack para as capacidades anunciadas: banco, cache, segredos, observabilidade e empacotamento. Fixe versões e valide lockfiles, builds e os comandos publicados.

Teste ao menos a base mínima, a ausência de módulos opcionais, as integrações suportadas e a conexão com um frontend quando aplicável. Só então altere o status para `ready` e disponibilize a stack em `new`. Não basta mudar o status no TOML: new exige fontes embutidas para o ID e rejeita integrações sem adapter. Os assets ficam em `templates/<id>/`; os arquivos opcionais de Compose ficam em `@compose/`. Consulte [manutenção por agentes](manutencao-por-agentes.md).

Java seguirá o mesmo processo. O template Go já utiliza seus caminhos convencionais e Go 1.27.3; a instalação global de seu runtime não faz parte da geração. Java/mobile permanecem nos ADRs de preparação.

Os fontes compostos passam por generator/format.rs antes da publicação. JS/TS usa dprint embutido, Rust usa rustfmt e Go usa gofmt da toolchain; Python distribui fontes/fragmentos previamente formatados e testa suas combinações com Ruff. Novas linguagens devem definir esse contrato e um caminho no scripts/validate-generated.py, sem formatadores ou CLIs instalados globalmente.
