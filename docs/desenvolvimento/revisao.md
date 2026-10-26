# Revisão do processo de desenvolvimento

[Microtasks](README.md)

A revisão considera o workspace, o gerador, os templates, os adapters e os scripts de validação. O guia Rust e os utilitários continuam separados. O objetivo é tornar uma alteração pequena fácil de revisar, testar e repetir, mantendo rápida a geração de projetos.

## O que já funciona

O catálogo usa metadados e capacidades, os pacotes têm responsabilidade separada, dependências possuem lockfiles e opções desativadas são verificadas por testes. A publicação dos projetos usa staging sem sobrescrever destinos. Há testes de contratos, layouts e falhas operacionais, além de um runner que compila aplicações reais. A revisão anterior executou persistência com serviços dedicados e verificou lint/formatação das stacks.

## Lacunas observadas

| Evidência no repositório | Consequência | Task |
| --- | --- | --- |
| Não havia .github/workflows nem um comando comum para todos os checks Rust | A validação dependia da memória de quem alterava o código | T001–T002 |
| validate-generated.py existia, mas não era chamado por um workflow | Alterar um template podia manter os testes do gerador verdes e quebrar a aplicação | T003–T006 |
| node.rs e rust.rs usam substituições de texto e recortes com find/unwrap | Mudança de formatação/âncoras pode provocar composição incompleta ou panic | T008–T010 |
| Há 12 locks uv, 11 locks Rust opcionais e locks npm base/superset sem atualizador versionado | É fácil esquecer uma combinação ao atualizar dependências | T011–T013 |
| subprocess.run/Command de instalação não possuem timeout explícito | Falhas de ferramentas podem deixar a execução esperando indefinidamente | T014 |
| JSON de sucesso e erros textuais seguem caminhos diferentes | Automação precisa conhecer dois formatos para recuperar falhas | T015 |
| Node usa length enquanto Rust/Python contam caracteres de outra forma | Limites HTTP podem divergir para Unicode | T016 |
| O comando prepare demonstra criação idempotente, sem histórico completo de migrations | Evoluir schemas exige um contrato adicional explícito | T017 |
| Não houve execução das imagens/cluster nesta máquina | Renderizar manifests não comprova que a imagem sobe no ambiente de destino | T018 |

Os arquivos grandes não são automaticamente defeitos. A prioridade é separar responsabilidades onde já há fragilidade concreta; não adicionar interfaces, serviços ou bibliotecas somente para preencher uma arquitetura.

## Processo proposto

1. Escolher uma microtask e ler seus critérios/dependências.
2. Reproduzir o problema ou identificar a evidência que falta.
3. Fazer a menor alteração que entrega o comportamento esperado.
4. Executar o comando do workspace e os checks da aplicação afetada, sem iniciar infraestrutura pessoal.
5. Registrar evidência, limitações e estado da task. Uma configuração validada localmente não deve ser anunciada como CI remoto executado.
6. Revisar contratos/documentação e o diff antes de distribuir a mudança.

CI do núcleo e dos projetos gerados são camadas diferentes. O primeiro verifica o gerador; o segundo verifica o produto do gerador. Persistência real e smoke de imagens pertencem a jobs dedicados, com recursos descartáveis. Alterações comuns não precisam levantar um cluster para receber feedback.

## Expansão do catálogo

A revisão inicial priorizou proteger as sete bases existentes antes de acrescentar linguagens. Após os gates locais, T021/T022 entregaram Go/net/http e PostgreSQL opcional, com build, race, vet, persistência e shutdown reais. Go não anuncia MongoDB/Redis/Vault/monitoria. Java e mobile têm ADRs próprios em T024; suas bases ainda não foram implementadas.

## Referências utilizadas

- [Cargo test: targets e doctests](https://doc.rust-lang.org/cargo/commands/cargo-test.html): testes de documentação devem ser considerados explicitamente ao selecionar targets.
- [Clippy em CI](https://doc.rust-lang.org/stable/clippy/continuous_integration/index.html): execução automatizada e falha em warnings selecionados.
- [GitHub Actions: uso seguro](https://docs.github.com/en/actions/reference/security/secure-use): permissões mínimas e actions referenciadas por SHA completo.

Essas fontes sustentam mecanismos de validação e CI. A prioridade das tasks e a divisão dos adapters resultam da inspeção deste repositório; não são exigências universais de uma arquitetura.
