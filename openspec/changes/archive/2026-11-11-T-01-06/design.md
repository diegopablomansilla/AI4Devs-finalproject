# Design

## Context

`develop` ya sirve la SPA (T-00-19) y el Gateway responde 404 `application/problem+json` con `code: "not_found"` en rutas no registradas (T-00-11). `vectra-web/` tiene ES Modules sin *build* (ADR-06) y harness `check-vectra-web.sh` + `node --test` sin npm. No existe `js/api.js`. Ver proposal.md para el porqué; la spec para el contrato de `request` / `ApiError`.

Node 22 (local y runner) expone `fetch`, `Response`, `AbortController`. El stub de DOM de componentes no hace falta para HTTP.

## Goals / Non-Goals

**Goals:**

- Un único cliente JSON testeable en Node con `fetch` y `storage` inyectados
- `ApiError` alineado al esquema `Problem`; `network` como código extra de cliente
- Abort distinguible para que T-03-15 / T-01-07 no muestren toast al cancelar búsqueda
- Clave `vectra.terminal` lista para que T-01-11 la escriba

**Non-Goals:**

- Factory pública distinta de `request` en el contrato del ticket (la inyección es detalle de tests)
- Base URL, retries, cola offline, timeout propio
- Importar `api.js` desde `app.js` o vistas (aún no hay callers de negocio)

## Decisions

### 1. Firma del ticket + factory para tests

`api.js` exporta `ApiError`, `request(method, path, { body, signal } = {})` y `createApi({ fetch, storage })`. El `request` de módulo usa `globalThis.fetch` y `globalThis.localStorage`. Los tests llaman `createApi` con un `fetch` falso y un `storage` `{ getItem }`.

Alternativa descartada: tercer argumento `deps` en `request` — ensucia la firma del ticket. Alternativa: `setApiDeps` mutable — peor para tests paralelos.

### 2. `path` tal cual, mismo origen

`fetch(path, { method, headers, body, signal })` con `path` que empieza en `/`. Cubre `/api/v1/…` y `/health` (T-01-12). Sin `baseURL`: en el MVP SPA y API son el mismo origen (`localhost`).

Alternativa descartada: prefijar `/api/v1` — rompería `/health` y el Listo cuando contra `/no-such-route`.

### 3. Abort no es `network`

Si `error.name === "AbortError"`, se re-lanza. Cualquier otro rechazo de `fetch` → `ApiError` `code: "network"`, `status: 0`, `errors: []`. T-03-15 cancela con `AbortController`; T-01-07 solo tostea `ApiError`.

Alternativa descartada: abort → `network`. Cada pulsación en el Omnibox mostraría un error.

### 4. Clave `vectra.terminal`

`storage.getItem("vectra.terminal")`. Valor trimmeado no vacío → cabecera `X-Vectra-Terminal` en **toda** petición (no solo escrituras). OpenAPI marca la cabecera opcional; enviarla de más no cambia el ledger. Si `getItem` lanza o no hay `storage`, se omite la cabecera.

T-01-11 MUST usar la misma clave al guardar "Nombre de esta terminal". No se trunca a 64 (el ajuste de T-01-11 validará).

### 5. Un solo tipo de error de API, salvo abort

- 2xx + JSON → objeto; 2xx/204 cuerpo vacío → `null`
- no OK + JSON con `code` string → `ApiError` (`status` HTTP, `title` del cuerpo o `""`, `errors` o `[]`)
- no OK sin JSON/`code` → `ApiError` `internal`
- no se mira el `Content-Type` como único oráculo (el Gateway a veces añade `charset`)

`ApiError extends Error`; `name = "ApiError"`; `message` = `title` o `code`.

### 6. Tests con `Response` nativo, no servidor vivo

`api.test.mjs` mockea `fetch` y construye `new Response(...)`. El Listo cuando manual (Gateway real, ruta inexistente → `not_found`) se documenta en tasks; la suite CI no arranca `vortex serve`.

## Risks / Trade-offs

- **[Clave de terminal antes de T-01-11]** → Spec fija `vectra.terminal`; T-01-11 no inventa otra. Hasta entonces la cabecera no se envía.
- **[Listo cuando manual]** → El 404 real ya existe en el Gateway; el test automatizado replica el mismo cuerpo. Manual: `vortex serve` + consola o `dev` page no se agrega (fuera del ~2 h y del No hace UI).
- **[2xx no JSON]** → Fuera de alcance (`/metrics` es texto). Un caller que pida esa ruta recibirá `internal` o parse error tratado como `internal` si se fuerza no-OK; en 2xx no JSON se rechaza como `internal` para no filtrar `SyntaxError`.
