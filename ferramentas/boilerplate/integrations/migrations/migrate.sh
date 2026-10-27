#!/bin/sh
# Explicit migrations only. No startup hook, global install, database creation or rollback.
set -eu
umask 077
root=$(CDPATH= cd -P "$(dirname "$0")/.." && pwd)
fail() { printf '%s\n' "$1" >&2; exit 1; }
case "${1:-}" in install|status|up) action=$1 ;; *) printf '%s\n' 'Uso: sh scripts/migrate.sh install|status|up' >&2; exit 2 ;; esac
[ "$#" -eq 1 ] || exit 2
[ "$(uname -s)" = Linux ] || fail 'Migrations requerem Linux/WSL.'
case "$(uname -m)" in
    x86_64) arch=amd64; checksum=f855e99a0f0a81322014bf686b1cb820d2b4d8074e4eadafd64b851b80ecb15a ;;
    aarch64|arm64) arch=arm64; checksum=42de2bf27989192b9ca8db4cfaa0a9d6b4aee154a03abef30239b7186b1baf82 ;;
    *) fail 'Arquitetura Linux não suportada pelo adapter.' ;;
esac
command -v sha256sum >/dev/null || fail 'Instale coreutils (sha256sum).'
verified() { [ -f "$1" ] && [ -x "$1" ] && [ "$(sha256sum < "$1" | cut -d ' ' -f 1)" = "$checksum" ]; }
binary="$root/.tools/dbmate/v2.37.0/dbmate-linux-$arch"
if [ -n "${DBMATE_BIN:-}" ]; then
    verified "$DBMATE_BIN" || fail 'DBMATE_BIN não corresponde ao SHA-256 oficial fixado.'
    binary=$DBMATE_BIN
elif global=$(command -v dbmate 2>/dev/null) && verified "$global"; then
    binary=$global
fi
if [ "$action" = install ]; then
    if verified "$binary"; then printf '%s\n' 'dbmate fixado já está disponível.'; exit 0; fi
    command -v curl >/dev/null || fail 'Instale curl para obter o binário fixado.'
    mkdir -p "$(dirname "$binary")"
    download=$(mktemp "$(dirname "$binary")/.download.XXXXXX")
    trap 'rm -f -- "$download"' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 \
        "https://github.com/amacneil/dbmate/releases/download/v2.37.0/dbmate-linux-$arch" -o "$download"
    actual=$(sha256sum < "$download" | cut -d ' ' -f 1)
    [ "$actual" = "$checksum" ] || fail 'Checksum do download divergente; binário descartado.'
    chmod 700 "$download"
    mv -- "$download" "$binary"
    printf '%s\n' 'dbmate instalado somente no projeto.'
    exit 0
fi
verified "$binary" || fail 'Execute primeiro: sh scripts/migrate.sh install'
command -v python3 >/dev/null || fail 'Instale Python 3.8+ (somente biblioteca padrão).'
command -v psql >/dev/null || fail 'Instale o cliente PostgreSQL 17+ (psql).'
command -v setsid >/dev/null || fail 'Instale util-linux (setsid).'
# Never source .env as shell code. The migration identity can differ from the API identity.
if [ -n "${MIGRATION_DATABASE_URL:-}${MIGRATION_DATABASE_URL_FILE:-}" ]; then
    url=${MIGRATION_DATABASE_URL:-}; url_file=${MIGRATION_DATABASE_URL_FILE:-}
else
    url=${DATABASE_URL:-}; url_file=${DATABASE_URL_FILE:-}
fi
[ -z "$url" ] || [ -z "$url_file" ] || fail 'Configure URL direta OU arquivo, nunca ambos.'
if [ -n "$url_file" ]; then url=$(cat -- "$url_file") || fail 'Não foi possível ler o arquivo de URL.'; fi
case "$url" in postgres://*|postgresql://*) ;; *) fail 'Exporte MIGRATION_DATABASE_URL ou DATABASE_URL PostgreSQL.' ;; esac
export BOILERPLATE_MIGRATION_URL="$url" PGCONNECT_TIMEOUT=5
export BOILERPLATE_DBMATE_BIN="$binary"
work=$(mktemp -d "${TMPDIR:-/tmp}/boilerplate-migrations.XXXXXX")
mkdir "$work/sql"
export BOILERPLATE_MIGRATION_DIR="$work/sql"
pid=
cleanup() {
    if [ -n "$pid" ]; then
        /bin/kill -TERM -- "-$pid" 2>/dev/null || :
        sleep 1
        /bin/kill -KILL -- "-$pid" 2>/dev/null || :
        wait "$pid" 2>/dev/null || :
    fi
    rm -rf -- "$work"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
export BOILERPLATE_MIGRATION_APPLY="$work/apply.sh" BOILERPLATE_MIGRATION_OUTPUT="$work/dbmate.log"
cat > "$work/apply.sh" <<'APPLY'
#!/bin/sh
exec "$BOILERPLATE_DBMATE_BIN" --env BOILERPLATE_MIGRATION_URL --env-file /dev/null \
    --migrations-dir "$BOILERPLATE_MIGRATION_DIR" --migrations-table schema_migrations \
    --no-dump-schema --migration-lock --migration-lock-timeout 30s migrate --strict
APPLY
cat > "$work/run.sql" <<'SQL'
\set ON_ERROR_STOP on
SET statement_timeout = '30s';
SET lock_timeout = '30s';
SELECT pg_advisory_lock(72757374646576) \g /dev/null
DO $$ BEGIN
 IF current_schema() <> 'public' THEN RAISE EXCEPTION 'Use o schema public para este adapter'; END IF;
 IF current_setting('server_version_num')::int < 170000 THEN RAISE EXCEPTION 'PostgreSQL 17+ obrigatório'; END IF;
END $$;
CREATE TEMP TABLE boilerplate_expected_migrations (version text PRIMARY KEY, sha256 text NOT NULL);
SQL
count=0
for file in "$root"/migrations/sql/*.sql; do
    [ -f "$file" ] || continue
    name=${file##*/}
    case "$name" in *[!a-zA-Z0-9_.-]*|[!0-9]*) fail 'Nome de migration inválido.' ;; esac
    version=${name%%_*}
    case "$version" in ''|*[!0-9]*) fail 'Migration precisa de versão numérica seguida de _.' ;; esac
    [ ! -L "$file" ] || fail 'Symlink de migration não é aceito.'
    cp -- "$file" "$work/sql/$name"
    digest=$(sha256sum < "$work/sql/$name" | cut -d ' ' -f 1)
    printf "INSERT INTO boilerplate_expected_migrations VALUES ('%s','%s');\n" "$version" "$digest" >> "$work/run.sql"
    count=$((count + 1))
