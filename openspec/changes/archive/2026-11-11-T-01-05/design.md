# Design

## Context

`develop` ya tiene el kit T-01-03/T-01-04 (`shared.js`, Shadow DOM, `adoptedStyleSheets`, overlays, `dev/components.html`, harness `check-vectra-web.sh` + `dom-stub.mjs`). ADR-06: ES Modules, sin *build*. Ver proposal.md para el porqué; la spec para el contrato de variantes, *slot* y "sin stock".

El stub no modela `<slot>` nativo ni `querySelector` genérico en el host. Los tests de tarjeta no pueden depender de jsdom ni del navegador. HU-03-05 es MVP parcial sin foto real (HU-02-09 v2); el ticket igual pide área de imagen: *placeholder* cuando no hay URL.

## Goals / Non-Goals

**Goals:**

- Primitive presentacional alineado a `SearchHit` / `RecommendationHit`, testeable en Node
- *Slot* `name` listo para T-03-16; el atributo `name` nunca se interpreta como HTML
- Catálogo de muestra con `full` y `compact` y un caso `stock="0"` (Listo cuando)

**Non-Goals:**

- `highlight.js` / allowlist de `<mark>` (T-03-16)
- Click, `aria-activedescendant` o evento de selección (T-01-10, T-03-17, T-04-08)
- Formateo de moneda, fetch de imágenes, tokens nuevos

## Decisions

### 1. Atributos kebab-case, un campo observable por `data-field`

El host expone `sku`, `name`, `brand`, `price`, `stock`, `location`, `image-url`, `variant`. En el shadow, cada dato lleva `data-field="<nombre>"` para que el stub consulte sin CSS complejo. Precios y stock se pintan como el string recibido (OpenAPI: decimal en string, nunca float).

Alternativa descartada: propiedades JS `imageUrl` / `inStock` como única API — el catálogo HTML y T-03-17 asignan atributos; `inStock` se deriva de `stock === 0`.

### 2. Cero de stock = decimal equivalente a 0

`data-out-of-stock` + texto "Sin stock" cuando el atributo `stock` parseado como decimal es 0 (`0`, `0.0`, `0.00`). Ausente o no numérico: no marca, no lanza. No se agrega atributo `in-stock`: un solo origen evita divergencia con `SearchHit.inStock`.

### 3. Compacta = recorte visual, no otro custom element

`variant="compact"` oculta `location` y el área de imagen (aunque vengan atributos). Coincide con `RecommendationHit` (sku, name, stock, price) usado por T-04-08. Un solo módulo; `hero` u otro valor cae a `full`.

Alternativa descartada: `vx-product-card-compact` — duplica estilos y el ticket pide variantes, no dos tags.

### 4. Slot `name` vs atributo `name`

Si existe un hijo con `slot="name"`, ese nodo es el título (T-03-16 pondrá HTML sanitizado). Si no, el atributo `name` se asigna con `textContent`. El componente no llama `innerHTML` con el atributo.

En el stub: `querySelector('[slot="name"]')` en el host y `querySelector('[data-field="name"]')` en el shadow. La proyección nativa del navegador se verifica a mano en `dev/components.html`.

### 5. Imagen: `img` solo con URL; si no, placeholder

`<img>` únicamente cuando `image-url` no está vacío (`alt` = nombre). Sin URL: nodo `data-field="image-placeholder"`, sin `src` vacío. La muestra puede omitir URL o usar un data-URI/SVG local; no hay fotos de producto en este ticket.

### 6. Presentacional: sin click ni `role="button"`

T-01-10 envuelve listas; T-03-17/T-04-08 deciden apertura. El host es `article` (o equivalente) con `vx-surface`. No se emite `vx-select`. El mínimo táctil de 44 px aplica a controles, no a la tarjeta entera en este change.

## Risks / Trade-offs

- **[Slot nativo vs stub]** → Tests cubren hijo `[slot="name"]` y fallback a `textContent`; el catálogo es el oráculo visual del Listo cuando.
- **[Foto en MVP parcial HU-03-05]** → El área existe; sin `image-url` hay placeholder. Fotos reales = HU-02-09 / T-02-39 (v2).
- **[Stock no numérico]** → No se marca "sin stock"; T-03-17 siempre mandará el decimal del API.
