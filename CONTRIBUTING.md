# Desenvolvimento do rust-devtool

[Microtasks](docs/desenvolvimento/README.md) · [Revisão do processo](docs/desenvolvimento/revisao.md)

Execute em WSL/Linux. O workspace reúne guia Rust, utilitários e gerador de boilerplates; mantenha as responsabilidades dos pacotes separadas. O membro padrão continua sendo a devtool: use o comando de validação abaixo para incluir todos os pacotes.

## Antes de alterar código

Escolha uma microtask, leia seus critérios e identifique o comportamento a preservar. Para bugs, reproduza a falha antes da correção. Para uma stack nova, defina o contrato mínimo e seus testes antes de anunciar status ready no catálogo. Consulte os arquivos e a receita da stack afetada.

Instale Rust/Cargo no Linux e os componentes de desenvolvimento, se ainda não estiverem presentes:

```sh
rustup component add rustfmt clippy
```

A instalação é uma ação explícita sua; os scripts de verificação não instalam ferramentas.

## Checks do workspace

Na raiz:

```sh
sh scripts/check-workspace.sh
```

Com caches preenchidos, para não acessar rede:

```sh
sh scripts/check-workspace.sh --offline
```

O script também funciona quando chamado pelo caminho absoluto de outro diretório. Executa cargo fmt --check, testes de todos os pacotes/targets, doctests e Clippy com warnings como erro. Os comandos de resolução usam --locked, preservando Cargo.lock; se manifesto e lock divergirem, o check falha. Não inicia bancos, containers ou GitHub, nem cria projetos de exemplo fora dos diretórios temporários dos testes.

Para verificar somente o comportamento do wrapper, sem compilar Rust:

```sh
sh scripts/tests/check-workspace.sh
```

Ajuda/argumentos inválidos não executam Cargo. A primeira falha encerra a execução com seu código de saída. Se faltar rustfmt/Clippy, prepare os componentes e execute novamente. Não rode cargo fmt ou atualize locks silenciosamente dentro de um check para torná-lo verde.

## Integração contínua

O [workflow Quality](.github/workflows/quality.yml) executa o wrapper e o mesmo comando em PRs, push em main e execução manual. Usa runner Linux, Rust stable, token de leitura e checkout por SHA, sem credenciais persistidas. A primeira execução no GitHub ainda precisa ser observada após o envio dos arquivos; validação local do YAML não equivale a um run remoto.

O workflow cobre o workspace. CI das aplicações geradas e persistência estão nas tasks T003–T006; esses jobs ainda não estão implementados. Para exigir um check antes de merge, configure as regras de branch/required checks no GitHub depois do primeiro run. Esse efeito não vem apenas do arquivo YAML.

## Checks da aplicação gerada

Os testes do workspace verificam a geração; também é necessário validar a aplicação resultante quando mudar templates, adapters ou dependências. Veja [validação dos boilerplates](ferramentas/boilerplate/docs/validacao.md) e [lint/formatação por linguagem](ferramentas/boilerplate/docs/qualidade.md).

Exemplo de validação mínima da stack afetada:

```sh
python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal --stacks nest-ts
```

Use serviços descartáveis dedicados para --with-services. Não aponte a suíte para bancos pessoais ou de produção. Checks que pulam persistência por ausência de configuração não comprovam que o banco funciona. Build de imagem e deploy de cluster exigem evidência própria.

## Registrar a entrega

Revise o diff, atualize documentação/receitas afetadas e registre os comandos efetivamente executados na task. Diferencie validação local, testes com serviços e execução remota. Não conclua tasks dependentes por antecipação, nem marque uma stack ready sem seus checks. Uma correção comum não exige executar todos os serviços e todas as linguagens: amplie a validação conforme o impacto da mudança.
