#!/usr/bin/env sh
# Verifica el workspace Cargo vortex-core (T-00-01-02).
set -eu

if [ -n "${CHECK_ROOT:-}" ]; then
  ROOT="${CHECK_ROOT}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi

fail() {
  printf 'check-workspace: %s\n' "$1" >&2
  exit 1
}

WS="${ROOT}/vortex-core"
MANIFEST="${WS}/Cargo.toml"

[ -f "${MANIFEST}" ] || fail 'falta vortex-core/Cargo.toml (workspace inválido)'

# --- members obligatorios (once libs) ---
REQUIRED_MEMBERS="
crates/api
crates/domain
crates/catalog
crates/search
crates/recommend
crates/ai
crates/speech
crates/vision
crates/sync
crates/storage
crates/supervisor
"

# --- bin/vortex no debe existir en este cambio ---
if grep -q 'bin/vortex' "${MANIFEST}"; then
  fail 'bin/vortex no pertenece a este cambio (no debe figurar en members)'
fi

if [ -f "${WS}/bin/vortex/Cargo.toml" ]; then
  fail 'bin/vortex no pertenece a este cambio (existe bin/vortex/Cargo.toml)'
fi

for member in ${REQUIRED_MEMBERS}; do
  if ! grep -qF "${member}" "${MANIFEST}"; then
    crate_name="${member#crates/}"
    fail "falta el crate lib: ${crate_name} (member ${member})"
  fi
  lib_rs="${WS}/${member}/src/lib.rs"
  if [ ! -f "${lib_rs}" ]; then
    crate_name="${member#crates/}"
    fail "falta el crate lib: ${crate_name} (no existe ${member}/src/lib.rs)"
  fi
done

# --- resolver y edición de workspace ---
if ! grep -Eq 'resolver[[:space:]]*=[[:space:]]*"3"' "${MANIFEST}"; then
  fail 'el workspace debe declarar resolver = "3"'
fi

if ! grep -Eq 'edition[[:space:]]*=[[:space:]]*"2024"' "${MANIFEST}"; then
  fail 'el workspace debe fijar edition = "2024" en [workspace.package]'
fi

# --- lints de workspace ---
if ! grep -q 'unsafe_code' "${MANIFEST}" || ! grep -Eq 'unsafe_code[[:space:]]*=[[:space:]]*"forbid"' "${MANIFEST}"; then
  fail 'el workspace debe fijar unsafe_code = "forbid" en [workspace.lints.rust]'
fi

if ! grep -Eq 'all[[:space:]]*=[[:space:]]*\{[[:space:]]*level[[:space:]]*=[[:space:]]*"warn"' "${MANIFEST}"; then
  fail 'el workspace debe fijar clippy grupo all en nivel warn'
fi

# --- pins exactos en [workspace.dependencies] ---
# shellcheck disable=SC2059
check_pin() {
  crate=$1
  version=$2
  if ! grep -Eq "^${crate}[[:space:]]*=[[:space:]]*\"=${version}\"" "${MANIFEST}"; then
    if grep -Eq "^${crate}[[:space:]]*=[[:space:]]*\"${version}\"" "${MANIFEST}"; then
      fail "versión no fijada con '=': ${crate}"
    fi
    if grep -Eq "^${crate}[[:space:]]*=[[:space:]]*\"" "${MANIFEST}"; then
      fail "versión no fijada con '=': ${crate}"
    fi
    fail "falta pin exacto: ${crate} = \"=${version}\""
  fi
}

check_pin tokio 1.54.1
check_pin axum 0.8.9
check_pin tower-http 0.7.1
check_pin axum-server 0.9.0
check_pin surrealdb 2.7.0
check_pin meilisearch-sdk 0.33.0
check_pin reqwest 0.13.5
check_pin ort 2.0.0-rc.15
check_pin tokenizers 0.23.2
check_pin whisper-rs 0.16.0
check_pin rust_decimal 1.43.0
check_pin ulid 3.0.0
check_pin thiserror 2.0.21
check_pin serde 1.0.229
check_pin tracing 0.1.44
check_pin tracing-appender 0.2.5
check_pin metrics 0.24.6
check_pin metrics-exporter-prometheus 0.18.3
check_pin rcgen 0.14.10
check_pin clap 4.6.7

# --- cada lib: lints heredados, sin dependencias ---
for member in ${REQUIRED_MEMBERS}; do
  crate_toml="${WS}/${member}/Cargo.toml"
  crate_name="${member#crates/}"

  [ -f "${crate_toml}" ] || fail "falta el crate lib: ${crate_name} (no existe Cargo.toml)"

  if ! grep -q 'edition.workspace = true' "${crate_toml}" \
    && ! grep -Eq 'edition[[:space:]]*=[[:space:]]*"2024"' "${crate_toml}"; then
    fail "el crate ${crate_name} debe usar edition 2024 (propia o heredada)"
  fi

  if ! grep -q 'workspace = true' "${crate_toml}"; then
    fail "el crate ${crate_name} no hereda lints del workspace ([lints] workspace = true)"
  fi

  if grep -Eq '^\[dependencies\]' "${crate_toml}"; then
    fail "el crate ${crate_name} no debe declarar [dependencies] en este cambio"
  fi

  if grep -Eq '^\[dev-dependencies\]' "${crate_toml}"; then
    fail "el crate ${crate_name} no debe declarar [dev-dependencies] en este cambio"
  fi
done

exit 0
