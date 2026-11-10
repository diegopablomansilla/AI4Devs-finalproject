#!/usr/bin/env sh
# Verifica el árbol normal,build del crate domain (T-00-02-01).
set -eu

if [ -n "${CHECK_ROOT:-}" ]; then
  ROOT="${CHECK_ROOT}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi

fail() {
  printf 'check-domain-deps: %s\n' "$1" >&2
  exit 1
}

WS="${ROOT}/vortex-core"
MANIFEST="${WS}/Cargo.toml"

[ -f "${MANIFEST}" ] || fail 'falta vortex-core/Cargo.toml (workspace inválido)'

DOMAIN_TOML="${WS}/crates/domain/Cargo.toml"
if [ ! -f "${DOMAIN_TOML}" ]; then
  fail 'falta el package domain en el workspace'
fi

if ! grep -Eq '^name[[:space:]]*=[[:space:]]*"domain"' "${DOMAIN_TOML}"; then
  fail 'falta el package domain en el workspace'
fi

if ! cargo tree --manifest-path "${MANIFEST}" -p domain -e normal,build --prefix none --format '{p}' >/dev/null 2>&1; then
  fail 'falta el package domain en el workspace'
fi

TREE_FILE="$(mktemp "${TMPDIR:-/tmp}/check-domain-deps-tree.XXXXXX")"
trap 'rm -f "${TREE_FILE}"' EXIT INT HUP TERM

if ! cargo tree --manifest-path "${MANIFEST}" -p domain -e normal,build --prefix none --format '{p}' >"${TREE_FILE}" 2>/dev/null; then
  fail 'falta el package domain en el workspace'
fi

FORBIDDEN_SDKS="surrealdb meilisearch-sdk reqwest ort whisper-rs"

WORKSPACE_NAMES=""
member_paths=$(sed -n '/^members = \[/,/^\]/p' "${MANIFEST}" | grep -o '"[^"]*"' | tr -d '"')
for path in ${member_paths}; do
  crate_toml="${WS}/${path}/Cargo.toml"
  [ -f "${crate_toml}" ] || continue
  name=$(grep -E '^name[[:space:]]*=' "${crate_toml}" | head -1 | sed -E 's/^name[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')
  case "${name}" in
    domain) ;;
    "") ;;
    *) WORKSPACE_NAMES="${WORKSPACE_NAMES} ${name}" ;;
  esac
done

while IFS= read -r pkgline || [ -n "${pkgline}" ]; do
  [ -n "${pkgline}" ] || continue
  pkg=$(printf '%s' "${pkgline}" | awk '{print $1}')
  [ "${pkg}" = "domain" ] && continue

  for sdk in ${FORBIDDEN_SDKS}; do
    if [ "${pkg}" = "${sdk}" ]; then
      fail "prohibido en el árbol de domain: ${sdk}"
    fi
  done

  for ws_pkg in ${WORKSPACE_NAMES}; do
    if [ "${pkg}" = "${ws_pkg}" ]; then
      fail "prohibido en el árbol de domain: ${ws_pkg}"
    fi
  done
done < "${TREE_FILE}"

exit 0
