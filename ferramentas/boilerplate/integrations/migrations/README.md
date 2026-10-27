# Migrations PostgreSQL

Esta opção é independente da stack da API. Requer Linux/WSL, PostgreSQL 17+, psql 17+, Python 3.8+ (biblioteca padrão para interpretar URI sem expor senha em argumentos), sh, coreutils, util-linux (setsid) e curl para instalar a ferramenta. Aplicação de migrations não é executada em new, startup ou readiness. --install yes instala também o binário local quando esta integração foi escolhida; --install no apenas gera arquivos. A preparação demonstrativa antiga continua disponível; não oferece histórico versionado.

```sh
sh scripts/migrate.sh install
# Exporte URL de um banco já criado; prefira identidade própria de DDL.
# MIGRATION_DATABASE_URL ou MIGRATION_DATABASE_URL_FILE têm preferência.
# Sem elas, usa DATABASE_URL ou DATABASE_URL_FILE. Não misture URL e arquivo.
sh scripts/migrate.sh status
sh scripts/migrate.sh up
```

O instalador reutiliza um dbmate global apenas se seu SHA-256 corresponder ao binário oficial fixado. Caso contrário, instala v2.37.0 em .tools do projeto, com checksum oficial para amd64/arm64; não usa sudo ou altera PATH/perfil. DBMATE_BIN permite fornecer esse mesmo binário já verificado para testes/offline. Não há download automático em status/up.

Exporte as variáveis no processo. O script nunca executa .env como shell e desativa o carregamento de dotenv do dbmate. Não passe URLs por argumentos. Para PostgreSQL local sem TLS, inclua sslmode=disable somente na URL de desenvolvimento; mantenha TLS com validação de certificado nos ambientes que o oferecem. URLs/erros brutos de drivers não são impressos pelo wrapper.

## Organização e execução

Arquivos em migrations/sql usam nome VERSAO_descricao.sql e seções migrate:up/down. A primeira migration cria a tabela demonstrativa idempotentemente, permitindo adotar uma base anteriormente preparada. O wrapper up usa dbmate migrate: exige banco existente, não cria banco, não executa down, não exporta schema e falha para migrations fora de ordem. Schema public é o único suportado inicialmente.

Os arquivos são copiados para snapshot privado antes da execução; o ledger guarda o hash dos bytes efetivamente enviados ao dbmate. Symlinks de arquivos são rejeitados.

A conexão psql mantém uma trava de sessão durante a execução e verifica SHA-256 de todos os arquivos já aplicados. Dbmate também mantém sua própria trava. Conexões diretas ou proxies com pooling por sessão são necessários; pooling por transação não preserva travas de sessão. Espera de trava tem limite de 30s. Aplicações não executam esse comando no startup. Use uma execução controlada de migrations por release.

schema_migrations registra versões pelo dbmate. boilerplate_migration_checksums registra os hashes dos arquivos correspondentes. O wrapper registra os arquivos efetivamente commitados mesmo se uma migration posterior falhar; a migration que falhou continua pendente e pode ser corrigida. Alterar ou remover arquivo já aplicado falha antes de executar novas migrations. Não altere o ledger para contornar o erro: restaure o arquivo original do Git e crie uma nova migration corretiva.

## Falha e recuperação

Migrations SQL usam transação por padrão. Teste cada alteração com banco descartável, incluindo falha parcial e reexecução. Não use transaction:false sem plano específico de recuperação. Rollback destrutivo não é distribuído; a seção down demonstrativa recusa execução.

Há uma janela entre o commit do dbmate e o registro dos checksums pelo wrapper. Se o processo/conexão cair nessa janela, a próxima execução falha fechada por versão aplicada sem checksum. Pare execuções concorrentes, confira versões em schema_migrations e os arquivos exatos do commit executado usando logs/auditoria/backup. Um responsável deve reconciliar o ledger somente após provar o conteúdo aplicado. Não existe comando automático de baseline ou reset que aceite arquivos modificados. Um histórico externo dbmate também exige essa adoção auditada.

O usuário do banco com privilégios de administração pode alterar histórico/ledger; checksums não substituem revisão de código e controle de acesso. Use credenciais de migrations distintas das da API em produção, backup com restauração ensaiada e mudanças expand/backfill/contract compatíveis com a release anterior.

Ao cancelar INT/TERM, o wrapper encerra o grupo de subprocessos. Não use contra banco pessoal/produção em testes. Scripts e diretórios são independentes de Compose/Kubernetes nesta entrega: nenhum Job automático de migrations foi acrescentado.

O cliente psql recebe host/porta/usuário/senha/banco por ambiente. URIs simples e IPv6 são aceitas; query suporta sslmode, sslrootcert, sslcert, sslkey e connect_timeout. URI multihost, socket e opções adicionais são recusadas explicitamente nesta versão.
