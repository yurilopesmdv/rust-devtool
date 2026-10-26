# Backlog de desenvolvimento

[Revisão do processo](revisao.md) · [Fluxo de contribuição](../../CONTRIBUTING.md)

As tasks descrevem entregas pequenas verificáveis. P0 estabiliza o processo, P1 reduz fragilidade/manutenção manual, P2 amplia contratos e testes, P3 expande o produto. Status pendente significa proposta priorizada, não funcionalidade entregue. Implementada com run remoto pendente significa que código e validação local estão prontos, mas falta a execução no GitHub. Concluir uma task exige implementação e evidência; os demais itens continuam neste backlog.

A prioridade inicial é validar de forma repetível as sete stacks existentes. Go é a próxima candidata; Java e mobile ficam em preparação. Nenhuma nova linguagem deve ser marcada ready apenas por ter uma pasta ou metadado.

| Task | Prioridade | Status | Entrega | Depende de |
| --- | --- | --- | --- | --- |
| [T001](tasks/T001.md) | P0 | concluída | Comando único de validação do workspace | — |
| [T002](tasks/T002.md) | P0 | implementada; run remoto pendente | CI do núcleo usando o mesmo comando | T001 |
| [T003](tasks/T003.md) | P0 | implementada; run remoto pendente | CI das bases mínimas JavaScript/TypeScript | T002 |
| [T004](tasks/T004.md) | P0 | implementada; run remoto pendente | CI da base mínima Python | T002 |
| [T005](tasks/T005.md) | P0 | implementada; run remoto pendente | CI da aplicação Rust gerada | T002 |
| [T006](tasks/T006.md) | P1 | implementada; run remoto pendente | CI de persistência com serviços descartáveis | T003, T004, T005 |
| [T007](tasks/T007.md) | P1 | concluída localmente; CI remoto pendente | Checagem dos links locais de documentação | T001 |
| [T008](tasks/T008.md) | P1 | concluída | Contratos exatos para pontos de extensão | T001 |
| [T009](tasks/T009.md) | P1 | concluída | Extrair lifecycle Node para fragmentos legíveis | T008 |
| [T010](tasks/T010.md) | P1 | concluída | Extrair composição de service Rust | T008 |
| [T011](tasks/T011.md) | P1 | concluída localmente; CI remoto pendente | Gerar snapshots de locks Rust por comando | T005 |
| [T012](tasks/T012.md) | P1 | concluída localmente; CI remota pendente | Gerar snapshots de locks Python por comando | T004 |
| [T013](tasks/T013.md) | P1 | concluída | Atualizar locks npm base e superset | T003 |
| [T014](tasks/T014.md) | P1 | concluída | Timeout e interrupção de processos externos | T001 |
| [T015](tasks/T015.md) | P1 | concluída | Contrato JSON para falhas operacionais | T001 |
| [T016](tasks/T016.md) | P2 | pendente | Uniformizar limites Unicode dos contratos HTTP | T003, T004, T005 |
| [T017](tasks/T017.md) | P2 | pendente | Especificar evolução de schemas | T006 |
| [T018](tasks/T018.md) | P2 | pendente | Smoke test das imagens Docker | T003, T004, T005 |
| [T019](tasks/T019.md) | P2 | pendente | Teste automatizado do assistente em PTY | T001 |
| [T020](tasks/T020.md) | P2 | pendente | Política de atualização de dependências | T002 |
| [T021](tasks/T021.md) | P3 | pendente | Go: base HTTP mínima | T003, T004, T005, T008 |
| [T022](tasks/T022.md) | P3 | pendente | Go: adapter PostgreSQL opcional | T021, T006 |
| [T023](tasks/T023.md) | P3 | pendente | CI opcional nos projetos gerados | T003, T004, T005 |
| [T024](tasks/T024.md) | P3 | pendente | Preparar contratos para Java e mobile | T020 |

A primeira sequência é T001 → T002 → T003/T004/T005 → T006. T007–T010 podem avançar após o comando local estar disponível. A expansão de linguagens ocorre depois dos gates de validação correspondentes. Dependências de processo orientam a ordem; não substituem os critérios de aceitação de cada arquivo.
