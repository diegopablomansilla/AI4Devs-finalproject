# 5. Contrato de API — Vectra

> **Estado:** Borrador v0.1 — MVP
> **Rol autor:** Arquitecto de Software
> **Documentos previos:** [1. Descripción General del Producto](1-descripcion-general-del-producto.md) · [2. Arquitectura del Sistema](2-arquitectura.md) · [3. Componentes de Vortex Core](3-componentes.md) · [4. Modelo de Datos](4-modelo.md)
> **Alcance:** API HTTP que expone el API Gateway de Vortex Core ([3-componentes.md, 3.4.1](3-componentes.md#341-api-gateway)) para el **alcance firme** del MVP. Especificación en **OpenAPI 3.1**, embebida en [5.13](#513-especificación-openapi-31) y disponible como archivo en [`openapi.yaml`](openapi.yaml).
> **Idioma objetivo:** Español (Latinoamérica) para la documentación; **inglés** para rutas, campos y valores del contrato.

---

## Índice

1. [Convenciones generales](#51-convenciones-generales)
2. [Mapa de endpoints](#52-mapa-de-endpoints)
3. [Correspondencia API ↔ modelo de datos](#53-correspondencia-api--modelo-de-datos)
4. [Errores](#54-errores)
5. [Concurrencia: bloqueo optimista](#55-concurrencia-bloqueo-optimista)
6. [Paginación](#56-paginación)
7. [Búsqueda *search-as-you-type* y cancelación](#57-búsqueda-search-as-you-type-y-cancelación)
8. [Streaming SSE del Modo IA](#58-streaming-sse-del-modo-ia)
9. [Voz](#59-voz)
10. [Límites, timeouts y comportamiento del Gateway](#510-límites-timeouts-y-comportamiento-del-gateway)
11. [Degradación vista desde la API](#511-degradación-vista-desde-la-api)
12. [Fuera de este contrato](#512-fuera-de-este-contrato)
13. [Especificación OpenAPI 3.1](#513-especificación-openapi-31)
14. [Preguntas abiertas del contrato](#514-preguntas-abiertas-del-contrato)

---

## 5.1. Convenciones generales

| Tema | Convención |
|---|---|
| **URL base** | `https://vectra.local` (mDNS, certificado de la CA local, ver [2.7](2-arquitectura.md#27-red-local-y-seguridad-mínima)). Rutas de negocio bajo `/api/v1`; `/health` y `/metrics` en la raíz. La SPA y `/media/...` se sirven como estáticos y no forman parte del contrato. |
| **Versionado** | Prefijo de ruta `/api/v1`. Agregar campos opcionales o endpoints no rompe la versión; quitar o renombrar campos, cambiar tipos o semántica exige `/api/v2`. El cliente debe ignorar campos desconocidos. |
| **Autenticación** | Ninguna en el MVP (`security: []`). Se asume una red local de confianza. |
| **Formato** | `application/json` en UTF-8; errores en `application/problem+json`; Modo IA en `text/event-stream`; audio crudo en `audio/*`; imágenes en `multipart/form-data`. |
| **Idioma del contrato** | Rutas, campos y valores enumerados en inglés, `camelCase` en campos y `snake_case` en valores enumerados. El modelo de datos está en español; la correspondencia está en [5.3](#53-correspondencia-api--modelo-de-datos). |
| **IDs** | ULID de 26 caracteres **sin** prefijo de tabla (`01J9ZKA7M3Q8R2T5V9X1Z4B6D8`, no `producto:01J9...`). El SKU es un campo y puede cambiar; las referencias siempre usan el ID. |
| **Decimales** | Precio, stock y cantidades viajan como **string decimal** (`"120.50"`, `"-2"`), igual que el tipo `decimal` del modelo; nunca como número de punto flotante. |
| **Fechas** | RFC 3339 en UTC (`2026-09-24T21:33:00Z`). |
| **Terminal** | Cabecera opcional `X-Vectra-Terminal` en las escrituras que afectan el ledger; se guarda en `movimiento_stock.terminal`. |
| **Baja** | `DELETE` sobre productos, categorías, marcas y definiciones de atributo es **baja lógica** (`active = false`); se reactiva con `PATCH { "active": true }`. `DELETE` es físico solo en imágenes y relaciones. |

---

## 5.2. Mapa de endpoints

| Método y ruta | Operación | Componente | Épica |
|---|---|---|---|
| `GET /api/v1/products` | Listar productos (admin) | Catalog & Inventory | EP-02 |
| `POST /api/v1/products` | Crear producto (con stock inicial opcional) | Catalog & Inventory | EP-02 |
| `GET /api/v1/products/{productId}` | Ficha completa | Catalog & Inventory | EP-02 |
| `PATCH /api/v1/products/{productId}` | Modificar (bloqueo optimista) | Catalog & Inventory | EP-02 |
| `DELETE /api/v1/products/{productId}` | Baja lógica | Catalog & Inventory | EP-02 |
| `GET /api/v1/products/{productId}/stock-movements` | Historial del ledger | Catalog & Inventory | EP-02 |
| `POST /api/v1/products/{productId}/stock-movements` | Ingreso o ajuste | Catalog & Inventory | EP-02 |
| `POST /api/v1/products/{productId}/images` | Subir imagen de catálogo | Catalog & Inventory | EP-02 |
| `DELETE /api/v1/products/{productId}/images/{imageId}` | Eliminar imagen | Catalog & Inventory | EP-02 |
| `GET·POST /api/v1/categories`, `PATCH·DELETE /api/v1/categories/{id}` | Árbol de categorías | Catalog & Inventory | EP-02 |
| `GET·POST /api/v1/brands`, `PATCH·DELETE /api/v1/brands/{id}` | Marcas | Catalog & Inventory | EP-02 |
| `GET·POST /api/v1/attribute-definitions`, `PATCH·DELETE .../{id}` | Atributos técnicos | Catalog & Inventory | EP-02 |
| `GET /api/v1/search` | Búsqueda por texto con facetas | Search | EP-03 |
| `GET /api/v1/search/suggestions` | Autocompletado | Search | EP-03 |
| `GET /api/v1/search/by-code/{code}` | Búsqueda exacta por SKU, fabricante u OEM | Search | EP-03 |
| `GET /api/v1/products/{productId}/recommendations` | Panel de recomendaciones | Recommendation | EP-04 |
| `GET /api/v1/products/{productId}/relations` | Relaciones de un producto (admin) | Recommendation | EP-04 |
| `POST /api/v1/relations` | Crear relación (opcionalmente bidireccional) | Recommendation | EP-04 |
| `PATCH·DELETE /api/v1/relations/{relationType}/{relationId}` | Modificar o eliminar relación | Recommendation | EP-04 |
| `POST /api/v1/ai/ask` | Pregunta en lenguaje natural (SSE) | RAG Orchestrator | EP-05 |
| `POST /api/v1/intent` | Clasificar intención | Intent Router | EP-05, EP-06 |
| `POST /api/v1/speech/transcriptions` | Transcribir audio (+ intención) | Speech + Intent Router | EP-06 |
| `GET /health` | Salud agregada | Sidecar Supervisor + Plataforma | EP-00 |
| `GET /metrics` | Métricas Prometheus | Plataforma | EP-00, EP-09 |

---

## 5.3. Correspondencia API ↔ modelo de datos

El API Gateway traduce entre el contrato (inglés) y el dominio (español, [4-modelo.md](4-modelo.md)). Esta tabla es la referencia única de esa traducción.

### Campos

| API | Modelo (SurrealDB) |
|---|---|
| `id` | Parte ULID del record ID (`producto:<ulid>`) |
| `name`, `productType`, `description` | `nombre`, `tipo`, `descripcion` |
| `brandId` / `brand` | `marca` |
| `categoryId` / `category.path` | `categoria` / `categoria.ruta` |
| `attributes[].definitionId`, `.value` | `atributos[*].def`, `atributos[*].valor` |
| `attributes[].key`, `.label`, `.unit` | `atributo_def.clave`, `.etiqueta`, `.unidad` |
| `codes[].type`, `.value` | `codigos[*].tipo`, `codigos[*].valor` |
| `unitOfMeasure` | `unidad_medida` |
| `price`, `stock` | `precio`, `stock` |
| `location.aisle`, `.shelf`, `.position` | `ubicacion.pasillo`, `.estante`, `.posicion` |
| `active`, `version`, `createdAt`, `updatedAt` | `activo`, `version`, `creado_en`, `actualizado_en` |
| `parentId`, `ancestorIds`, `path` (categoría) | `padre`, `ancestros`, `ruta` |
| `quantity`, `resultingStock`, `reason`, `saleId`, `terminal` | `cantidad`, `stock_resultante`, `motivo`, `venta`, `terminal` |
| `sourceProductId`, `targetProductId` (relación) | `in`, `out` |
| `priority`, `note`, `bidirectionalGroup` | `prioridad`, `nota`, `grupo_bidireccional` |
| `facetable`, `searchable`, `options`, `order` | `facetable`, `buscable`, `opciones`, `orden` |
| `isPrimary`, `origin`, `width`, `height` (imagen) | `es_principal`, `origen`, `ancho`, `alto` |

### Valores enumerados

| Campo API | API → modelo |
|---|---|
| `unitOfMeasure` | `unit`→`unidad`, `pair`→`par`, `box`→`caja`, `meter`→`metro`, `kilogram`→`kilogramo`, `liter`→`litro` |
| `codes[].type` | `manufacturer`→`fabricante`, `oem`→`oem`, `barcode`→`barras` |
| Tipo de movimiento | `initial`→`inicial`, `inbound`→`ingreso`, `adjustment`→`ajuste`, `sale`→`venta` |
| Tipo de atributo | `text`→`texto`, `number`→`numero`, `boolean`→`booleano`, `enum`→`enum` |
| Tipo de relación | `similar`→`similar_a`, `substitute`→`sustituye_a`, `complement`→`complementa` |
| `origin` (imagen) | `catalog`→`catalogo`, `operator`→`operador` |

> El valor `substitute` se lee en el sentido de la arista ("al consultar `source`, ofrecer `target` si `source` no tiene stock"), de modo que el contrato no cambia si se resuelve [PM-05](4-modelo.md#413-preguntas-abiertas-del-modelo) renombrando la tabla a `sustituible_por`.

---

## 5.4. Errores

Todos los errores usan **RFC 9457** (`application/problem+json`) con un campo extendido `code`, estable y pensado para que la SPA decida qué mostrar sin interpretar textos:

```json
{
  "type": "https://vectra.local/problems/version-conflict",
  "title": "El producto fue modificado por otra terminal",
  "status": 409,
  "code": "version_conflict",
  "detail": "Versión enviada 3, versión actual 4.",
  "instance": "/api/v1/products/01J9ZKA7M3Q8R2T5V9X1Z4B6D8"
}
```

Los errores de validación agregan `errors: [{ field, message }]`, con rutas de campo como `attributes[1].value`.

### Mapeo de errores de dominio a HTTP

La taxonomía de errores de dominio se define en `domain` ([3-componentes.md, 3.4.2](3-componentes.md#342-domain)). El API Gateway la traduce así; `api` no decide degradaciones, solo traduce:

| Error de dominio | HTTP | `code` posibles |
|---|---|---|
| `InvalidInput` | `400` | `invalid_input`, `self_relation`, `category_cycle`, `negative_stock` |
| `NotFound` | `404` | `not_found` |
| `Conflict` | `409` | `conflict`, `version_conflict`, `duplicate_sku`, `duplicate_slug`, `duplicate_relation`, `attribute_in_use` |
| Cuerpo demasiado grande | `413` | `payload_too_large` |
| Tipo de contenido no soportado | `415` | `unsupported_media_type` |
| `Unavailable` | `503` (+ `Retry-After`) | `unavailable`, `ai_unavailable`, `speech_unavailable` |
| `Timeout` | `504` | `timeout` |
| Error no previsto | `500` | `internal` |

Origen de los códigos específicos: `negative_stock` viene del `THROW "stock_negativo"` del ledger ([4.5.2](4-modelo.md#452-registro-de-un-movimiento-transacción)); `duplicate_sku`, `duplicate_slug` y `duplicate_relation`, de los índices `UNIQUE`; `category_cycle` y `self_relation`, de reglas del dominio.

---

## 5.5. Concurrencia: bloqueo optimista

Varias terminales pueden editar la misma ficha. `Product` expone `version`; `PATCH /api/v1/products/{id}` exige `version` en el cuerpo:

1. La SPA lee la ficha (`version: 3`) y envía `PATCH { "version": 3, "price": "130.00" }`.
2. Vortex Core ejecuta `UPDATE ... SET ..., version += 1 WHERE version = 3` ([4.3](4-modelo.md#43-convenciones)).
3. Si ninguna fila se actualizó → `409` con `code = version_conflict`; la SPA recarga la ficha y muestra los cambios de la otra terminal.

Los movimientos de stock **no** usan `version` ni la incrementan: una venta o un ingreso concurrente no invalida la edición de la ficha.

---

## 5.6. Paginación

| Tipo | Dónde | Mecánica |
|---|---|---|
| **Offset** | `GET /api/v1/search` | `offset` (≤ 1000) y `limit` (≤ 50); la respuesta trae `estimatedTotalHits`. Es lo que ofrece Meilisearch y lo que necesita una grilla de resultados por relevancia. |
| **Cursor** | Listados de administración (`/products`, `/stock-movements`) | `cursor` opaco y `limit` (≤ 100); la respuesta trae `nextCursor` (`null` al final). Internamente es el último ULID visto, lo que da orden estable aunque haya altas concurrentes. |

Los listados de maestros pequeños (categorías, marcas, atributos, relaciones de un producto) se devuelven completos, sin paginar.

---

## 5.7. Búsqueda *search-as-you-type* y cancelación

- La SPA invoca `GET /api/v1/search` en cada pulsación y **cancela la petición anterior con `AbortController`**. Al detectar la desconexión, Axum descarta el *future* de la petición y libera el trabajo en curso.
- Los filtros facetados se envían como parámetros: `categoryId` (incluye la rama), `brandId`, `inStock` y `attr[<clave>]=<valor>` para atributos con `facetable = true`. Ej.: `/api/v1/search?q=perno&attr[medida]=M8&inStock=true&facets=true`.
- `highlight` trae los fragmentos coincidentes envueltos en `<mark>…</mark>` (derivado de `_formatted` de Meilisearch). La SPA los inserta como HTML sanitizado, permitiendo solo `<mark>`.
- La búsqueda por código (`/search/by-code/{code}`) es exacta: un código con un carácter distinto corresponde a otra pieza.
- La configuración de ranking, sinónimos y facetas está en [4-modelo.md, 4.11](4-modelo.md#411-modelo-derivado-en-meilisearch).
- Si Meilisearch no responde, la respuesta mantiene la forma, con `degraded: true`, `degradedReason: "text_search_fallback"`, `highlight: null` y `facets: null` ([5.11](#511-degradación-vista-desde-la-api)).

---

## 5.8. Streaming SSE del Modo IA

`POST /api/v1/ai/ask` responde `text/event-stream`. Se usa `POST` en lugar de `EventSource` (que solo admite `GET` y no se puede cancelar de forma limpia): la SPA consume el stream con `fetch` + `ReadableStream` y lo cancela con `AbortController`, lo que corta la generación en Ollama.

### Eventos

| Evento | Cuándo | `data` (JSON) |
|---|---|---|
| `meta` | Primero, al terminar la recuperación | `{ requestId, candidates, model }` |
| `token` | Durante la generación | `{ text }`: texto sin marcadores internos de cita |
| `product` | Cuando el LLM cita un SKU **y** este se validó contra la base | `{ id, sku, name, stock, price, location, imageUrl }` |
| `done` | Último evento de un stream exitoso | `{ reason, elapsedMs, tokens }`, con `reason` ∈ `completed`, `no_candidates`, `length` |
| `error` | Falla después de abierto el stream (caída de Ollama, timeout) | Objeto `Problem` ([5.4](#54-errores)); cierra el stream |

Reglas del contrato (guardarraíles de [1.7.4](1-descripcion-general-del-producto.md#174-modo-ia-rag-local)):

- Un SKU citado que no existe en el catálogo **nunca** genera un evento `product`; se descarta.
- Si no hay candidatos relevantes, el servidor no invoca al LLM: emite `meta` (`candidates: 0`), un `token` con el mensaje fijo de "no tenemos productos para esa consulta" y `done` con `reason: "no_candidates"`.
- Si Ollama no está disponible **antes** de abrir el stream, la respuesta es `503` con `code = ai_unavailable` (JSON, sin stream) y la SPA vuelve a búsqueda por texto.
- No se admite reanudar un stream (sin `Last-Event-ID`): una pregunta cortada se vuelve a hacer.

---

## 5.9. Voz

`POST /api/v1/speech/transcriptions` recibe el audio **crudo** como cuerpo (`audio/webm` u `audio/ogg` con Opus, o `audio/wav` PCM 16 kHz mono), tal como lo produce `MediaRecorder` en la terminal.

- Respuesta: `rawText` (salida de Whisper), `text` (normalizado: "tres octavos" → "3/8", "eme ocho" → "M8") y, con `routeIntent=true` (defecto), `intent` con la clasificación del Intent Router. Así la SPA resuelve voz → destino en **una sola ida**.
- La SPA muestra `text` en el Omnibox, editable, y ejecuta el destino sugerido (`text_search`, `ai` o `camera`).
- El audio se procesa en memoria y no se persiste.

---

## 5.10. Límites, timeouts y comportamiento del Gateway

Comportamiento transversal del API Gateway ([3-componentes.md, 3.4.1](3-componentes.md#341-api-gateway)). Los valores son **propuestos** y se ajustan con los *benchmarks* de EP-09.

| Ruta | Límite de cuerpo | Timeout |
|---|---|---|
| JSON de ABM (general) | 256 KB | 5 s |
| `GET /api/v1/search` | — | 2 s |
| `GET /api/v1/search/suggestions` | — | 1 s |
| `POST .../images` | 10 MB | 15 s |
| `POST /api/v1/speech/transcriptions` | 2 MB y 30 s de audio | 10 s |
| `POST /api/v1/intent` | 16 KB | 1,5 s (luego cae a `direct_search`) |
| `POST /api/v1/ai/ask` | 16 KB | 10 s hasta el primer evento; 90 s en total |

Además:

- **Compresión** (gzip/brotli) en respuestas JSON; nunca en `text/event-stream`.
- **Caché:** `Cache-Control: no-store` en la API; los estáticos del *app shell* se versionan y se sirven con caché larga (el Service Worker gestiona la actualización).
- **Cancelación:** toda ruta libera el trabajo si el cliente cierra la conexión.
- **Concurrencia de inferencia:** voz y embeddings pasan por un semáforo por modelo ([DC-01](3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes)); si la espera supera el timeout de la ruta, se responde `504`.

---

## 5.11. Degradación vista desde la API

| Situación | Efecto en la API | Señal en `/health` |
|---|---|---|
| Arranque en curso | `/health` → `503` `starting`; la API de negocio puede responder `503` hasta estar lista | `status: starting` |
| Meilisearch caído | `/search` → `200` con `degraded: true` (búsqueda de respaldo en SurrealDB); `/search/suggestions` → solo `products`, sin `terms` | `meilisearch: down`, `status: degraded` |
| Ollama caído | `/ai/ask` → `503 ai_unavailable`; `/intent` → `direct_search` con `source: fallback` | `llm: down`, `status: degraded` |
| Modelo Whisper no cargado | `/speech/transcriptions` → `503 speech_unavailable` | `speech: disabled` |
| Embeddings de texto no disponibles | `/ai/ask` sigue funcionando con recuperación solo léxica (sin cambio de contrato) | `textEmbeddings: down` |
| Outbox atrasado | Sin error; los resultados de búsqueda pueden reflejar datos de hace más de 1 s | `sync.outboxLag` > 0 |
| SurrealDB no abre | Vortex Core no arranca; no hay API | — |

---

## 5.12. Fuera de este contrato

| Capacidad | Motivo | Dónde se define |
|---|---|---|
| Búsqueda por imagen (EP-07) | Condicionada | Se agregará como `POST /api/v1/search/image` al habilitar la épica |
| POS básico (EP-08) | Condicionado | Se agregará bajo `/api/v1/sales` |
| ABM de sinónimos y notas técnicas | Fuera del alcance acordado para esta versión del contrato | Modelo en [4.4.6](4-modelo.md#446-sinonimo) y [4.4.7](4-modelo.md#447-nota_tecnica) |
| Reindexación, backup, dataset semilla, certificados | Operación por CLI, no por HTTP | `vortex reindex`, `vortex backup`, `vortex seed`, `vortex cert` ([3.4.13](3-componentes.md#3413-plataforma-binvortex)) |
| Archivos de imagen (`/media/...`) y la SPA | Estáticos | API Gateway |

---

## 5.13. Especificación OpenAPI 3.1

Copia idéntica de [`openapi.yaml`](openapi.yaml), validada con Redocly CLI (`npx @redocly/cli lint docs/openapi.yaml`). Ante cualquier cambio, se edita `openapi.yaml` y se actualiza este bloque.

<!-- OPENAPI:BEGIN -->

```yaml
openapi: 3.1.0
info:
  title: Vectra — Vortex Core API
  version: 0.1.0
  summary: API local de Vortex Core consumida por la SPA Vectra en las terminales de la LAN.
  description: |
    Contrato de las APIs principales del MVP (alcance firme). Documento explicativo:
    `docs/5-contract-api.md`. Esta especificación y la copia embebida en ese documento
    deben mantenerse idénticas.

    - Sin autenticación en el MVP (red local de confianza, ver 2.7).
    - Montos, precios y cantidades de stock se transmiten como **string decimal** (nunca float).
    - IDs: ULID de 26 caracteres, sin el prefijo de tabla de SurrealDB.
    - Errores: `application/problem+json` (RFC 9457) con el campo extendido `code`.
servers:
  - url: https://vectra.local
    description: Nodo Maestro anunciado por mDNS (certificado emitido por la CA local)
security: []
tags:
  - name: Products
    description: ABM de productos (Catalog & Inventory Service, EP-02)
  - name: Categories
    description: Árbol de categorías (EP-02)
  - name: Brands
    description: Marcas (EP-02)
  - name: AttributeDefinitions
    description: Definición de atributos técnicos tipados (EP-02)
  - name: Stock
    description: Ledger de movimientos de stock (EP-02)
  - name: Images
    description: Imágenes de producto (EP-02)
  - name: Search
    description: Búsqueda por texto, sugerencias y código (Search Service, EP-03)
  - name: Recommendations
    description: Recomendaciones y relaciones de grafo (Recommendation Service, EP-04)
  - name: AI
    description: Modo IA (RAG Orchestrator) y ruteo de intención (EP-05, EP-06)
  - name: Speech
    description: Transcripción de voz (Speech Service, EP-06)
  - name: Operations
    description: Salud y métricas (EP-00)

paths:
  # ───────────────────────────── Products ─────────────────────────────
  /api/v1/products:
    get:
      tags: [Products]
      operationId: listProducts
      summary: Listar productos (administración)
      parameters:
        - $ref: '#/components/parameters/Cursor'
        - $ref: '#/components/parameters/Limit'
        - name: categoryId
          in: query
          description: Filtra por la categoría y todas sus descendientes.
          schema: { $ref: '#/components/schemas/Ulid' }
        - name: brandId
          in: query
          schema: { $ref: '#/components/schemas/Ulid' }
        - name: active
          in: query
          description: '`true` (defecto) solo activos; `false` solo dados de baja.'
          schema: { type: boolean, default: true }
      responses:
        '200':
          description: Página de productos ordenada por ID (ULID, orden de alta).
          content:
            application/json:
              schema: { $ref: '#/components/schemas/ProductSummaryPage' }
        '400': { $ref: '#/components/responses/BadRequest' }
    post:
      tags: [Products]
      operationId: createProduct
      summary: Crear producto
      parameters:
        - $ref: '#/components/parameters/Terminal'
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/ProductCreate' }
      responses:
        '201':
          description: Producto creado. Si se envió `initialStock`, se registró un movimiento `initial`.
          headers:
            Location:
              schema: { type: string }
              description: URL del producto creado.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Product' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '409': { $ref: '#/components/responses/Conflict' }

  /api/v1/products/{productId}:
    parameters:
      - $ref: '#/components/parameters/ProductId'
    get:
      tags: [Products]
      operationId: getProduct
      summary: Obtener ficha completa de un producto
      responses:
        '200':
          description: Ficha del producto.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Product' }
        '404': { $ref: '#/components/responses/NotFound' }
    patch:
      tags: [Products]
      operationId: updateProduct
      summary: Modificar producto (bloqueo optimista)
      description: |
        Actualización parcial: solo se modifican los campos enviados; los arrays
        (`attributes`, `codes`) se reemplazan completos. `version` es obligatorio y debe
        coincidir con la versión actual; si no coincide → `409` con `code = version_conflict`.
        `stock` no es editable (se modifica solo por el ledger).
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/ProductUpdate' }
      responses:
        '200':
          description: Producto actualizado, con `version` incrementada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Product' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }
        '409': { $ref: '#/components/responses/Conflict' }
    delete:
      tags: [Products]
      operationId: deactivateProduct
      summary: Dar de baja un producto (baja lógica)
      description: Marca `active = false`. Se conservan ledger, relaciones e imágenes; el producto deja de aparecer en la búsqueda.
      responses:
        '204': { description: Producto dado de baja. }
        '404': { $ref: '#/components/responses/NotFound' }

  /api/v1/search/by-code/{code}:
    get:
      tags: [Search]
      operationId: findProductsByCode
      summary: Búsqueda exacta por código (SKU, fabricante u OEM)
      description: |
        Coincidencia exacta, sin tolerancia a errores. Puede devolver varios productos
        porque los códigos OEM no son únicos.
      parameters:
        - name: code
          in: path
          required: true
          schema: { type: string, minLength: 1, maxLength: 64 }
      responses:
        '200':
          description: Productos activos cuyo SKU o alguno de sus códigos coincide.
          content:
            application/json:
              schema:
                type: object
                required: [items]
                properties:
                  items:
                    type: array
                    items: { $ref: '#/components/schemas/ProductSummary' }

  # ───────────────────────────── Stock ─────────────────────────────
  /api/v1/products/{productId}/stock-movements:
    parameters:
      - $ref: '#/components/parameters/ProductId'
    get:
      tags: [Stock]
      operationId: listStockMovements
      summary: Historial de movimientos de stock de un producto
      parameters:
        - $ref: '#/components/parameters/Cursor'
        - $ref: '#/components/parameters/Limit'
      responses:
        '200':
          description: Movimientos del más reciente al más antiguo.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/StockMovementPage' }
        '404': { $ref: '#/components/responses/NotFound' }
    post:
      tags: [Stock]
      operationId: registerStockMovement
      summary: Registrar un ingreso o ajuste de stock
      description: |
        Agrega un movimiento inmutable al ledger y actualiza la proyección `stock` del
        producto en la misma transacción. No requiere `version`: los cambios de stock no
        compiten con la edición de la ficha. Los movimientos `initial` solo se crean con el
        alta del producto o el dataset semilla, y los `sale` solo desde el POS.
      parameters:
        - $ref: '#/components/parameters/Terminal'
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/StockMovementCreate' }
      responses:
        '201':
          description: Movimiento registrado.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/StockMovement' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }

  # ───────────────────────────── Images ─────────────────────────────
  /api/v1/products/{productId}/images:
    parameters:
      - $ref: '#/components/parameters/ProductId'
    post:
      tags: [Images]
      operationId: uploadProductImage
      summary: Subir una imagen de catálogo
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              type: object
              required: [file]
              properties:
                file:
                  type: string
                  contentMediaType: application/octet-stream
                  description: JPEG, PNG o WebP; máximo 10 MB.
                isPrimary: { type: boolean, default: false }
                order: { type: integer, minimum: 0, default: 0 }
      responses:
        '201':
          description: Imagen almacenada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/ProductImage' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }
        '409': { $ref: '#/components/responses/Conflict' }
        '413': { $ref: '#/components/responses/PayloadTooLarge' }
        '415': { $ref: '#/components/responses/UnsupportedMediaType' }

  /api/v1/products/{productId}/images/{imageId}:
    parameters:
      - $ref: '#/components/parameters/ProductId'
      - name: imageId
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Ulid' }
    delete:
      tags: [Images]
      operationId: deleteProductImage
      summary: Eliminar una imagen de producto
      responses:
        '204': { description: Imagen eliminada. }
        '404': { $ref: '#/components/responses/NotFound' }

  # ───────────────────────────── Categories ─────────────────────────────
  /api/v1/categories:
    get:
      tags: [Categories]
      operationId: listCategories
      summary: Listar categorías (lista plana; el cliente arma el árbol con `parentId`)
      parameters:
        - $ref: '#/components/parameters/IncludeInactive'
      responses:
        '200':
          description: Todas las categorías.
          content:
            application/json:
              schema:
                type: object
                required: [items]
                properties:
                  items:
                    type: array
                    items: { $ref: '#/components/schemas/Category' }
    post:
      tags: [Categories]
      operationId: createCategory
      summary: Crear categoría
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/CategoryCreate' }
      responses:
        '201':
          description: Categoría creada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Category' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '409': { $ref: '#/components/responses/Conflict' }

  /api/v1/categories/{categoryId}:
    parameters:
      - name: categoryId
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Ulid' }
    patch:
      tags: [Categories]
      operationId: updateCategory
      summary: Renombrar o mover una categoría
      description: |
        Mover o renombrar recalcula `ancestorIds` y `path` de toda la rama y reindexa sus
        productos. Mover una categoría debajo de una descendiente → `400` con
        `code = category_cycle`.
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/CategoryUpdate' }
      responses:
        '200':
          description: Categoría actualizada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Category' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }
        '409': { $ref: '#/components/responses/Conflict' }
    delete:
      tags: [Categories]
      operationId: deactivateCategory
      summary: Dar de baja una categoría (baja lógica)
      responses:
        '204': { description: Categoría dada de baja. }
        '404': { $ref: '#/components/responses/NotFound' }

  # ───────────────────────────── Brands ─────────────────────────────
  /api/v1/brands:
    get:
      tags: [Brands]
      operationId: listBrands
      summary: Listar marcas
      parameters:
        - name: q
          in: query
          description: Filtro por prefijo del nombre.
          schema: { type: string, maxLength: 100 }
        - $ref: '#/components/parameters/IncludeInactive'
      responses:
        '200':
          description: Marcas ordenadas por nombre.
          content:
            application/json:
              schema:
                type: object
                required: [items]
                properties:
                  items:
                    type: array
                    items: { $ref: '#/components/schemas/Brand' }
    post:
      tags: [Brands]
      operationId: createBrand
      summary: Crear marca
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/BrandCreate' }
      responses:
        '201':
          description: Marca creada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Brand' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '409': { $ref: '#/components/responses/Conflict' }

  /api/v1/brands/{brandId}:
    parameters:
      - name: brandId
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Ulid' }
    patch:
      tags: [Brands]
      operationId: updateBrand
      summary: Modificar marca
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/BrandUpdate' }
      responses:
        '200':
          description: Marca actualizada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Brand' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }
        '409': { $ref: '#/components/responses/Conflict' }
    delete:
      tags: [Brands]
      operationId: deactivateBrand
      summary: Dar de baja una marca (baja lógica)
      responses:
        '204': { description: Marca dada de baja. }
        '404': { $ref: '#/components/responses/NotFound' }

  # ───────────────────────────── Attribute definitions ─────────────────────────────
  /api/v1/attribute-definitions:
    get:
      tags: [AttributeDefinitions]
      operationId: listAttributeDefinitions
      summary: Listar definiciones de atributos técnicos
      parameters:
        - $ref: '#/components/parameters/IncludeInactive'
      responses:
        '200':
          description: Definiciones ordenadas por `order`.
          content:
            application/json:
              schema:
                type: object
                required: [items]
                properties:
                  items:
                    type: array
                    items: { $ref: '#/components/schemas/AttributeDefinition' }
    post:
      tags: [AttributeDefinitions]
      operationId: createAttributeDefinition
      summary: Crear definición de atributo
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/AttributeDefinitionCreate' }
      responses:
        '201':
          description: Definición creada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/AttributeDefinition' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '409': { $ref: '#/components/responses/Conflict' }

  /api/v1/attribute-definitions/{attributeId}:
    parameters:
      - name: attributeId
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Ulid' }
    patch:
      tags: [AttributeDefinitions]
      operationId: updateAttributeDefinition
      summary: Modificar definición de atributo
      description: |
        Cambiar `key` cuando algún producto ya usa el atributo → `409 attribute_in_use`.
        Cambiar `type` o `options` de forma incompatible con valores existentes → `409 attribute_in_use`.
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/AttributeDefinitionUpdate' }
      responses:
        '200':
          description: Definición actualizada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/AttributeDefinition' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }
        '409': { $ref: '#/components/responses/Conflict' }
    delete:
      tags: [AttributeDefinitions]
      operationId: deactivateAttributeDefinition
      summary: Dar de baja una definición de atributo (baja lógica)
      responses:
        '204': { description: Definición dada de baja. }
        '404': { $ref: '#/components/responses/NotFound' }

  # ───────────────────────────── Search ─────────────────────────────
  /api/v1/search:
    get:
      tags: [Search]
      operationId: searchProducts
      summary: Búsqueda por texto (search-as-you-type)
      description: |
        Búsqueda tolerante a errores tipográficos, con sinónimos, resaltado y facetas.
        Pensada para invocarse en cada pulsación: el cliente cancela la petición anterior
        con `AbortController` y el servidor libera el trabajo en curso.
        Si Meilisearch no está disponible, responde con la búsqueda de respaldo de SurrealDB
        (prefijo, sin tolerancia a errores, sin facetas ni resaltado) y `degraded = true`.
      parameters:
        - name: q
          in: query
          description: Texto de búsqueda. Vacío permite explorar solo con filtros.
          schema: { type: string, maxLength: 200, default: '' }
        - name: categoryId
          in: query
          description: Filtra por la rama de la categoría (incluye descendientes).
          schema: { $ref: '#/components/schemas/Ulid' }
        - name: brandId
          in: query
          schema: { $ref: '#/components/schemas/Ulid' }
        - name: inStock
          in: query
          description: '`true` solo con stock; `false` solo sin stock; ausente, ambos.'
          schema: { type: boolean }
        - name: attr
          in: query
          style: deepObject
          explode: true
          description: 'Filtros por atributos facetables, por `key`. Ej.: `attr[medida]=M8&attr[grado]=5`.'
          schema:
            type: object
            additionalProperties: { type: string }
        - name: sort
          in: query
          schema:
            type: string
            enum: [relevance, price_asc, price_desc, stock_desc]
            default: relevance
        - name: offset
          in: query
          schema: { type: integer, minimum: 0, maximum: 1000, default: 0 }
        - name: limit
          in: query
          schema: { type: integer, minimum: 1, maximum: 50, default: 20 }
        - name: facets
          in: query
          description: Incluir la distribución de facetas en la respuesta.
          schema: { type: boolean, default: false }
      responses:
        '200':
          description: Resultados de la búsqueda.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/SearchResponse' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '504': { $ref: '#/components/responses/Timeout' }

  /api/v1/search/suggestions:
    get:
      tags: [Search]
      operationId: suggest
      summary: Autocompletado de términos y productos
      parameters:
        - name: q
          in: query
          required: true
          schema: { type: string, minLength: 1, maxLength: 100 }
        - name: limit
          in: query
          schema: { type: integer, minimum: 1, maximum: 10, default: 5 }
      responses:
        '200':
          description: Sugerencias.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/SuggestionResponse' }
        '400': { $ref: '#/components/responses/BadRequest' }

  # ───────────────────────────── Recommendations & relations ─────────────────────────────
  /api/v1/products/{productId}/recommendations:
    parameters:
      - $ref: '#/components/parameters/ProductId'
    get:
      tags: [Recommendations]
      operationId: getRecommendations
      summary: Panel de recomendaciones de un producto
      description: |
        Resuelve los tres bloques en una sola consulta al grafo. Solo incluye destinos
        activos; los sustitutos se filtran por `stock > 0`. `highlightSubstitutes` es `true`
        cuando el producto consultado tiene `stock = 0`.
      responses:
        '200':
          description: Recomendaciones ordenadas por prioridad (1 = más alta).
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Recommendations' }
        '404': { $ref: '#/components/responses/NotFound' }

  /api/v1/products/{productId}/relations:
    parameters:
      - $ref: '#/components/parameters/ProductId'
    get:
      tags: [Recommendations]
      operationId: listProductRelations
      summary: Listar relaciones de un producto (administración)
      description: A diferencia de `/recommendations`, incluye destinos inactivos y sin stock.
      parameters:
        - name: type
          in: query
          schema: { $ref: '#/components/schemas/RelationType' }
        - name: direction
          in: query
          schema:
            type: string
            enum: [outgoing, incoming]
            default: outgoing
      responses:
        '200':
          description: Relaciones del producto.
          content:
            application/json:
              schema:
                type: object
                required: [items]
                properties:
                  items:
                    type: array
                    items: { $ref: '#/components/schemas/Relation' }
        '404': { $ref: '#/components/responses/NotFound' }

  /api/v1/relations:
    post:
      tags: [Recommendations]
      operationId: createRelation
      summary: Crear una relación (opcionalmente bidireccional)
      description: |
        Si `bidirectional = true` se crean dos aristas en una transacción, con el mismo
        `bidirectionalGroup`. Relación hacia sí mismo → `400 self_relation`; duplicada →
        `409 duplicate_relation`; producto inexistente → `404`.
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/RelationCreate' }
      responses:
        '201':
          description: Arista(s) creada(s).
          content:
            application/json:
              schema:
                type: object
                required: [items]
                properties:
                  items:
                    type: array
                    minItems: 1
                    maxItems: 2
                    items: { $ref: '#/components/schemas/Relation' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }
        '409': { $ref: '#/components/responses/Conflict' }

  /api/v1/relations/{relationType}/{relationId}:
    parameters:
      - name: relationType
        in: path
        required: true
        schema: { $ref: '#/components/schemas/RelationType' }
      - name: relationId
        in: path
        required: true
        schema: { $ref: '#/components/schemas/Ulid' }
    patch:
      tags: [Recommendations]
      operationId: updateRelation
      summary: Modificar prioridad o nota de una relación
      description: Si la arista pertenece a un grupo bidireccional, el cambio se aplica a ambas.
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/RelationUpdate' }
      responses:
        '200':
          description: Arista(s) actualizada(s).
          content:
            application/json:
              schema:
                type: object
                required: [items]
                properties:
                  items:
                    type: array
                    items: { $ref: '#/components/schemas/Relation' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '404': { $ref: '#/components/responses/NotFound' }
    delete:
      tags: [Recommendations]
      operationId: deleteRelation
      summary: Eliminar una relación
      description: Si la arista pertenece a un grupo bidireccional, se eliminan ambas.
      responses:
        '204': { description: Relación eliminada. }
        '404': { $ref: '#/components/responses/NotFound' }

  # ───────────────────────────── AI ─────────────────────────────
  /api/v1/ai/ask:
    post:
      tags: [AI]
      operationId: askAi
      summary: Pregunta en lenguaje natural (Modo IA, respuesta en streaming SSE)
      description: |
        Respuesta como `text/event-stream`. Se usa `POST` (no `EventSource`): el cliente
        consume el stream con `fetch` + `ReadableStream` y lo cancela con `AbortController`.
        Eventos: `meta`, `token`, `product`, `done`, `error` (esquemas `AiMetaEvent`,
        `AiTokenEvent`, `AiProductEvent`, `AiDoneEvent` y `Problem`).
        Si el LLM no está disponible, responde de inmediato `503` con `code = ai_unavailable`
        (sin abrir el stream).
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/AskRequest' }
      responses:
        '200':
          description: Stream de eventos SSE.
          content:
            text/event-stream:
              schema:
                type: string
                description: Secuencia de eventos `event:` / `data:` (JSON) separados por línea en blanco.
              example: |
                event: meta
                data: {"requestId":"01JA2B3C4D5E6F7G8H9J0K1M2N","candidates":6,"model":"qwen2.5:7b-instruct-q4_K_M"}

                event: token
                data: {"text":"Para un taco de 8 en hormigón usá una broca de widia de 8 mm. "}

                event: product
                data: {"id":"01J9ZKC7M3Q8R2T5V9X1Z4B6D8","sku":"BRO-W8-150","name":"Broca widia 8 x 150 mm","stock":"12","price":"3450.00","location":"P2 · E-A · 04"}

                event: done
                data: {"reason":"completed","elapsedMs":4210,"tokens":96}
          x-vectra-sse-events:
            meta: { $ref: '#/components/schemas/AiMetaEvent' }
            token: { $ref: '#/components/schemas/AiTokenEvent' }
            product: { $ref: '#/components/schemas/AiProductEvent' }
            done: { $ref: '#/components/schemas/AiDoneEvent' }
            error: { $ref: '#/components/schemas/Problem' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '503': { $ref: '#/components/responses/Unavailable' }

  /api/v1/intent:
    post:
      tags: [AI]
      operationId: classifyIntent
      summary: Clasificar la intención de una consulta del Omnibox
      description: |
        Primero reglas locales; si el resultado es ambiguo, clasificador LLM con timeout corto.
        Si el LLM no responde, devuelve `direct_search` con `source = fallback` (nunca `503`).
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [query]
              properties:
                query: { type: string, minLength: 1, maxLength: 500 }
      responses:
        '200':
          description: Intención detectada.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/IntentResult' }
        '400': { $ref: '#/components/responses/BadRequest' }

  # ───────────────────────────── Speech ─────────────────────────────
  /api/v1/speech/transcriptions:
    post:
      tags: [Speech]
      operationId: transcribe
      summary: Transcribir audio de la terminal (y opcionalmente rutear la intención)
      description: |
        El cuerpo es el audio crudo. Máximo 2 MB y 30 s. El audio se procesa en memoria y no
        se persiste.
      parameters:
        - name: routeIntent
          in: query
          description: Si es `true`, incluye la clasificación de intención de la transcripción.
          schema: { type: boolean, default: true }
      requestBody:
        required: true
        content:
          audio/webm:
            schema: { type: string, contentMediaType: audio/webm }
          audio/ogg:
            schema: { type: string, contentMediaType: audio/ogg }
          audio/wav:
            schema:
              type: string
              contentMediaType: audio/wav
              description: PCM 16 kHz mono.
      responses:
        '200':
          description: Transcripción.
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Transcription' }
        '400': { $ref: '#/components/responses/BadRequest' }
        '413': { $ref: '#/components/responses/PayloadTooLarge' }
        '415': { $ref: '#/components/responses/UnsupportedMediaType' }
        '503': { $ref: '#/components/responses/Unavailable' }
        '504': { $ref: '#/components/responses/Timeout' }

  # ───────────────────────────── Operations ─────────────────────────────
  /health:
    get:
      tags: [Operations]
      operationId: getHealth
      summary: Salud agregada de Vortex Core y sus componentes
      responses:
        '200':
          description: Operativo (`ok`) o degradado (`degraded`).
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Health' }
        '503':
          description: Arrancando (`starting`) o sin fuente de verdad (`down`).
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Health' }

  /metrics:
    get:
      tags: [Operations]
      operationId: getMetrics
      summary: Métricas en formato de exposición Prometheus
      responses:
        '200':
          description: Latencias p50/p95 por operación, atraso del outbox, eventos fallidos, estado de sidecars.
          content:
            text/plain:
              schema: { type: string }

components:
  parameters:
    ProductId:
      name: productId
      in: path
      required: true
      schema: { $ref: '#/components/schemas/Ulid' }
    Cursor:
      name: cursor
      in: query
      description: Valor `nextCursor` de la página anterior. Ausente = primera página.
      schema: { type: string }
    Limit:
      name: limit
      in: query
      schema: { type: integer, minimum: 1, maximum: 100, default: 50 }
    IncludeInactive:
      name: includeInactive
      in: query
      schema: { type: boolean, default: false }
    Terminal:
      name: X-Vectra-Terminal
      in: header
      description: Identificador de la terminal que origina la operación; se guarda en el ledger.
      schema: { type: string, maxLength: 64 }

  responses:
    BadRequest:
      description: Entrada inválida (`invalid_input` u otro código específico).
      content:
        application/problem+json:
          schema: { $ref: '#/components/schemas/Problem' }
    NotFound:
      description: Recurso inexistente (`not_found`).
      content:
        application/problem+json:
          schema: { $ref: '#/components/schemas/Problem' }
    Conflict:
      description: Conflicto con el estado actual (`conflict`, `version_conflict`, `duplicate_sku`, ...).
      content:
        application/problem+json:
          schema: { $ref: '#/components/schemas/Problem' }
    PayloadTooLarge:
      description: El cuerpo supera el límite de la ruta (`payload_too_large`).
      content:
        application/problem+json:
          schema: { $ref: '#/components/schemas/Problem' }
    UnsupportedMediaType:
      description: Tipo de contenido no soportado (`unsupported_media_type`).
      content:
        application/problem+json:
          schema: { $ref: '#/components/schemas/Problem' }
    Unavailable:
      description: Capacidad degradada o motor no disponible (`ai_unavailable`, `speech_unavailable`, `unavailable`).
      headers:
        Retry-After:
          schema: { type: integer }
          description: Segundos sugeridos antes de reintentar.
      content:
        application/problem+json:
          schema: { $ref: '#/components/schemas/Problem' }
    Timeout:
      description: La operación superó el timeout de la ruta (`timeout`).
      content:
        application/problem+json:
          schema: { $ref: '#/components/schemas/Problem' }

  schemas:
    # ── Tipos base ──
    Ulid:
      type: string
      pattern: '^[0-9A-HJKMNP-TV-Z]{26}$'
      examples: ['01J9ZKA7M3Q8R2T5V9X1Z4B6D8']
    Decimal:
      type: string
      description: Número decimal serializado como string, con signo opcional y hasta 4 decimales.
      pattern: '^-?\d{1,12}(\.\d{1,4})?$'
      examples: ['120.50', '-3']
    NonNegativeDecimal:
      type: string
      pattern: '^\d{1,12}(\.\d{1,4})?$'
      examples: ['34', '120.50']
    Timestamp:
      type: string
      format: date-time
      description: RFC 3339 en UTC.

    Problem:
      type: object
      description: Error RFC 9457 extendido con `code` (código estable de dominio).
      required: [type, title, status, code]
      properties:
        type: { type: string, format: uri-reference, examples: ['https://vectra.local/problems/version-conflict'] }
        title: { type: string, examples: ['El producto fue modificado por otra terminal'] }
        status: { type: integer, examples: [409] }
        detail: { type: string }
        instance: { type: string, examples: ['/api/v1/products/01J9ZKA7M3Q8R2T5V9X1Z4B6D8'] }
        code: { $ref: '#/components/schemas/ErrorCode' }
        errors:
          type: array
          description: Detalle por campo en errores de validación.
          items:
            type: object
            required: [field, message]
            properties:
              field: { type: string, examples: ['attributes[1].value'] }
              message: { type: string }
    ErrorCode:
      type: string
      enum:
        - invalid_input
        - not_found
        - conflict
        - version_conflict
        - duplicate_sku
        - duplicate_slug
        - duplicate_relation
        - self_relation
        - category_cycle
        - attribute_in_use
        - negative_stock
        - payload_too_large
        - unsupported_media_type
        - unavailable
        - ai_unavailable
        - speech_unavailable
        - timeout
        - internal

    # ── Catálogo ──
    UnitOfMeasure:
      type: string
      enum: [unit, pair, box, meter, kilogram, liter]
    CodeType:
      type: string
      enum: [manufacturer, oem, barcode]
      description: '`barcode` reservado para el lector de código de barras post-MVP.'
    ProductCode:
      type: object
      required: [type, value]
      properties:
        type: { $ref: '#/components/schemas/CodeType' }
        value: { type: string, minLength: 1, maxLength: 64 }
    AttributeValueInput:
      type: object
      required: [definitionId, value]
      properties:
        definitionId: { $ref: '#/components/schemas/Ulid' }
        value:
          description: Debe coincidir con el `type` de la definición (y con `options` si es `enum`).
          oneOf:
            - type: string
            - type: number
            - type: boolean
    AttributeValue:
      allOf:
        - $ref: '#/components/schemas/AttributeValueInput'
        - type: object
          required: [key, label]
          properties:
            key: { type: string, examples: ['medida'] }
            label: { type: string, examples: ['Medida'] }
            unit: { type: [string, 'null'], examples: ['mm'] }
    Location:
      type: object
      properties:
        aisle: { type: [string, 'null'], examples: ['3'] }
        shelf: { type: [string, 'null'], examples: ['B'] }
        position: { type: [string, 'null'], examples: ['12'] }
    BrandRef:
      type: object
      required: [id, name]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        name: { type: string }
    CategoryRef:
      type: object
      required: [id, name, path]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        name: { type: string }
        path:
          type: array
          items: { type: string }
          examples: [['Fijaciones', 'Pernos']]

    ProductCreate:
      type: object
      required: [sku, name, productType, categoryId, price]
      properties:
        sku: { type: string, minLength: 1, maxLength: 64 }
        name: { type: string, minLength: 1, maxLength: 200 }
        productType: { type: string, minLength: 1, maxLength: 60, description: 'Sustantivo principal ("Perno", "Arandela"); pesa en el ranking.' }
        description: { type: [string, 'null'], maxLength: 4000 }
        brandId: { oneOf: [{ $ref: '#/components/schemas/Ulid' }, { type: 'null' }] }
        categoryId: { $ref: '#/components/schemas/Ulid' }
        attributes:
          type: array
          items: { $ref: '#/components/schemas/AttributeValueInput' }
          default: []
        codes:
          type: array
          items: { $ref: '#/components/schemas/ProductCode' }
          default: []
        unitOfMeasure: { allOf: [{ $ref: '#/components/schemas/UnitOfMeasure' }], default: unit }
        price: { $ref: '#/components/schemas/NonNegativeDecimal' }
        location: { $ref: '#/components/schemas/Location' }
        initialStock:
          allOf: [{ $ref: '#/components/schemas/NonNegativeDecimal' }]
          description: Si es mayor que 0, registra un movimiento `initial` en la misma transacción.
      example:
        sku: PER-M8X40-ZN
        name: Perno hexagonal M8 x 40 zincado
        productType: Perno
        brandId: 01J9ZK4Q6Y3M2V8W0XAB1CD2EF
        categoryId: 01J9ZK3T2N5R7P9Q1S3V5W7Y9A
        attributes:
          - { definitionId: 01J9ZK1A0000000000000000AA, value: M8 }
          - { definitionId: 01J9ZK1B0000000000000000BB, value: 40 }
        codes:
          - { type: manufacturer, value: HX-0840Z }
        unitOfMeasure: unit
        price: '120.50'
        location: { aisle: '3', shelf: B, position: '12' }
        initialStock: '34'
    ProductUpdate:
      type: object
      required: [version]
      minProperties: 2
      properties:
        version: { type: integer, minimum: 1, description: Versión leída por el cliente (bloqueo optimista). }
        sku: { type: string, minLength: 1, maxLength: 64 }
        name: { type: string, minLength: 1, maxLength: 200 }
        productType: { type: string, minLength: 1, maxLength: 60 }
        description: { type: [string, 'null'], maxLength: 4000 }
        brandId: { oneOf: [{ $ref: '#/components/schemas/Ulid' }, { type: 'null' }] }
        categoryId: { $ref: '#/components/schemas/Ulid' }
        attributes:
          type: array
          items: { $ref: '#/components/schemas/AttributeValueInput' }
        codes:
          type: array
          items: { $ref: '#/components/schemas/ProductCode' }
        unitOfMeasure: { $ref: '#/components/schemas/UnitOfMeasure' }
        price: { $ref: '#/components/schemas/NonNegativeDecimal' }
        location: { $ref: '#/components/schemas/Location' }
        active: { type: boolean, description: 'Permite reactivar un producto dado de baja.' }
    Product:
      type: object
      required: [id, sku, name, productType, category, attributes, codes, unitOfMeasure, price, stock, active, version, images, createdAt, updatedAt]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        sku: { type: string }
        name: { type: string }
        productType: { type: string }
        description: { type: [string, 'null'] }
        brand: { oneOf: [{ $ref: '#/components/schemas/BrandRef' }, { type: 'null' }] }
        category: { $ref: '#/components/schemas/CategoryRef' }
        attributes:
          type: array
          items: { $ref: '#/components/schemas/AttributeValue' }
        codes:
          type: array
          items: { $ref: '#/components/schemas/ProductCode' }
        unitOfMeasure: { $ref: '#/components/schemas/UnitOfMeasure' }
        price: { $ref: '#/components/schemas/NonNegativeDecimal' }
        location: { oneOf: [{ $ref: '#/components/schemas/Location' }, { type: 'null' }] }
        stock: { $ref: '#/components/schemas/NonNegativeDecimal' }
        active: { type: boolean }
        version: { type: integer, minimum: 1 }
        images:
          type: array
          items: { $ref: '#/components/schemas/ProductImage' }
        createdAt: { $ref: '#/components/schemas/Timestamp' }
        updatedAt: { $ref: '#/components/schemas/Timestamp' }
    ProductSummary:
      type: object
      required: [id, sku, name, productType, price, stock, active]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        sku: { type: string }
        name: { type: string }
        productType: { type: string }
        brand: { type: [string, 'null'] }
        categoryPath:
          type: array
          items: { type: string }
        price: { $ref: '#/components/schemas/NonNegativeDecimal' }
        stock: { $ref: '#/components/schemas/NonNegativeDecimal' }
        active: { type: boolean }
        imageUrl: { type: [string, 'null'] }
    ProductSummaryPage:
      type: object
      required: [items, nextCursor]
      properties:
        items:
          type: array
          items: { $ref: '#/components/schemas/ProductSummary' }
        nextCursor: { type: [string, 'null'], description: '`null` si no hay más páginas.' }

    ProductImage:
      type: object
      required: [id, url, mime, width, height, origin, isPrimary, order, createdAt]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        url: { type: string, examples: ['/media/productos/01J9ZKA7M3Q8R2T5V9X1Z4B6D8/3f2a9c.jpg'] }
        mime: { type: string, enum: [image/jpeg, image/png, image/webp] }
        width: { type: integer }
        height: { type: integer }
        origin: { type: string, enum: [catalog, operator] }
        isPrimary: { type: boolean }
        order: { type: integer }
        createdAt: { $ref: '#/components/schemas/Timestamp' }

    Category:
      type: object
      required: [id, name, slug, parentId, ancestorIds, path, active]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        name: { type: string }
        slug: { type: string }
        parentId: { oneOf: [{ $ref: '#/components/schemas/Ulid' }, { type: 'null' }] }
        ancestorIds:
          type: array
          items: { $ref: '#/components/schemas/Ulid' }
        path:
          type: array
          items: { type: string }
        active: { type: boolean }
    CategoryCreate:
      type: object
      required: [name]
      properties:
        name: { type: string, minLength: 1, maxLength: 100 }
        slug: { type: string, pattern: '^[a-z0-9]+(-[a-z0-9]+)*$', description: 'Si se omite, se genera desde `name`.' }
        parentId: { oneOf: [{ $ref: '#/components/schemas/Ulid' }, { type: 'null' }] }
    CategoryUpdate:
      type: object
      minProperties: 1
      properties:
        name: { type: string, minLength: 1, maxLength: 100 }
        slug: { type: string, pattern: '^[a-z0-9]+(-[a-z0-9]+)*$' }
        parentId: { oneOf: [{ $ref: '#/components/schemas/Ulid' }, { type: 'null' }] }
        active: { type: boolean }

    Brand:
      type: object
      required: [id, name, slug, active]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        name: { type: string }
        slug: { type: string }
        active: { type: boolean }
    BrandCreate:
      type: object
      required: [name]
      properties:
        name: { type: string, minLength: 1, maxLength: 100 }
        slug: { type: string, pattern: '^[a-z0-9]+(-[a-z0-9]+)*$' }
    BrandUpdate:
      type: object
      minProperties: 1
      properties:
        name: { type: string, minLength: 1, maxLength: 100 }
        slug: { type: string, pattern: '^[a-z0-9]+(-[a-z0-9]+)*$' }
        active: { type: boolean }

    AttributeType:
      type: string
      enum: [text, number, boolean, enum]
    AttributeDefinition:
      type: object
      required: [id, key, label, type, facetable, searchable, order, active]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        key: { type: string, pattern: '^[a-z][a-z0-9_]*$', examples: ['medida'] }
        label: { type: string, examples: ['Medida'] }
        type: { $ref: '#/components/schemas/AttributeType' }
        unit: { type: [string, 'null'], examples: ['mm'] }
        options:
          type: [array, 'null']
          items: { type: string }
          description: Obligatorio si `type = enum`.
        facetable: { type: boolean }
        searchable: { type: boolean }
        order: { type: integer }
        active: { type: boolean }
    AttributeDefinitionCreate:
      type: object
      required: [key, label, type]
      properties:
        key: { type: string, pattern: '^[a-z][a-z0-9_]*$' }
        label: { type: string, minLength: 1, maxLength: 60 }
        type: { $ref: '#/components/schemas/AttributeType' }
        unit: { type: [string, 'null'], maxLength: 16 }
        options:
          type: [array, 'null']
          items: { type: string }
        facetable: { type: boolean, default: false }
        searchable: { type: boolean, default: true }
        order: { type: integer, default: 0 }
    AttributeDefinitionUpdate:
      type: object
      minProperties: 1
      properties:
        key: { type: string, pattern: '^[a-z][a-z0-9_]*$' }
        label: { type: string, minLength: 1, maxLength: 60 }
        type: { $ref: '#/components/schemas/AttributeType' }
        unit: { type: [string, 'null'], maxLength: 16 }
        options:
          type: [array, 'null']
          items: { type: string }
        facetable: { type: boolean }
        searchable: { type: boolean }
        order: { type: integer }
        active: { type: boolean }

    # ── Stock ──
    StockMovementType:
      type: string
      enum: [initial, inbound, adjustment, sale]
    StockMovementCreate:
      type: object
      required: [type, quantity]
      properties:
        type:
          type: string
          enum: [inbound, adjustment]
        quantity:
          allOf: [{ $ref: '#/components/schemas/Decimal' }]
          description: Con signo y distinto de 0. `inbound` > 0; `adjustment` ±.
        reason:
          type: [string, 'null']
          maxLength: 500
          description: Obligatorio si `type = adjustment`.
      example:
        type: adjustment
        quantity: '-2'
        reason: Conteo físico del pasillo 3
    StockMovement:
      type: object
      required: [id, productId, type, quantity, resultingStock, createdAt]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        productId: { $ref: '#/components/schemas/Ulid' }
        type: { $ref: '#/components/schemas/StockMovementType' }
        quantity: { $ref: '#/components/schemas/Decimal' }
        resultingStock: { $ref: '#/components/schemas/NonNegativeDecimal' }
        reason: { type: [string, 'null'] }
        saleId: { oneOf: [{ $ref: '#/components/schemas/Ulid' }, { type: 'null' }] }
        terminal: { type: [string, 'null'] }
        createdAt: { $ref: '#/components/schemas/Timestamp' }
    StockMovementPage:
      type: object
      required: [items, nextCursor]
      properties:
        items:
          type: array
          items: { $ref: '#/components/schemas/StockMovement' }
        nextCursor: { type: [string, 'null'] }

    # ── Búsqueda ──
    SearchHit:
      type: object
      required: [id, sku, name, productType, price, unitOfMeasure, stock, inStock]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        sku: { type: string }
        name: { type: string }
        productType: { type: string }
        brand: { type: [string, 'null'] }
        categoryPath:
          type: array
          items: { type: string }
        codes:
          type: array
          items: { type: string }
        attributes:
          type: array
          items: { type: string }
          examples: [['Medida: M8', 'Largo: 40 mm', 'Grado: 5']]
        price: { $ref: '#/components/schemas/NonNegativeDecimal' }
        unitOfMeasure: { $ref: '#/components/schemas/UnitOfMeasure' }
        stock: { $ref: '#/components/schemas/NonNegativeDecimal' }
        inStock: { type: boolean }
        location: { type: [string, 'null'], examples: ['P3 · E-B · 12'] }
        imageUrl: { type: [string, 'null'] }
        highlight:
          type: [object, 'null']
          description: Fragmentos con coincidencias envueltas en `<mark>…</mark>`. `null` en modo degradado.
          properties:
            name: { type: string, examples: ['<mark>Perno</mark> hexagonal M8 x 40 zincado'] }
            description: { type: [string, 'null'] }
    FacetValue:
      type: object
      required: [value, count]
      properties:
        value: { type: string }
        count: { type: integer }
    SearchFacets:
      type: object
      properties:
        categories:
          type: array
          items:
            allOf:
              - $ref: '#/components/schemas/FacetValue'
              - type: object
                properties:
                  level: { type: integer, minimum: 0, maximum: 3 }
        brands:
          type: array
          items: { $ref: '#/components/schemas/FacetValue' }
        inStock:
          type: object
          properties:
            'true': { type: integer }
            'false': { type: integer }
        attributes:
          type: object
          description: Por `key` de atributo facetable.
          additionalProperties:
            type: array
            items: { $ref: '#/components/schemas/FacetValue' }
    SearchResponse:
      type: object
      required: [query, hits, estimatedTotalHits, offset, limit, processingTimeMs, degraded]
      properties:
        query: { type: string }
        hits:
          type: array
          items: { $ref: '#/components/schemas/SearchHit' }
        estimatedTotalHits: { type: integer }
        offset: { type: integer }
        limit: { type: integer }
        processingTimeMs: { type: integer, description: Tiempo del motor de búsqueda. }
        facets: { oneOf: [{ $ref: '#/components/schemas/SearchFacets' }, { type: 'null' }] }
        degraded: { type: boolean }
        degradedReason:
          type: [string, 'null']
          enum: [text_search_fallback, null]
    SuggestionResponse:
      type: object
      required: [terms, products]
      properties:
        terms:
          type: array
          items: { type: string }
          examples: [['perno', 'perno hexagonal', 'perno allen']]
        products:
          type: array
          items: { $ref: '#/components/schemas/ProductSummary' }

    # ── Recomendaciones y relaciones ──
    RelationType:
      type: string
      enum: [similar, substitute, complement]
      description: |
        Lectura de una arista `source → target`: "al consultar `source`, ofrecer `target`".
        `similar`: equivalente o alternativo. `substitute`: reemplaza a `source` si no tiene stock.
        `complement`: venta cruzada.
    RecommendedProduct:
      type: object
      required: [relationId, id, sku, name, stock, price, priority]
      properties:
        relationId: { $ref: '#/components/schemas/Ulid' }
        id: { $ref: '#/components/schemas/Ulid' }
        sku: { type: string }
        name: { type: string }
        stock: { $ref: '#/components/schemas/NonNegativeDecimal' }
        price: { $ref: '#/components/schemas/NonNegativeDecimal' }
        location: { type: [string, 'null'] }
        imageUrl: { type: [string, 'null'] }
        priority: { type: integer, minimum: 1 }
        note: { type: [string, 'null'] }
    Recommendations:
      type: object
      required: [productId, highlightSubstitutes, similar, substitutes, complements]
      properties:
        productId: { $ref: '#/components/schemas/Ulid' }
        highlightSubstitutes: { type: boolean }
        similar:
          type: array
          items: { $ref: '#/components/schemas/RecommendedProduct' }
        substitutes:
          type: array
          items: { $ref: '#/components/schemas/RecommendedProduct' }
        complements:
          type: array
          items: { $ref: '#/components/schemas/RecommendedProduct' }
    Relation:
      type: object
      required: [id, type, sourceProductId, targetProductId, priority, createdAt, updatedAt]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        type: { $ref: '#/components/schemas/RelationType' }
        sourceProductId: { $ref: '#/components/schemas/Ulid' }
        targetProductId: { $ref: '#/components/schemas/Ulid' }
        target: { $ref: '#/components/schemas/ProductSummary' }
        priority: { type: integer, minimum: 1 }
        note: { type: [string, 'null'] }
        bidirectionalGroup: { oneOf: [{ $ref: '#/components/schemas/Ulid' }, { type: 'null' }] }
        createdAt: { $ref: '#/components/schemas/Timestamp' }
        updatedAt: { $ref: '#/components/schemas/Timestamp' }
    RelationCreate:
      type: object
      required: [type, sourceProductId, targetProductId]
      properties:
        type: { $ref: '#/components/schemas/RelationType' }
        sourceProductId: { $ref: '#/components/schemas/Ulid' }
        targetProductId: { $ref: '#/components/schemas/Ulid' }
        priority: { type: integer, minimum: 1, default: 1 }
        note: { type: [string, 'null'], maxLength: 500 }
        bidirectional:
          type: boolean
          description: 'Si se omite: `true` para `similar`, `false` para `substitute` y `complement`.'
      example:
        type: complement
        sourceProductId: 01J9ZKA7M3Q8R2T5V9X1Z4B6D8
        targetProductId: 01J9ZKB1N4R9S3V6W0Y2A5C7E9
        priority: 1
        note: Arandela plana M8
    RelationUpdate:
      type: object
      minProperties: 1
      properties:
        priority: { type: integer, minimum: 1 }
        note: { type: [string, 'null'], maxLength: 500 }

    # ── IA ──
    AskRequest:
      type: object
      required: [question]
      properties:
        question: { type: string, minLength: 1, maxLength: 1000 }
        maxProducts: { type: integer, minimum: 1, maximum: 10, default: 10, description: Fichas de producto inyectadas como contexto. }
      example:
        question: ¿Qué broca uso para hacer un agujero para un taco de 8 en hormigón?
    AiMetaEvent:
      type: object
      description: Primer evento del stream.
      required: [requestId, candidates, model]
      properties:
        requestId: { $ref: '#/components/schemas/Ulid' }
        candidates: { type: integer, description: Productos recuperados como contexto. }
        model: { type: string }
    AiTokenEvent:
      type: object
      description: Fragmento de texto de la respuesta, sin marcadores internos de citas.
      required: [text]
      properties:
        text: { type: string }
    AiProductEvent:
      type: object
      description: Tarjeta de un producto citado, emitida solo tras validar el SKU contra la base.
      required: [id, sku, name, stock, price]
      properties:
        id: { $ref: '#/components/schemas/Ulid' }
        sku: { type: string }
        name: { type: string }
        stock: { $ref: '#/components/schemas/NonNegativeDecimal' }
        price: { $ref: '#/components/schemas/NonNegativeDecimal' }
        location: { type: [string, 'null'] }
        imageUrl: { type: [string, 'null'] }
    AiDoneEvent:
      type: object
      description: Último evento de un stream exitoso.
      required: [reason, elapsedMs]
      properties:
        reason:
          type: string
          enum: [completed, no_candidates, length]
          description: '`no_candidates`: no hubo productos relevantes y no se invocó al LLM. `length`: se alcanzó el límite de tokens.'
        elapsedMs: { type: integer }
        tokens: { type: integer }

    IntentType:
      type: string
      enum: [direct_search, technical_question, visual_search, stock_query]
    IntentResult:
      type: object
      required: [intent, normalizedQuery, target, source]
      properties:
        intent: { $ref: '#/components/schemas/IntentType' }
        normalizedQuery: { type: string, examples: ['disco de corte 115'] }
        target:
          type: string
          enum: [text_search, ai, camera]
          description: Destino sugerido para la SPA. `stock_query` se resuelve con `text_search`.
        source:
          type: string
          enum: [rules, llm, fallback]
        confidence: { type: [number, 'null'], minimum: 0, maximum: 1 }

    # ── Voz ──
    Transcription:
      type: object
      required: [rawText, text, language, audioDurationMs, processingMs]
      properties:
        rawText: { type: string, examples: ['perno eme ocho por cuarenta'] }
        text: { type: string, description: Texto normalizado (medidas y jerga)., examples: ['perno M8 x 40'] }
        language: { type: string, const: es }
        audioDurationMs: { type: integer }
        processingMs: { type: integer }
        intent: { oneOf: [{ $ref: '#/components/schemas/IntentResult' }, { type: 'null' }] }

    # ── Operación ──
    ComponentStatus:
      type: string
      enum: [up, starting, degraded, down, disabled]
    ComponentHealth:
      type: object
      required: [status]
      properties:
        status: { $ref: '#/components/schemas/ComponentStatus' }
        detail: { type: [string, 'null'] }
        latencyMs: { type: [integer, 'null'] }
    Health:
      type: object
      required: [status, version, uptimeSeconds, components]
      properties:
        status:
          type: string
          enum: [ok, degraded, starting, down]
        version: { type: string, examples: ['0.1.0'] }
        uptimeSeconds: { type: integer }
        components:
          type: object
          required: [storage, meilisearch, llm, speech, textEmbeddings, sync]
          properties:
            storage: { $ref: '#/components/schemas/ComponentHealth' }
            meilisearch: { $ref: '#/components/schemas/ComponentHealth' }
            llm: { $ref: '#/components/schemas/ComponentHealth' }
            speech: { $ref: '#/components/schemas/ComponentHealth' }
            textEmbeddings: { $ref: '#/components/schemas/ComponentHealth' }
            sync:
              allOf:
                - $ref: '#/components/schemas/ComponentHealth'
                - type: object
                  properties:
                    outboxLag: { type: integer, description: Eventos pendientes del consumidor más atrasado. }
                    failedEvents: { type: integer }
```

<!-- OPENAPI:END -->

---

## 5.14. Preguntas abiertas del contrato

| # | Pregunta | Opciones | Recomendación del Arquitecto | Decide |
|---|---|---|---|---|
| **PAPI-01** | ¿Se mantiene la copia embebida de la especificación en este documento? | **A)** Mantener ambas copias. **B)** Dejar solo `openapi.yaml` y enlazarlo. | **B** cuando empiece la implementación: una sola fuente evita divergencias; generar la vista con Redocly. | Arquitecto |
| **PAPI-02** | ¿Límites de audio (2 MB, 30 s) adecuados? | Validar con consultas reales del mostrador. | Medir en EP-06; la mayoría de las consultas dura menos de 10 s. | TL |
| **PAPI-03** | ¿Cuándo se exponen los ABM de sinónimos y notas técnicas? | **A)** En EP-03 / EP-05 como extensión de `/api/v1`. **B)** Solo por dataset semilla en el MVP. | **A** para sinónimos (el diccionario es editable según 1.7.2); notas técnicas según PM-04. | PO |
| **PAPI-04** | ¿Se genera código a partir de la especificación? | **A)** Tipos Rust (`utoipa` desde el código). **B)** Especificación primero y validación en tests. | **B** en el MVP: la especificación es el contrato y los tests de integración de `api` validan las respuestas contra ella. | TL |
