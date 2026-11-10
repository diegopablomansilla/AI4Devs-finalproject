#!/usr/bin/env sh
# Verifica el contrato de GitHub Actions CI base (T-09-01).
set -eu

if [ -n "${CHECK_ROOT:-}" ]; then
  ROOT="${CHECK_ROOT}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi

fail() {
  printf 'check-ci: %s\n' "$1" >&2
  exit 1
}

WORKFLOW="${ROOT}/.github/workflows/ci.yml"

[ -f "${WORKFLOW}" ] || fail 'falta .github/workflows/ci.yml'

# --- triggers ---
if ! grep -q 'pull_request' "${WORKFLOW}"; then
  fail 'falta el trigger pull_request en el workflow'
fi

# --- jobs obligatorios ---
for job in fmt clippy test; do
  if ! grep -Eq "^[[:space:]]{2}${job}:" "${WORKFLOW}"; then
    fail "falta el job: ${job}"
  fi
done

# --- runner macos-14 (sin ubuntu en este workflow) ---
if grep -Eq 'runs-on:[[:space:]]*ubuntu' "${WORKFLOW}"; then
  fail 'el workflow no debe usar runner ubuntu (se requiere macos-14)'
fi

runs_on_count=$(grep -c 'runs-on: macos-14' "${WORKFLOW}" 2>/dev/null || true)
if [ "${runs_on_count}" -lt 3 ]; then
  fail 'cada job fmt, clippy y test debe declarar runs-on: macos-14'
fi

# --- caché de Cargo (al menos una por job) ---
cache_count=$(grep -c 'actions/cache@v4' "${WORKFLOW}" 2>/dev/null || true)
if [ "${cache_count}" -lt 3 ]; then
  fail 'cada job fmt, clippy y test debe restaurar caché con actions/cache@v4'
fi

if ! grep -q '.cargo/registry' "${WORKFLOW}"; then
  fail 'falta la caché de ~/.cargo/registry'
fi

if ! grep -q 'vortex-core/target' "${WORKFLOW}"; then
  fail 'falta la caché de vortex-core/target'
fi

# --- workspace vortex-core ---
if ! grep -q 'vortex-core' "${WORKFLOW}"; then
  fail 'el workflow debe apuntar al workspace vortex-core'
fi

# --- comandos § 5 ---
if ! grep -q 'cargo fmt --all -- --check' "${WORKFLOW}"; then
  fail 'el job fmt debe ejecutar cargo fmt --all -- --check'
fi

if ! grep -q 'cargo clippy --workspace -- -D warnings' "${WORKFLOW}"; then
  fail 'el job clippy debe ejecutar cargo clippy --workspace -- -D warnings'
fi

if ! grep -q 'cargo test --workspace' "${WORKFLOW}"; then
  fail 'el job test debe ejecutar cargo test --workspace'
fi

# --- non-goals (motores, OpenAPI, harness local en GitHub) ---
for forbidden in Meilisearch meilisearch Ollama Whisper Redocly redocly 'docs/openapi.yaml' check-domain-deps check-ci.sh; do
  if grep -qF "${forbidden}" "${WORKFLOW}"; then
    fail "el workflow no debe mencionar ${forbidden} (fuera de alcance T-09-01)"
  fi
done

exit 0
