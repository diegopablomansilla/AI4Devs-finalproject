# Design

## Context

En `develop`, T-01-01 dejó `vectra-web/` con *router* *hash*, `css/.gitkeep` y `index.html` sin `<link>`. El Gateway (T-00-19) sirve `{web}/css/…` en `/static/<versión>/css/…`. ADR-06: vanilla, sin *build* ni `package.json`. Ver proposal.md para motivación y `specs/vectra-web/spec.md` para el contrato.

Constraints: ~2 h; **No hace** componentes; Node 22 ya corre `scripts/check-vectra-web.sh`; axe DevTools es extensión de navegador (no hay axe-core en el repo). WCAG AA y 44 px son DoD de UI ([6.1](../../docs/6-historias.md)).

## Goals / Non-Goals

**Goals:**

- Contrato CSS `--vx-*` estable para T-01-03+ (`adoptedStyleSheets` copiará estas hojas)
- Oráculo de contraste WCAG 2.1 en Node, sin npm, que falle en red si un par baja de 4,5:1
- *Glass* por defecto y sólido bajo `prefers-reduced-transparency`
- `index.html` carga las hojas por el prefijo versionado

**Non-Goals (diseño):**

- Constructable stylesheets / `document.adoptedStyleSheets` (T-01-03 las adopta en Shadow DOM)
- Página `dev/components.html` o catálogo visual aparte
- Integrar axe-core, Playwright o un job de CI
- Cambiar el HTML de las vistas del *router* más allá de lo que `base.css` estiliza por elemento
- Tema claro o toggle

## Decisions

### 1. Un archivo de tokens y uno de base; clase `.vx-surface`

**Elección:** `tokens.css` solo custom properties en `:root` (valores literales). `base.css` consume `var(--vx-*)`: reset mínimo (`box-sizing`), `html`/`body`, `:focus-visible`, mínimo táctil en `a, button, input, select, textarea`, y `.vx-surface { background, backdrop-filter, border-radius }`. El *media* `prefers-reduced-transparency: reduce` vive en `base.css` (reescribe `.vx-surface`) y puede reasignar `--vx-color-surface` en `tokens.css` para quien use la variable directo.

**Por qué:** el ticket nombra esos dos archivos. Separar valores de aplicación deja las variables listas para `adoptedStyleSheets`. `.vx-surface` es el gancho del *glass* sin inventar un Web Component.

**Alternativa:** todo en `tokens.css`. Se descartó: el **Hace** pide `base.css`. **Alternativa:** solo variables, sin clase. Se descartó: nadie aplicaría `backdrop-filter` hasta T-01-03 y el Listo cuando (axe sobre superficies) no se podría ejercer en la SPA. **Alternativa:** `@layer`. Se descartó: no está en el stack ni hace falta.

### 2. Mapa URL igual que el JS versionado

**Elección:**

```html
<link rel="stylesheet" href="/static/{{VERSION}}/css/tokens.css" />
<link rel="stylesheet" href="/static/{{VERSION}}/css/base.css" />
```

Orden: *tokens* luego *base*. El Gateway ya mapea `/static/<versión>/css/tokens.css` → `{web}/css/tokens.css`.

**Por qué:** T-00-19 / T-01-01: HTML `no-cache`, assets inmutables. Un `href="css/tokens.css"` relativo al documento no pasa por el prefijo versionado.

**Alternativa:** `@import` desde un único `app.css`. Se descartó: dos *links* coinciden con los dos archivos del ticket y evitan un tercer archivo.

### 3. Contraste en Node (WCAG 2.1), axe DevTools como Listo cuando humano

**Elección:** `scripts/fixtures/vectra-web/tokens-contrast.mjs` exporta `parseRootVars(css)`, `compositeOver(fg, bg)`, `contrastRatio(a, b)`. Los tests `node --test` leen `vectra-web/css/tokens.css` (happy) y un fixture `scripts/fixtures/vectra-web/low-contrast.tokens.css` (unhappy: texto `#777` sobre `#0A0A0A` u otro par por debajo de 4,5). Superficie *glass*: `src-over` del rgba sobre `--vx-color-bg`, luego el ratio contra ese color opaco. Umbral 4,5 exactamente (texto normal AA). No se añade axe-core.

En apply, al servir la SPA (`vortex serve` con `paths.web` al clone, o abrir el HTML con CSS ya linkeado), el fondo `#0A0A0A` y el texto de `body` son lo que axe DevTools mide para el Listo cuando.

**Por qué:** el CLI de verificación del proyecto no tiene navegador. El Listo cuando nombra axe; el TDD necesita un oráculo determinista y un unhappy path (un CSS de producción que ya cumple no puede demostrar el fallo). Compositar el *glass* evita un falso AA sobre el fondo crudo cuando el texto vive sobre la superficie.

**Alternativa:** Playwright + `@axe-core/playwright`. Se descartó: `package.json`, ~el tamaño de T-09-09, fuera de ~2 h. **Alternativa:** solo documentar “correr axe a mano”. Se descartó: no hay red TDD ni unhappy path. **Alternativa:** parsear con `css` npm. Se descartó: ADR-06 / sin dependencias; un parser mínimo de `--vx-*: …;` en `:root` basta.

### 4. Paleta: literales en `:root`, valores que pasen el oráculo

