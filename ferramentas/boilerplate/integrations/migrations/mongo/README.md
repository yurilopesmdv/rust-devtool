# Migrations MongoDB nativas

Execute na pasta da API. Drivers já são os da stack; Rust/Python não dependem de Node para executar migrations. Configure MIGRATION_DATABASE_URL ou MIGRATION_DATABASE_URL_FILE; sem essas variáveis, usa DATABASE_URL/_FILE. Não combine URL e arquivo da mesma família. Não há leitura automática de .env.

- Node (Express/Nest): `node scripts/migrate.mjs status` ou `node scripts/migrate.mjs up`.
- Python: `uv run --locked python scripts/migrate.py status` ou `up`.
- Rust: `cargo run --locked --bin migrate -- status` ou `up`. Requer sha256sum (coreutils). default-run preserva cargo run da API.

Nenhum desses comandos roda em new, startup ou readiness. --install yes instala as dependências normais da stack; nenhum runtime de outra linguagem é acrescentado. Os executores desta entrega são comandos locais, sem Job automático ou promessa de presença nas imagens runtime.

## Arquivos e operações

migrations/mongo/VERSAO_descricao.json usa versão com 14 dígitos, nome ASCII e UTF-8 sem BOM. Arquivos são carregados/validados antes da execução e seus hashes SHA-256 guardam os bytes originais. Não modificar nem remover arquivo que já teve execução iniciada, inclusive se falhou parcialmente. Adicione uma nova versão para corrigir decisões de schema.

A migration inicial bootstrap_examples cria o índice de message e a entrada demonstrativa default idempotentemente. Para backfill, adicione arquivo com versão superior, por exemplo:

```json
{
  "operation": "copy_field",
  "source": "message",
  "target": "display_name",
  "batch_size": 100
}
```

Opera sobre examples: copia valores string apenas onde o destino está ausente. Campos são nomes simples (sem pontos/$), source e target diferentes, lote 1–1000. O executor não avalia JS/Python/Rust vindos do JSON. Novas operações precisam de implementação revisada e testes em cada adapter. Este contrato inicial não é um motor genérico de migração de qualquer coleção.

## Reserva, integridade e checkpoint

_boilerplate_migration_lock tem uma reserva única com owner. Execuções concorrentes esperam até 30s; não existe expiração/roubo automático de reserva. O histórico em _boilerplate_migrations registra checksum, estado pending/running/failed/applied e checkpoint. Novas versões fora de ordem ou arquivos divergentes/removidos são recusados.

No standalone não há transação entre documentos. Cada escrita é condicionada ao destino ausente; ao retomar, os documentos restantes são consultados novamente. Checkpoint conta as alterações confirmadas pelo executor, mas uma queda entre escrita e atualização do ledger pode fazê-lo subcontar; ele é diagnóstico, não condição que pule documentos. A retomada usa o estado dos dados, sem repetir alterações já feitas.

Se encontrar dado inválido durante o backfill, a migration fica failed e preserva o progresso. Corrija os dados conforme o contrato e repita up com o mesmo arquivo/checksum. Não altere o arquivo parcialmente aplicado para contornar o erro. O ledger não guarda mensagens brutas do driver nem valores dos documentos.

## Recuperação de processo interrompido

status informa reserved_by. Antes de remover uma reserva, prove que o processo original parou e bloqueie novas execuções durante a inspeção. Não remova reserva de execução ativa. Com o owner exato e confirmação explícita:

- Node: `node scripts/migrate.mjs unlock OWNER --confirm-stopped`.
- Python: `uv run --locked python scripts/migrate.py unlock OWNER --confirm-stopped`.
- Rust: `cargo run --locked --bin migrate -- unlock OWNER --confirm-stopped`.

O comando só remove a reserva com owner correspondente; não apaga histórico/dados. Uma versão running/failed com arquivo intacto pode ser retomada após liberar a reserva de processo comprovadamente encerrado. SIGKILL e falha de rede podem exigir essa intervenção; não há garantia de cleanup assíncrono em encerramento forçado.

Use identidade de migrations separada e backup com restauração ensaiada. Os drivers se conectam ao banco indicado na URI e preservam opções TLS; não desabilite certificados em produção. Erros de operação são resumidos sem imprimir URI/credenciais. No primeiro uso, execute os testes somente contra Mongo descartável, nunca um banco pessoal/produção.

## Contract e recuperação por versão corretiva

A operação require_field exige que todos os documentos tenham o campo string e compõe a regra com o validator anterior, sem descartá-lo:

```json
{"operation": "require_field", "field": "display_name"}
```

Ela guarda o validator/nível/ação anteriores no ledger antes da alteração. Reexecução usa esse snapshot original; requer identidade com permissão collMod (por exemplo, readWrite + dbAdmin somente neste banco), separada da API quando necessário. Retire writers antigos e verifique o backfill antes do contract; a ferramenta não descobre automaticamente versões de aplicações.

Para uma recuperação revisada, crie uma NOVA migration com versão superior, apontando para uma require_field anterior aplicada:

```json
{"operation": "restore_validator", "from_version": "20261027000300"}
```

O comando restaura o snapshot anterior de validação, preserva o histórico e não reverte dados. Não é rollback automático ou comando down; exige arquivo novo revisado e execução explícita de up. Não use se os writers novos deixarem de tolerar o schema anterior. A recuperação de dados e restauração de backup continuam decisões separadas.
