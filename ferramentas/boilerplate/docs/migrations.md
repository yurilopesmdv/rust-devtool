# Migrations versionadas opcionais

A opção inicial usa PostgreSQL nas cinco stacks backend existentes. Não há migrations implícitas no startup/readiness e o gerador não cria banco, executa SQL, inicia serviços ou faz rollback. Sem a opção, os projetos mantêm a preparação demonstrativa anterior e não recebem o runner versionado.

```sh
cargo run -p rust-boilerplate -- new minha-api \
  --type backend --backend nest-ts \
  --database postgres --database-location external \
  --migrations dbmate --install no --git yes --github no --non-interactive
cd "$HOME/projects/minha-api"
sh scripts/migrate.sh install
# Exporte MIGRATION_DATABASE_URL (identidade de DDL) ou DATABASE_URL.
sh scripts/migrate.sh status
sh scripts/migrate.sh up
```

Com --install yes, a escolha de instalação das dependências também obtém o binário verificado no projeto. Com --install no, o download só ocorre ao executar install manualmente. O assistente pergunta sobre migrations depois de escolher PostgreSQL; a receita salva backend.migrations como dbmate/none. CLI rejeita dbmate com MongoDB/nenhum banco; desligar o banco por flag também desliga migrations herdadas, salvo escolha explícita incompatível que será rejeitada.

## Dependências e limites

Linux/WSL, PostgreSQL 17+, cliente psql 17+, Python 3.8+ sem bibliotecas adicionais, util-linux (setsid), coreutils e curl. Confira versões antes de executar. Instale o cliente pelo [canal oficial PostgreSQL da sua distribuição](https://www.postgresql.org/download/linux/); o gerador não usa sudo ou altera o sistema.

Dbmate v2.37.0 e SHA-256 dos binários Linux amd64/arm64 estão fixados no script. Um binário global é reutilizado apenas se tiver o checksum esperado; .tools local fica fora do Git e do contexto Docker. A [release oficial](https://github.com/amacneil/dbmate/releases/tag/v2.37.0) permite conferir os artefatos. Não usar latest ou executar download sem verificar integridade.

Cada aplicação recebe docs/MIGRATIONS.md com instalação, variáveis, uso e recuperação. MIGRATION_DATABASE_URL/_FILE permite identidade diferente da API; DATABASE_URL/_FILE é fallback. Não carregamos .env como shell. URIs são interpretadas por biblioteca padrão e transmitidas ao cliente em ambiente, sem senha em argumentos. Logs brutos de conexão não são publicados pelo wrapper.

## Histórico e recuperação

Migrations ficam em migrations/sql. O histórico de versões e o ledger de SHA-256 são verificados antes de aplicar novos arquivos. A execução mantém trava de sessão e dbmate tem outra trava própria; exige conexão direta ou pooling por sessão, não pooling por transação. A migration falha em transação não é registrada; arquivos anteriores que já foram commitados continuam registrados, permitindo corrigir e retomar a pendente.

Nenhum rollback destrutivo é distribuído. Há uma janela entre commit e registro do checksum: queda nessa janela exige reconciliação auditada, nunca baseline/reset automático. O projeto gerado explica como diagnosticar esse caso. Checksums não impedem um administrador de alterar o próprio banco: preserve revisão de releases e permissões separadas.

Nesta entrega, nenhum Job automático de migrations foi criado para Compose/Kubernetes. A preparação demonstrativa não tem ledger e permanece identificada como demonstração. Consulte o [ADR de schemas](../../../docs/desenvolvimento/adr/001-evolucao-de-schemas.md) e a [task M001](../../../docs/desenvolvimento/tasks/M001.md).

## Validar a implementação

```sh
# Banco PostgreSQL vazio e descartável; DBMATE_BIN aponta para o binário fixado.
python3 ferramentas/boilerplate/scripts/validate_migrations.py --disposable-database
```

O verificador exige MIGRATIONS_TEST_URL e DBMATE_BIN, recusa tabelas preexistentes e testa as cinco stacks: status/up repetido, checksum alterado, rollback de falha, retomada e concorrência. Remove somente as tabelas que criou. Não inicia banco; a CI usa seu próprio serviço PostgreSQL descartável.
