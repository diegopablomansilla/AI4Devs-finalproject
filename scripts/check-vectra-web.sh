#!/usr/bin/env sh
# Verifica vectra-web: árbol, tokens (T-01-02), router (T-01-01), componentes (T-01-03), overlays (T-01-04), product-card (T-01-05), api (T-01-06), errors (T-01-07), omnibox foco (T-01-08), modos y vx-query (T-01-09), keyboard-list (T-01-10).
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
[ -f "${WEB}/css/tokens.css" ] || fail 'falta tokens.css (vectra-web/css/tokens.css)'
[ -f "${WEB}/css/base.css" ] || fail 'falta vectra-web/css/base.css'
[ -d "${WEB}/js/components" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/js/components/'
[ -f "${WEB}/js/app.js" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/js/app.js'
[ -f "${WEB}/js/router.js" ] || fail 'vectra_web_tree_includes_css_app_router_components: falta vectra-web/js/router.js'
[ -f "${WEB}/js/api.js" ] || fail 'api_tree_includes_api_js: falta vectra-web/js/api.js'
[ -f "${WEB}/js/errors.js" ] || fail 'errors_tree_includes_errors_js: falta vectra-web/js/errors.js'
[ -f "${WEB}/js/keyboard-list.js" ] || fail 'keyboard_list_tree_includes_module: falta vectra-web/js/keyboard-list.js'

COMPONENTS_DIR="${WEB}/js/components"
for rel in shared.js index.js vx-button.js vx-field.js vx-chip.js; do
  [ -f "${COMPONENTS_DIR}/${rel}" ] || fail "components_tree_includes_button_field_chip: falta vectra-web/js/components/${rel}"
done

for rel in vx-modal.js vx-side-panel.js vx-toast.js; do
  [ -f "${COMPONENTS_DIR}/${rel}" ] || fail "components_tree_includes_modal_panel_toast: falta vectra-web/js/components/${rel}"
done

[ -f "${COMPONENTS_DIR}/vx-product-card.js" ] || fail "components_tree_includes_product_card: falta vectra-web/js/components/vx-product-card.js"
[ -f "${COMPONENTS_DIR}/vx-omnibox.js" ] || fail "components_tree_includes_omnibox: falta vectra-web/js/components/vx-omnibox.js"

DEV_HTML="${WEB}/dev/components.html"
[ -f "${DEV_HTML}" ] || fail 'components_tree_includes_button_field_chip: falta vectra-web/dev/components.html'

for tag in vx-button vx-field vx-chip; do
  if ! grep -q "<${tag}" "${DEV_HTML}"; then
    fail "components_html_catalog_loads_three_tags: falta <${tag}> en dev/components.html"
  fi
done

for tag in vx-modal vx-side-panel vx-toast; do
  if ! grep -q "<${tag}" "${DEV_HTML}"; then
    fail "components_html_catalog_loads_overlay_tags: falta <${tag}> en dev/components.html"
  fi
done

if ! grep -q "<vx-product-card" "${DEV_HTML}"; then
  fail "components_html_catalog_loads_product_card_variants: falta <vx-product-card> en dev/components.html"
fi
if ! grep -q 'variant="full"' "${DEV_HTML}" || ! grep -q 'variant="compact"' "${DEV_HTML}"; then
  fail "components_html_catalog_loads_product_card_variants: faltan variant=\"full\" y variant=\"compact\" en dev/components.html"
fi

if ! grep -q "<vx-omnibox" "${DEV_HTML}"; then
  fail "components_html_catalog_loads_omnibox: falta <vx-omnibox> en dev/components.html"
fi

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

if ! grep -q 'href="/static/{{VERSION}}/css/tokens.css"' "${INDEX}"; then
  fail 'index_html_links_versioned_tokens_and_base: falta link href="/static/{{VERSION}}/css/tokens.css"'
fi

if ! grep -q 'href="/static/{{VERSION}}/css/base.css"' "${INDEX}"; then
  fail 'index_html_links_versioned_tokens_and_base: falta link href="/static/{{VERSION}}/css/base.css"'
fi

TOKENS_LINE=$(grep -n 'href="/static/{{VERSION}}/css/tokens.css"' "${INDEX}" | head -1 | cut -d: -f1)
BASE_LINE=$(grep -n 'href="/static/{{VERSION}}/css/base.css"' "${INDEX}" | head -1 | cut -d: -f1)
if [ -n "${TOKENS_LINE}" ] && [ -n "${BASE_LINE}" ] && [ "${TOKENS_LINE}" -ge "${BASE_LINE}" ]; then
  fail 'index_html_links_versioned_tokens_and_base: tokens.css debe enlazarse antes que base.css'
fi

ROUTER_TEST="${ROOT}/scripts/fixtures/vectra-web/router.test.mjs"
TOKENS_TEST="${ROOT}/scripts/fixtures/vectra-web/tokens.test.mjs"
COMPONENTS_TEST="${ROOT}/scripts/fixtures/vectra-web/components.test.mjs"
API_TEST="${ROOT}/scripts/fixtures/vectra-web/api.test.mjs"
ERRORS_TEST="${ROOT}/scripts/fixtures/vectra-web/errors.test.mjs"
OMNIBOX_TEST="${ROOT}/scripts/fixtures/vectra-web/omnibox.test.mjs"
KEYBOARD_LIST_TEST="${ROOT}/scripts/fixtures/vectra-web/keyboard-list.test.mjs"
[ -f "${ROUTER_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/router.test.mjs'
[ -f "${TOKENS_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/tokens.test.mjs'
[ -f "${COMPONENTS_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/components.test.mjs'
[ -f "${API_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/api.test.mjs'
[ -f "${ERRORS_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/errors.test.mjs'
[ -f "${OMNIBOX_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/omnibox.test.mjs'
[ -f "${KEYBOARD_LIST_TEST}" ] || fail 'falta scripts/fixtures/vectra-web/keyboard-list.test.mjs'

if ! command -v node >/dev/null 2>&1; then
  fail 'node no está en PATH (se requiere para tests del router)'
fi

node --test "${ROUTER_TEST}" "${TOKENS_TEST}" "${COMPONENTS_TEST}" "${API_TEST}" "${ERRORS_TEST}" "${OMNIBOX_TEST}" "${KEYBOARD_LIST_TEST}"
