#!/usr/bin/env sh
# Verifica toolchain Rust, rustfmt y patrones de .gitignore (T-00-01-01).
set -eu

if [ -n "${CHECK_ROOT:-}" ]; then
  ROOT="${CHECK_ROOT}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi

fail() {
  printf 'check-toolchain: %s\n' "$1" >&2
  exit 1
}

toolchain="${ROOT}/rust-toolchain.toml"
rustfmt="${ROOT}/rustfmt.toml"
gitignore="${ROOT}/.gitignore"

[ -f "${toolchain}" ] || fail 'falta rust-toolchain.toml (canal fijado)'

if ! grep -Eq 'channel[[:space:]]*=[[:space:]]*"1\.99\.0"' "${toolchain}"; then
  fail 'falta el canal fijado: channel = "1.99.0" en rust-toolchain.toml'
fi

if ! grep -q 'rustfmt' "${toolchain}"; then
  fail 'falta el componente rustfmt en rust-toolchain.toml'
fi

if ! grep -q 'clippy' "${toolchain}"; then
  fail 'falta el componente clippy en rust-toolchain.toml'
fi

[ -f "${rustfmt}" ] || fail 'falta rustfmt.toml (edición de rustfmt)'

if ! grep -Eq 'edition[[:space:]]*=[[:space:]]*"2024"' "${rustfmt}"; then
  fail 'falta la edición de rustfmt: edition = "2024" en rustfmt.toml'
fi

[ -f "${gitignore}" ] || fail 'falta .gitignore'

gitignore_active() {
  grep -v '^[[:space:]]*#' "${gitignore}" | grep -v '^[[:space:]]*$'
}

has_ignore_pattern() {
  pattern=$1
  gitignore_active | grep -qF "${pattern}"
}

if ! has_ignore_pattern 'models/'; then
  fail 'falta el patrón models/ en .gitignore'
fi

if ! has_ignore_pattern 'data/'; then
  fail 'falta el patrón data/ en .gitignore'
fi

if gitignore_active | grep -Eq '^(\*\*/)?target/$'; then
  :
elif has_ignore_pattern '/target/'; then
  fail 'el patrón /target/ en .gitignore solo ignora la raíz; se requiere target/ (cualquier profundidad)'
else
  fail 'falta el patrón target/ en .gitignore'
fi

exit 0
