#!/usr/bin/env sh
# Verifica el workspace Cargo vortex-core (T-00-01-03; allowlist domain: T-00-04; vortex: T-00-05, T-00-06).
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

# `domain` allowlist: [dependencies] thiserror|ulid|rust_decimal|serde; [dev-dependencies] serde_json only.
domain_deps_allowed() {
  crate_toml=$1

  if grep -Eq '^\[dev-dependencies\]' "${crate_toml}"; then
    dev_other=$(
      awk '
        /^\[dev-dependencies\]/ { indev = 1; next }
        /^\[/ { indev = 0 }
        indev && /^[^[:space:]#]/ { print }
      ' "${crate_toml}" | grep -v -E '^serde_json[[:space:]]*=' || true
    )
    if [ -n "${dev_other}" ]; then
      return 1
    fi
    if grep -Eq '^\[dev-dependencies\]' "${crate_toml}" \
      && grep -E '^serde_json[[:space:]]*=' "${crate_toml}" >/dev/null 2>&1; then
      grep -E '^serde_json[[:space:]]*=' "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
        || return 1
    fi
  fi

  if ! grep -Eq '^\[dependencies\]' "${crate_toml}"; then
    return 0
  fi

  other_deps=$(
    awk '
      /^\[dependencies\]/ { indeps = 1; next }
      /^\[/ { indeps = 0 }
      indeps && /^[^[:space:]#]/ { print }
    ' "${crate_toml}" | grep -v -E '^(thiserror|ulid|rust_decimal|serde)[[:space:]]*=' || true
  )
  if [ -n "${other_deps}" ]; then
    return 1
  fi

  for dep in thiserror ulid rust_decimal serde; do
    if grep -E "^${dep}[[:space:]]*=" "${crate_toml}" >/dev/null 2>&1; then
      grep -E "^${dep}[[:space:]]*=" "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
        || return 1
    fi
  done
  return 0
}

# `storage` allowlist: [dependencies] surrealdb + thiserror + sha2; [dev-dependencies] tokio only.
storage_deps_allowed() {
  crate_toml=$1

  if grep -Eq '^\[dev-dependencies\]' "${crate_toml}"; then
    dev_other=$(
      awk '
        /^\[dev-dependencies\]/ { indev = 1; next }
        /^\[/ { indev = 0 }
        indev && /^[^[:space:]#]/ { print }
      ' "${crate_toml}" | grep -v -E '^tokio[[:space:]]*=' || true
    )
    if [ -n "${dev_other}" ]; then
      return 1
    fi
    if grep -Eq '^\[dev-dependencies\]' "${crate_toml}" \
      && grep -E '^tokio[[:space:]]*=' "${crate_toml}" >/dev/null 2>&1; then
      grep -E '^tokio[[:space:]]*=' "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
        || return 1
    fi
  fi

  if ! grep -Eq '^\[dependencies\]' "${crate_toml}"; then
    return 0
  fi

  other_deps=$(
    awk '
      /^\[dependencies\]/ { indeps = 1; next }
      /^\[/ { indeps = 0 }
      indeps && /^[^[:space:]#]/ { print }
    ' "${crate_toml}" | grep -v -E '^(surrealdb|thiserror|sha2)[[:space:]]*=' || true
  )
  if [ -n "${other_deps}" ]; then
    return 1
  fi

  if grep -E '^surrealdb[[:space:]]*=' "${crate_toml}" >/dev/null 2>&1; then
    grep -E '^surrealdb[[:space:]]*=' "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
      || return 1
    grep -E '^surrealdb[[:space:]]*=' "${crate_toml}" | grep -Fq 'kv-rocksdb' \
      || return 1
  fi

  if grep -E '^thiserror[[:space:]]*=' "${crate_toml}" >/dev/null 2>&1; then
    grep -E '^thiserror[[:space:]]*=' "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
      || return 1
  fi

  if grep -E '^sha2[[:space:]]*=' "${crate_toml}" >/dev/null 2>&1; then
    grep -E '^sha2[[:space:]]*=' "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
      || return 1
  fi
  return 0
}

# `vortex` allowlist: [dependencies] config + tracing stack + clap; [dev-dependencies] serde_json only.
vortex_deps_allowed() {
  crate_toml=$1

  if grep -Eq '^\[dev-dependencies\]' "${crate_toml}"; then
    dev_other=$(
      awk '
        /^\[dev-dependencies\]/ { indev = 1; next }
        /^\[/ { indev = 0 }
        indev && /^[^[:space:]#]/ { print }
      ' "${crate_toml}" | grep -v -E '^serde_json[[:space:]]*=' || true
    )
    if [ -n "${dev_other}" ]; then
      return 1
    fi
    if grep -Eq '^\[dev-dependencies\]' "${crate_toml}" \
      && grep -E '^serde_json[[:space:]]*=' "${crate_toml}" >/dev/null 2>&1; then
      grep -E '^serde_json[[:space:]]*=' "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
        || return 1
    fi
  fi

  if ! grep -Eq '^\[dependencies\]' "${crate_toml}"; then
    return 0
  fi

  other_deps=$(
    awk '
      /^\[dependencies\]/ { indeps = 1; next }
      /^\[/ { indeps = 0 }
      indeps && /^[^[:space:]#]/ { print }
    ' "${crate_toml}" | grep -v -E '^(toml|serde|thiserror|tracing|tracing-subscriber|tracing-appender|clap)[[:space:]]*=' || true
  )
  if [ -n "${other_deps}" ]; then
    return 1
  fi

  for dep in toml serde thiserror tracing tracing-subscriber tracing-appender clap; do
    if grep -E "^${dep}[[:space:]]*=" "${crate_toml}" >/dev/null 2>&1; then
      grep -E "^${dep}[[:space:]]*=" "${crate_toml}" | grep -Eq 'workspace[[:space:]]*=[[:space:]]*true' \
        || return 1
    fi
  done
  return 0
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
check_pin serde_json 1.0.154
check_pin tracing 0.1.44
check_pin tracing-subscriber 0.3.23
check_pin tracing-appender 0.2.5
check_pin metrics 0.24.6
check_pin metrics-exporter-prometheus 0.18.3
check_pin rcgen 0.14.10
check_pin clap 4.6.7
check_pin toml 1.2.1
check_pin sha2 0.10.9

# --- cada lib: lints heredados, sin dependencias ---
for member in ${REQUIRED_MEMBERS}; do
  crate_toml="${WS}/${member}/Cargo.toml"
  crate_name="${member#crates/}"

  [ -f "${crate_toml}" ] || fail "falta el crate lib: ${crate_name} (no existe Cargo.toml)"

  if ! grep -q 'edition.workspace = true' "${crate_toml}" \
    && ! grep -Eq 'edition[[:space:]]*=[[:space:]]*"2024"' "${crate_toml}"; then
    fail "el crate ${crate_name} debe usar edition 2024 (propia o heredada)"
  fi

  if ! grep -Eq '^workspace[[:space:]]*=[[:space:]]*true' "${crate_toml}"; then
    fail "el crate ${crate_name} no hereda lints del workspace ([lints] workspace = true)"
  fi

  if grep -Eq '^\[(dependencies|dev-dependencies)\]' "${crate_toml}"; then
    if [ "${crate_name}" = "domain" ]; then
      if ! domain_deps_allowed "${crate_toml}"; then
        fail "el crate domain declara dependencias fuera de la allowlist de tipos base"
      fi
    elif [ "${crate_name}" = "storage" ]; then
      if ! storage_deps_allowed "${crate_toml}"; then
        fail "el crate storage declara dependencias fuera de la allowlist de SurrealDB"
      fi
    else
      fail "el crate ${crate_name} no debe declarar [dependencies] en este cambio"
    fi
  fi
done

# --- bin/vortex obligatorio ---
BIN_MEMBER=bin/vortex
BIN_TOML="${WS}/${BIN_MEMBER}/Cargo.toml"
BIN_MAIN="${WS}/${BIN_MEMBER}/src/main.rs"

if ! grep -qF "${BIN_MEMBER}" "${MANIFEST}"; then
  fail 'falta el binario: bin/vortex (no figura en members)'
fi

[ -f "${BIN_TOML}" ] || fail 'falta el binario: bin/vortex (no existe bin/vortex/Cargo.toml)'

[ -f "${BIN_MAIN}" ] || fail 'falta bin/vortex/src/main.rs (binario sin main.rs)'

if ! grep -Eq '^name[[:space:]]*=[[:space:]]*"vortex"' "${BIN_TOML}"; then
  fail 'el crate vortex debe declarar name = "vortex"'
fi

if ! grep -q 'edition.workspace = true' "${BIN_TOML}" \
  && ! grep -Eq 'edition[[:space:]]*=[[:space:]]*"2024"' "${BIN_TOML}"; then
  fail 'el crate vortex debe usar edition 2024 (propia o heredada)'
fi

if ! grep -Eq '^workspace[[:space:]]*=[[:space:]]*true' "${BIN_TOML}"; then
  fail 'el crate vortex no hereda lints del workspace ([lints] workspace = true)'
fi

if grep -Eq '^\[(dependencies|dev-dependencies)\]' "${BIN_TOML}"; then
  if ! vortex_deps_allowed "${BIN_TOML}"; then
    fail 'el crate vortex declara dependencias fuera de la allowlist de configuración'
  fi
fi

exit 0
