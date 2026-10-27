# Ensaio de evolução de schema

Fixtures de teste, não assets gerados automaticamente para todas as APIs. O verificador cria duas APIs Express reais em diretórios temporários: a release antiga usa o repository distribuído; a nova recebe um dos repositories desta pasta, acrescentando display_name opcional e fallback para message.

PostgreSQL: expansão com coluna anulável, backfill que falha/rollback, correção da migration ainda pendente e reexecução. Mongo: campo inicialmente ausente, backfill parcial com checkpoint, reparo de dado incompatível e retomada com arquivo intacto.

Ambas as APIs continuam lendo/escrevendo durante o schema expandido. display_name é campo novo independente; o writer antigo preserva esse valor sem saber atualizá-lo. O writer novo preenche ambos. Antes de contract, o ensaio retira explicitamente o processo antigo; não existe descoberta automática de versões em deployments externos.

Contract impõe NOT NULL (SQL) ou validator Mongo de campo string. A reprodução negativa do writer antigo após contract mostra por HTTP 500 que rollback apenas do binário pode ser inválido. Uma migration corretiva nova relaxa a restrição/restaura o validator anterior sem apagar o histórico ou os dados. O writer antigo volta a criar documentos/linhas e a nova release mantém o fallback.

Comandos exigem Node 24 e banco vazio descartável. O processo não inicia banco ou usa cluster; encerra as APIs e limpa apenas as tabelas/coleções próprias ao final. Veja o [guia de migrations](../../docs/migrations.md) do pacote e o script validate_schema_evolution.py para as variáveis de teste.
