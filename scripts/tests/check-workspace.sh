#!/bin/sh
# Behavioral tests for the wrapper; no Rust, services or network are invoked here.
set -eu

workspace_root=$(CDPATH= cd -P "$(dirname "$0")/../.." && pwd)
check_script=$workspace_root/scripts/check-workspace.sh
fixture=$(mktemp -d "${TMPDIR:-/tmp}/rust-devtool-check.XXXXXX")
trap 'rm -rf -- "$fixture"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir -p "$fixture/bin" "$fixture/working directory"

cat > "$fixture/bin/cargo" <<'CARGO'
#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$CHECK_TEST_LOG"
printf '%s\n' "$PWD" >> "$CHECK_TEST_CWD"
if [ "${CHECK_TEST_FAIL:-}" = "${1:-}" ]; then
    exit 23
fi
CARGO
chmod +x "$fixture/bin/cargo"
export CHECK_TEST_LOG="$fixture/commands"
export CHECK_TEST_CWD="$fixture/directories"
export PATH="$fixture/bin:$PATH"

assert_equal() {
    if [ "$1" != "$2" ]; then
        printf 'Esperado: %s\nRecebido: %s\n' "$1" "$2" >&2
        exit 1
    fi
}

# Help and input errors must not resolve the workspace or execute tools.
sh "$check_script" --help > "$fixture/output"
[ ! -e "$CHECK_TEST_LOG" ]
if sh "$check_script" --invalid > "$fixture/output" 2>&1; then
    exit 1
else
    assert_equal 2 "$?"
fi
[ ! -e "$CHECK_TEST_LOG" ]

# The script must find its own workspace when called from another directory.
(cd "$fixture/working directory" && sh "$check_script" > "$fixture/output")
expected='fmt --all -- --check
test --workspace --locked --all-targets
test --workspace --locked --doc
clippy --workspace --locked --all-targets -- -D warnings'
assert_equal "$expected" "$(cat "$CHECK_TEST_LOG")"
while IFS= read -r directory; do
    assert_equal "$workspace_root" "$directory"
done < "$CHECK_TEST_CWD"

: > "$CHECK_TEST_LOG"
sh "$check_script" --offline > "$fixture/output"
expected='fmt --all -- --check
test --workspace --locked --offline --all-targets
test --workspace --locked --offline --doc
clippy --workspace --locked --offline --all-targets -- -D warnings'
assert_equal "$expected" "$(cat "$CHECK_TEST_LOG")"

# A failed test must stop before doctests/Clippy and retain its exit status.
: > "$CHECK_TEST_LOG"
if CHECK_TEST_FAIL=test sh "$check_script" > "$fixture/output" 2>&1; then
    exit 1
else
    assert_equal 23 "$?"
fi
expected='fmt --all -- --check
test --workspace --locked --all-targets'
assert_equal "$expected" "$(cat "$CHECK_TEST_LOG")"

# A formatter failure must not trigger builds or tests.
: > "$CHECK_TEST_LOG"
if CHECK_TEST_FAIL=fmt sh "$check_script" > "$fixture/output" 2>&1; then
    exit 1
else
    assert_equal 23 "$?"
fi
assert_equal 'fmt --all -- --check' "$(cat "$CHECK_TEST_LOG")"
printf '%s\n' 'Wrapper: ajuda, argumentos, destino, offline e falha imediata passaram.'
