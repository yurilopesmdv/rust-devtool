# ADR 001 — Evolução de schemas

Estado: PostgreSQL implementado na M001 e testado localmente; executor Mongo e prova de compatibilidade seguem M002/M003.

## Contexto

Express/Nest usam pg/MongoClient; Rust usa SQLx/MongoDB; Python usa psycopg/PyMongo. Os repositories possuem consultas, enquanto infraestrutura gerencia conexão e preparação. Hoje db:prepare e equivalentes criam a tabela/coleção demonstrativa de forma idempotente. Reexecutar CREATE IF NOT EXISTS não atualiza tabelas existentes nem registra uma release aplicada.

## Decisão

Preservar os drivers existentes. Migrations são comandos explícitos e revisáveis, executados uma vez por release, antes da aplicação quando compatíveis. Startup HTTP e readiness não alteram schema. A identidade de execução de migrations pode ter DDL; a identidade da aplicação deve receber somente as permissões necessárias ao domínio.

| Banco | Ferramenta/contrato escolhido | Organização |
| --- | --- | --- |
| PostgreSQL, todas as stacks | dbmate como CLI externa fixada e verificada na futura implementação | migrations/sql com arquivos ordenados, aplicação transacional quando suportada e histórico schema_migrations |
| MongoDB, todas as stacks | scripts da própria stack sobre o driver já utilizado, com executor explícito e coleção de histórico | migrations/mongo, id único, checksum, estado, timestamps e checkpoint para backfills |
| Nenhum | Sem ferramenta, pastas ou comandos de migrations | Omitir completamente |

A escolha de dbmate mantém SQL independente da linguagem e dispensa instalar ORM. SQLx migrate foi avaliado: aproveita o driver Rust, mas imporia uma segunda estratégia aos projetos Node/Python. Ferramentas de ORM adicionariam um modelo de persistência que as bases atuais não usam. O contrato Mongo exige implementação separada por adapter; não serão anunciados comandos inexistentes. Não exigir Node em uma API Python ou Rust apenas para executar migrations.

A [documentação do dbmate](https://github.com/amacneil/dbmate) descreve sua CLI independente de framework. A implementação futura deverá fixar versão/checksum, provar o comportamento transacional no PostgreSQL suportado e testar falha/reexecução. Não instalar globalmente como efeito de new.

## Histórico e concorrência

Arquivos aplicados são imutáveis. Uma correção gera nova migration; checksum divergente falha antes de aplicar. O executor deve impedir concorrência, registrar versão/estado e só confirmar aplicação após sucesso. Para Mongo, reservar atomicamente a execução e usar checkpoints idempotentes por lote; a expiração de uma reserva exige tratamento explícito, sem roubar execução ativa. Transação entre documentos depende da topologia e não será presumida em um Mongo standalone: consulte as [condições de transações MongoDB](https://www.mongodb.com/docs/manual/core/transactions/).

## Exemplo: adicionar display_name

1. Release A adiciona coluna PostgreSQL anulável ou campo Mongo opcional. A API antiga continua lendo/escrevendo; a nova aceita ausência e mantém compatibilidade.
2. Executar backfill em lotes limitados, idempotentes, com checkpoint e observação de duração/locks. Testar dados existentes, falha no meio e retomada.
3. Release B passa a preencher o campo. Verificar o backfill e a compatibilidade antes de adicionar restrições; impor NOT NULL, se necessário, somente em etapa posterior.
4. Remover leitura/escrita antiga apenas depois de encerrar a janela de rollback das aplicações.

Em falha de migration SQL transacional, reverter a transação e não registrar como aplicada. Operações não transacionais precisam de plano específico de compensação. Em Mongo, retomar pelo checkpoint após inspecionar o estado. Recuperação preferencial é uma migration corretiva; rollback da aplicação não implica apagar dados ou executar down automaticamente. Mudanças destrutivas exigem backup com restauração ensaiada e janela própria.

## Microentregas de implementação

- M001: adapter dbmate opcional, versão/checksum, status/up explícitos, teste PostgreSQL de falha, concorrência e reexecução; manter db:prepare identificado como demonstração até a substituição.
- M002: executor Mongo mínimo por stack com ledger/checksum/reserva; provar retomada de backfill no standalone e testar topologia transacional separadamente.
- M003: validar release antiga e nova contra schema expandido; documentar recuperação antes de habilitar jobs de migration em Compose/Kubernetes.

As opções continuam explícitas e desativadas por padrão. A implementação PostgreSQL adiciona ledger de checksums ao dbmate, snapshot dos arquivos executados e trava externa de sessão; a janela de queda entre commit e ledger é documentada e falha fechada. Consulte a task M001 e o guia de migrations para os limites efetivamente testados.
