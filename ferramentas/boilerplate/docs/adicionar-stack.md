# Como adicionar uma linguagem ou framework

[Guia de uso](../README.md) · [Arquitetura do pacote](arquitetura-do-pacote.md)

O catálogo é orientado por metadados, sem enum de linguagens. Python, Go e Java poderão ter seus próprios IDs, comandos e estruturas. Adicionar uma linguagem ao planejador é diferente de entregar um boilerplate gerável: o primeiro passo cadastra a intenção; o segundo exige fontes, adapters, dependências fixadas e testes reais.

## 1. Adicionar metadados

Crie `catalog/python-fastapi.toml`, por exemplo. O arquivo abaixo é uma proposta de cadastro, não um template já entregue:

```toml
id = "python-fastapi"
role = "backend"
language = "Python"
framework = "FastAPI"
port = 3000
container_port = 3000
architecture = "layered"
status = "planned"
package_manager = "uv"
paths = ["app/controllers/", "app/services/", "tests/", "docs/"]
capabilities = ["postgres"]
install = ["uv", "sync", "--locked"]

[[requirements]]
command = "uv"
args = ["--version"]
purpose = "Instalar dependências; não é necessário para planejar"

[conditional_paths]
postgres = ["app/repositories/", "app/adapters/postgres/"]
```

Ao recompilar, `build.rs` inclui o arquivo automaticamente; não é necessário alterar um dispatcher ou registrar o ID no código da CLI. `language`, `framework` e `package_manager` são metadados textuais. Campos desconhecidos são rejeitados. IDs precisam ser únicos; caminhos absolutos, com `..` ou barras invertidas são rejeitados.

Os comandos são arrays de argumentos, não strings de shell. Os requisitos executam consultas locais de versão no doctor; revise cada programa e argumento antes de adicionar ao catálogo, sem operações de instalação ou acesso a serviços. `install` é somente informativo no plano atual.

## 2. Declarar capacidades e caminhos condicionais

Capacidades atuais: `postgres`, `mongodb`, `redis`, `compose`, `kubernetes`, `prometheus-grafana`, `vault`. Frontend estático pode declarar `prometheus-grafana-container` para permitir monitoria via servidor de container quando Compose estiver selecionado.

Omitir uma capacidade faz o planner rejeitar uma escolha incompatível. Não copie a lista inteira sem prever e validar cada integração. `conditional_paths` contém os caminhos que só aparecem quando aquela integração é escolhida; a estrutura pode ser `app/` para Python, `internal/` para Go ou a árvore apropriada de Java, sem condicionais por linguagem no planejador.

`paths` contém a estrutura base; dados/cache/segredos não devem aparecer nela. Os nomes das integrações são contratos compartilhados, não nomes de linguagens. Novas linguagens reutilizam esses contratos; um novo tipo de integração, como outro banco, exigirá evolução explícita do schema/validador.

## 3. Validar o cadastro no ambiente WSL/Linux

Depois de adicionar o arquivo, na raiz:

```sh
cargo run -p rust-boilerplate -- templates show python-fastapi
cargo run -p rust-boilerplate -- plan minha-api --type backend --backend python-fastapi
cargo run -p rust-boilerplate -- doctor --template python-fastapi
cargo test -p rust-boilerplate --locked
```

O doctor exige `uv` apenas nesse exemplo; o planner deve continuar funcionando sem ele. Esses comandos do exemplo dependem de você primeiro criar os metadados; Python não faz parte das seis stacks já cadastradas.

Inclua testes com a nova estrutura e com uma opção incompatível. Já existe um teste que registra Python em memória e demonstra a extensão sem alterar CLI ou caminhos internos do planner.

## 4. Entregar a geração depois do cadastro

Quando o renderer existir, acrescente fontes, configuração de runtime, comandos de build/test/lint e documentação. Implemente adapters por stack para as capacidades anunciadas: banco, cache, segredos, observabilidade e empacotamento. Fixe versões e valide lockfiles, builds e os comandos publicados.

Teste ao menos a base mínima, a ausência de módulos opcionais, as integrações suportadas e a conexão com um frontend quando aplicável. Só então altere o status para `ready` e disponibilize a stack em `new`. Não basta mudar o status no TOML: new exige fontes embutidas para o ID e rejeita integrações sem adapter. Os assets ficam em `templates/<id>/`; os arquivos opcionais de Compose ficam em `@compose/`. Consulte [manutenção por agentes](manutencao-por-agentes.md).

Java e Go seguirão o mesmo processo, com ferramentas próprias e seus caminhos convencionais. Não há templates dessas linguagens nem instaladores de runtimes nesta revisão.
