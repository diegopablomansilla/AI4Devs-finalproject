#!/usr/bin/env sh
# Verifica el esqueleto de vectra-web y ejecuta tests del router (T-01-01).
set -eu

if [ -n "${CHECK_ROOT:-}" ]; then
  ROOT="${CHECK_ROOT}"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi

fail() {
  printf 'check-vectra-web: %s\n' "$1" >&2
  exit 1
}

WEB="${ROOT}/vectra-web"
INDEX="${WEB}/index.html"

[ -d "${WEB}" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/'
[ -d "${WEB}/css" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/css/'
[ -d "${WEB}/js/components" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/js/components/'
[ -f "${WEB}/js/app.js" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/js/app.js'
[ -f "${WEB}/js/router.js" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/js/router.js'

[ -f "${INDEX}" ] || fail 'index_html_declares_lang_es_and_versioned_module: falta vectra-web/index.html'

if ! grep -q 'lang="es"' "${INDEX}"; then
  fail 'index_html_declares_lang_es_and_versioned_module: html debe tener lang="es"'
fi

if ! grep -Eq 'type="module"[[:space:]]+src="/static/\{\{VERSION\}\}/js/app\.js"' "${INDEX}" \
  && ! grep -Eq 'src="/static/\{\{VERSION\}\}/js/app\.js"[[:space:]]+type="module"' "${INDEX}"; then
  fail 'index_html_declares_lang_es_and_versioned_module: falta script type="module" src="/static/{{VERSION}}/js/app.js"'
fi

if ! grep -q 'id="app"' "${INDEX}"; then
  fail 'index_html_declares_lang_es_and_versioned_module: falta outlet id="app"'
fi

if grep -q 'package\.json' "${INDEX}"; then
  fail 'index_html_declares_lang_es_and_versioned_module: index no debe referenciar package.json'
fi

ROUTER_TEST="${ROOT}/scripts/fixtures/vectra-web/router.test.mjs"
[ -f "${ROUTER_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/router.test.mjs'

if ! command -v node >/dev/null 2>&1; then
  fail 'node no está en PATH (se requiere para tests del router)'
fi

node --test "${ROUTER_TEST}"
