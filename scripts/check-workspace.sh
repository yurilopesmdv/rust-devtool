#!/bin/sh
# Shared entry point for WSL/Linux contributors and CI. No installation or generation.
set -eu

usage() {
    printf '%s\n' \
        'Uso: sh scripts/check-workspace.sh [--offline]' \
        '' \
        'Verifica formatação, testes de todos os targets, doctests e Clippy.' \
        'Usa Cargo.lock sem atualizá-lo. Para na primeira falha.' \
        '' \
        '  --offline  Exige dependências já presentes no cache Cargo.' \
        '  --help     Mostra esta ajuda sem executar Cargo.' \
        '' \
        'Requisitos: Rust/Cargo e componentes rustfmt e clippy no WSL/Linux.' \
        'Não instala ferramentas nem inicia serviços.' \
        'Fixtures dos testes ficam em diretórios temporários.'
}

offline=no
for argument in "$@"; do
    case "$argument" in
        --offline) offline=yes ;;
        --help|-h) usage; exit 0 ;;
        *) printf 'Argumento inválido: %s\n' "$argument" >&2; usage >&2; exit 2 ;;
    esac
done

if ! command -v cargo >/dev/null 2>&1; then
    printf '%s\n' 'Cargo não encontrado. Carregue a toolchain Rust do WSL/Linux.' >&2
    exit 127
fi

workspace_root=$(CDPATH= cd -P "$(dirname "$0")/.." && pwd)
cd "$workspace_root"

set -- --workspace --locked
if [ "$offline" = yes ]; then
    set -- "$@" --offline
fi

printf '%s\n' '[1/4] Formatação Rust'
cargo fmt --all -- --check
printf '%s\n' '[2/4] Testes do workspace, incluindo exemplos e CLIs'
cargo test "$@" --all-targets
printf '%s\n' '[3/4] Testes de documentação Rust'
cargo test "$@" --doc
printf '%s\n' '[4/4] Clippy sem warnings'
cargo clippy "$@" --all-targets -- -D warnings
printf '%s\n' 'Workspace validado.'