done
[ "$count" -gt 0 ] || fail 'Nenhuma migration SQL encontrada.'
if [ "$action" = up ]; then
    cat >> "$work/run.sql" <<'SQL'
CREATE TABLE IF NOT EXISTS public.boilerplate_migration_checksums (
 version text PRIMARY KEY, sha256 text NOT NULL, recorded_at timestamptz NOT NULL DEFAULT now()
);
SQL
fi
cat >> "$work/run.sql" <<'SQL'
DO $$ BEGIN
 IF to_regclass('public.schema_migrations') IS NOT NULL THEN
  IF to_regclass('public.boilerplate_migration_checksums') IS NULL THEN
   IF EXISTS (SELECT 1 FROM public.schema_migrations) THEN RAISE EXCEPTION 'Histórico sem checksums; reconciliação manual necessária'; END IF;
  ELSE
   IF EXISTS (SELECT 1 FROM public.schema_migrations m LEFT JOIN public.boilerplate_migration_checksums c ON c.version=m.version LEFT JOIN boilerplate_expected_migrations e ON e.version=m.version WHERE c.sha256 IS NULL OR e.sha256 IS NULL OR c.sha256<>e.sha256) THEN RAISE EXCEPTION 'Migration aplicada removida/alterada ou sem checksum'; END IF;
  END IF;
 END IF;
 IF to_regclass('public.boilerplate_migration_checksums') IS NOT NULL THEN
  IF EXISTS (SELECT 1 FROM public.boilerplate_migration_checksums c LEFT JOIN boilerplate_expected_migrations e USING(version) WHERE e.sha256 IS NULL OR c.sha256<>e.sha256) THEN RAISE EXCEPTION 'Ledger divergente dos arquivos'; END IF;
 END IF;
END $$;
SQL
if [ "$action" = up ]; then
    cat >> "$work/run.sql" <<'SQL'
\! sh "$BOILERPLATE_MIGRATION_APPLY" >"$BOILERPLATE_MIGRATION_OUTPUT" 2>&1
\set migration_failed :SHELL_ERROR
-- Record every committed migration, including earlier successes before a later failure.
INSERT INTO public.boilerplate_migration_checksums(version,sha256)
 SELECT e.version,e.sha256 FROM boilerplate_expected_migrations e
 JOIN public.schema_migrations m ON m.version=e.version
 ON CONFLICT (version) DO NOTHING;
\if :migration_failed
DO $$ BEGIN RAISE EXCEPTION 'dbmate reportou falha; confira a migration pendente'; END $$;
\endif
SQL
fi
cat >> "$work/run.sql" <<'SQL'
DO $$ BEGIN
 IF to_regclass('public.schema_migrations') IS NULL THEN
  CREATE TEMP TABLE boilerplate_applied_versions(version text);
 ELSE
  CREATE TEMP TABLE boilerplate_applied_versions AS SELECT version FROM public.schema_migrations;
 END IF;
END $$;
SELECT e.version, CASE WHEN a.version IS NULL THEN 'pending' ELSE 'applied' END AS status
 FROM boilerplate_expected_migrations e LEFT JOIN boilerplate_applied_versions a USING(version) ORDER BY e.version;
SQL
# psql keeps the outer advisory lock during dbmate. Its own lock also guards direct dbmate invocations.
setsid python3 "$root/scripts/_pg_client.py" --no-psqlrc --set ON_ERROR_STOP=on --quiet --file "$work/run.sql" >"$work/result" 2>"$work/error" &
pid=$!
result=0
wait "$pid" || result=$?
pid=
if [ "$result" -ne 0 ]; then
    fail 'Migrations falharam: confira conectividade/permissões, SQL e integridade do histórico. Nenhuma URL foi exibida. Consulte docs/MIGRATIONS.md para recuperar.'
fi
cat "$work/result"