**Elección:** hex u `rgba()` literales (sin `color-mix` ni `oklch`, por el parser). Candidatos a fijar en apply (ajustables si el oráculo exige otro valor; el spec solo fija `#0A0A0A` y 44 px):

| Token | Valor inicial propuesto |
|---|---|
| `--vx-color-bg` | `#0A0A0A` |
| `--vx-color-surface` | `rgba(255, 255, 255, 0.08)` |
| `--vx-color-surface-solid` | `#1C1C1E` |
| `--vx-color-text` | `#F5F5F7` |
| `--vx-color-text-muted` | `#C7C7CC` (no `#8E8E93`: arriesga AA sobre la superficie compositeda) |
| `--vx-color-accent` | `#0A84FF` |
| `--vx-color-error` | `#FF453A` |
| `--vx-color-success` | `#30D158` |
| `--vx-color-focus-ring` | `#64D2FF` |
| `--vx-color-on-accent` / `on-error` / `on-success` | `#FFFFFF` o `#0A0A0A`, el que dé ≥ 4,5:1 sobre el relleno |
| `--vx-space-1`…`4` | `4px`, `8px`, `16px`, `24px` |
| `--vx-radius-sm` / `md` | `8px`, `16px` |
| `--vx-touch-min` | `44px` |
| `--vx-surface-blur` | `20px` |

`--vx-color-on-*` no está en el **Hace** literal; se agrega para que acento/error/éxito sean superficies de texto medibles (T-01-03 pintará botones sobre esos rellenos).

**Por qué:** estética Apple dark + ferretería (acento sistema iOS). El oráculo manda: si un par falla, se aclara el texto o se oscurece el relleno, no se baja el umbral.

**Alternativa:** acento naranja “herramienta”. Se descartó: no está en el ticket; se puede retocar en un ticket de pulido. **Alternativa:** `color-mix(in srgb, …)`. Se descartó: el parser de tests se complica y Safari antiguo de tablets del piloto no es el objetivo, pero Node no evalúa `color-mix`.

### 5. Mínimo táctil en nativos, no en cada `<p>`

**Elección:** `base.css` pone `min-height` y `min-width: var(--vx-touch-min)` a `a, button, input, select, textarea`. Los *placeholders* del *router* ya tienen `<a>`: al aplicar tokens, esos enlaces pasan a 44 px sin tocar `router.js`.

**Por qué:** el **Hace** pide el *token* 44 px; el DoD de UI aplica a controles interactivos. Estirar un `<p>` rompería el layout.

**Alternativa:** solo el *token*, sin usarlo en `base.css`. Se descartó: T-01-03 cubre componentes, pero la SPA de T-01-01 ya tiene enlaces. **Alternativa:** `padding` en `html { font-size }` para que `2.75rem = 44px`. Se descartó: `44px` literal es lo que el harness puede grepear.

### 6. Extender `check-vectra-web.sh`, sin job de CI

**Elección:** el script existente exige `css/tokens.css`, `css/base.css` y los dos `<link>` versionados; al final corre `node --test scripts/fixtures/vectra-web/*.test.mjs` (hoy `router.test.mjs`; se suma `tokens.test.mjs`). Sin `package.json`, sin tocar `.github/workflows/ci.yml`.

**Por qué:** un harness por SPA. Node ya está en el runner; CI posterior puede seguir llamando el mismo script.

**Alternativa:** `scripts/check-vectra-tokens.sh`. Se descartó: dos comandos para el mismo árbol. **Alternativa:** job ahora. Se descartó: mezcla `ci-base` / oráculos de `check-ci.sh`.

## Risks / Trade-offs

- **[axe DevTools ≠ oráculo Node]** → El Listo cuando es humano sobre la SPA servida. Mitigación: `base.css` aplica los mismos pares que el oráculo; compositar el *glass* alinea la medida con lo que el ojo ve sobre la superficie. Si axe marca un nodo que no es par de la tabla (p. ej. texto del *user agent*), no es fallo de este ticket.
- **[Parser CSS frágil]** → Solo se exige `--vx-*: <valor>;` en `:root` y un `@media` con `.vx-surface`. Mitigación: spec + tests de fixture; no minificar.
- **[min-width 44 px en enlaces de texto]** → Los `<a>` del *placeholder* se ven como pastillas. Mitigación: aceptable hasta T-01-03; no cambiar el *router* para añadir clases.
- **[Safari `backdrop-filter` + transparencia reducida]** → El *media* está poco difundido; quien no lo implementa queda en *glass*. Mitigación: el ticket lo pide; el sólido también se usa si un usuario activa el setting en macOS/iOS.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-01-02` y rename a `T-01-02` (igual que T-01-01). `validate` / `status` aceptan ambos.

## Migration Plan

1. Red: harness exige `tokens.css` / *links*; `tokens.test.mjs` falla (no hay archivo o contraste).
2. Green: `tokens.css` + `base.css` + `<link>` en `index.html`; `./scripts/check-vectra-web.sh` sale 0 (router previo incluido).
3. Cierre: § 5 en `vortex-core/` (regresión); `openspec validate T-01-02`.
4. Rollback: borrar las dos hojas, revertir `index.html` y los tests nuevos. Sin datos de usuario ni OpenAPI. El *router* sigue igual.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El acento exacto se ajusta en apply si el oráculo lo exige.
