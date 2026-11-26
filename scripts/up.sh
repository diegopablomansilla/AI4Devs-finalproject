#!/usr/bin/env sh
# Levanta el MVP local: compila vortex, descarga Meilisearch (pin de
# scripts/meilisearch-version), escribe vortex.toml si falta, importa el seed
# cuando todavía no hay base, reindexa Meilisearch si el índice `productos`
# está vacío y deja `vortex serve` en primer plano.
# Vortex no supervisa el sidecar: este script arranca Meilisearch y lo mata
# al salir, solo si lo inició él.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONFIG="${ROOT}/vortex.toml"
VORTEX="${ROOT}/vortex-core/target/debug/vortex"
MEILI="${ROOT}/meilisearch"
MEILI_URL="http://127.0.0.1:7700"
meili_pid=

say() {
  printf 'up: %s\n' "$1" >&2
}

fail() {
  printf 'up: %s\n' "$1" >&2
  exit 1
}

cleanup() {
  if [ -n "$meili_pid" ]; then
    kill "$meili_pid" 2>/dev/null || true
    wait "$meili_pid" 2>/dev/null || true
    meili_pid=
  fi
}

trap cleanup EXIT

toml_value() {
  # $1 archivo, $2 clave (`datos = "..."` o `bind = "..."`).
  sed -n "s/^${2} = \"\\(.*\\)\"$/\\1/p" "$1" | head -n 1
}

require_darwin_arm64() {
  os=$(uname -s)
  arch=$(uname -m)
  if [ "$os" != "Darwin" ] || [ "$arch" != "arm64" ]; then
    fail "hace falta macOS Apple Silicon (Darwin arm64); este equipo es ${os} ${arch}"
  fi
}

require_cargo() {
  if ! command -v cargo >/dev/null 2>&1; then
    fail "cargo no está en PATH; instalá Rust con rustup (https://rustup.rs/)"
  fi
}

build_vortex() {
  say "compilando vortex (debug)"
  cargo build --bin vortex --manifest-path "${ROOT}/vortex-core/Cargo.toml"
  if [ ! -x "$VORTEX" ]; then
    fail "no quedó el binario ${VORTEX}"
  fi
}

ensure_meilisearch_binary() {
  if [ -x "$MEILI" ]; then
    return 0
  fi
  if [ -e "$MEILI" ]; then
    chmod +x "$MEILI"
    return 0
  fi
  tag=$(tr -d '[:space:]' < "${ROOT}/scripts/meilisearch-version")
  url="https://github.com/meilisearch/meilisearch/releases/download/${tag}/meilisearch-macos-apple-silicon"
  partial="${MEILI}.partial"
  say "descargando Meilisearch ${tag}"
  if ! curl -fsSL -o "$partial" "$url"; then
    rm -f "$partial"
    fail "no se pudo descargar Meilisearch ${tag}"
  fi
  chmod +x "$partial"
  mv "$partial" "$MEILI"
}

write_config_if_missing() {
  if [ -f "$CONFIG" ]; then
    say "usando ${CONFIG}"
    return 0
  fi
  say "escribiendo ${CONFIG}"
  cat >"$CONFIG" <<EOF
# Generado por scripts/up.sh. Rutas de esta máquina; no se versiona.

[paths]
datos = "${ROOT}/data"
modelos = "${ROOT}/models"
backups = "${ROOT}/backups"
web = "${ROOT}/vectra-web"

[server]
bind = "127.0.0.1:8080"
hostname = "localhost"

[negocio]
moneda = "ARS"

[meilisearch]
url = "${MEILI_URL}"

[ollama]
url = "http://127.0.0.1:11434"

[backup]
hora = "03:00"
retencion = 14

[logs]
nivel = "info"
retencion = 14
EOF
}

meili_up() {
  health=$(curl -fsS --max-time 2 "${MEILI_URL}/health" 2>/dev/null) || return 1
  printf '%s\n' "$health" | grep -q '"status":"available"'
}

wait_meili() {
  i=0
  while [ "$i" -lt 60 ]; do
    if meili_up; then
      return 0
    fi
    i=$((i + 1))
    sleep 1
  done
  return 1
}

port_busy() {
  if curl -sS --max-time 2 -o /dev/null "http://127.0.0.1:${1}/" 2>/dev/null; then
    return 0
  fi
  return 1
}

start_meili_if_needed() {
  if meili_up; then
    say "Meilisearch ya responde en ${MEILI_URL}"
    return 0
  fi
  say "arrancando Meilisearch en ${MEILI_URL} (log en ${ROOT}/data/meili.log)"
  MEILI_NO_ANALYTICS=true "$MEILI" \
    --http-addr 127.0.0.1:7700 \
    --db-path "${ROOT}/data/meili" \
    >"${ROOT}/data/meili.log" 2>&1 &
  meili_pid=$!
  if ! wait_meili; then
    fail "Meilisearch no respondió available en ${MEILI_URL} (ver ${ROOT}/data/meili.log)"
  fi
}

seed_if_needed() {
  datos=$(toml_value "$CONFIG" datos)
  if [ -z "$datos" ]; then
    fail "no pude leer paths.datos en ${CONFIG}"
  fi
  if [ -d "${datos}/db" ]; then
    say "base existente en ${datos}/db; no corro seed"
    return 0
  fi
  if [ ! -d "${ROOT}/seed" ]; then
    fail "no está ${ROOT}/seed"
  fi
  say "importando seed en ${datos}/db"
  "$VORTEX" --config "$CONFIG" seed "${ROOT}/seed"
}

meili_document_count() {
  stats=$(curl -fsS --max-time 5 "${MEILI_URL}/indexes/productos/stats" 2>/dev/null) || {
    printf '0'
    return 0
  }
  printf '%s' "$stats" | tr -d ' \n' | sed -n 's/.*"numberOfDocuments":\([0-9][0-9]*\).*/\1/p'
}

reindex_if_needed() {
  docs=$(meili_document_count)
  if [ -z "$docs" ]; then
    docs=0
  fi
  if [ "$docs" -gt 0 ]; then
    say "índice Meilisearch ya tiene ${docs} documentos; no corro reindex"
    return 0
  fi
  say "reindexando Meilisearch desde SurrealDB (índice productos vacío)"
  "$VORTEX" --config "$CONFIG" reindex
}

require_darwin_arm64
require_cargo
if [ ! -d "${ROOT}/vectra-web" ]; then
  fail "no está ${ROOT}/vectra-web"
fi

build_vortex
ensure_meilisearch_binary
write_config_if_missing
mkdir -p "${ROOT}/data/meili" "${ROOT}/models" "${ROOT}/backups"

bind=$(toml_value "$CONFIG" bind)
case "$bind" in
  127.0.0.1:*)
    http_port=${bind##*:}
    ;;
  *)
    fail "server.bind en ${CONFIG} debe ser 127.0.0.1:puerto (es '${bind}')"
    ;;
esac

start_meili_if_needed

if port_busy "$http_port"; then
  fail "el puerto ${http_port} ya está ocupado"
fi

seed_if_needed
reindex_if_needed

say "API y PWA en http://127.0.0.1:${http_port}/  (Ctrl+C detiene vortex y el Meilisearch que inició este script)"
set +e
"$VORTEX" --config "$CONFIG" serve
status=$?
set -e
exit "$status"
