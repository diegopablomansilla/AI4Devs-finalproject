# 5. Contrato de API — Vectra

> **Estado:** Borrador v0.1 — MVP
> **Rol autor:** Arquitecto de Software
> **Documentos previos:** [1. Descripción General del Producto](1-descripcion-general-del-producto.md) · [2. Arquitectura del Sistema](2-arquitectura.md) · [3. Componentes de Vortex Core](3-componentes.md) · [4. Modelo de Datos](4-modelo.md)
> **Alcance:** API HTTP que expone el API Gateway de Vortex Core ([3-componentes.md, 3.4.1](3-componentes.md#341-api-gateway)) para el **alcance firme** del MVP. Especificación ejecutable en **OpenAPI 3.1** ([`openapi.yaml`](openapi.yaml)); este documento es la guía narrativa ([5.13](#513-especificación-openapi-31)).
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
14. [Decisiones adoptadas del contrato](#514-decisiones-adoptadas-del-contrato)

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
| `GET·POST /api/v1/synonyms`, `PATCH·DELETE /api/v1/synonyms/{synonymId}` | Diccionario de sinónimos para búsqueda | Catalog & Inventory / Search | EP-03 |
| `GET /api/v1/products/{productId}/recommendations` | Panel de recomendaciones | Recommendation | EP-04 |
| `GET /api/v1/products/{productId}/relations` | Relaciones de un producto (admin) | Recommendation | EP-04 |
| `POST /api/v1/relations` | Crear relación (opcionalmente bidireccional) | Recommendation | EP-04 |
| `PATCH·DELETE /api/v1/relations/{relationType}/{relationId}` | Modificar o eliminar relación | Recommendation | EP-04 |
| `GET·POST /api/v1/technical-notes`, `GET·PATCH·DELETE /api/v1/technical-notes/{noteId}` | Notas técnicas para el RAG | Catalog & Inventory / RAG | EP-05 |
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
| Tipo de relación | `similar`→`similar_a`, `substitute`→`sustituible_por`, `complement`→`complementa` |
| Tipo de sinónimo | `mutual`→`mutuo`, `unidirectional`→`unidireccional` |
| Campos de sinónimo | `term`→`termino`, `equivalents`→`equivalentes` |
| Campos de nota técnica | `title`→`titulo`, `content`→`contenido`, `productIds`→`productos` |
| `origin` (imagen) | `catalog`→`catalogo`, `operator`→`operador` |

> El valor `substitute` se lee en el sentido de la arista ("al consultar `source`, ofrecer `target` si `source` no tiene stock"). En SurrealDB la tabla se llama `sustituible_por` (decisión [PM-05](7-tickets.md#72-decisiones-adoptadas)).

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
| Reindexación, backup, dataset semilla, certificados | Operación por CLI, no por HTTP | `vortex reindex`, `vortex backup`, `vortex seed`, `vortex cert` ([3.4.13](3-componentes.md#3413-plataforma-binvortex)) |
| Archivos de imagen (`/media/...`) y la SPA | Estáticos | API Gateway |

---

## 5.13. Especificación OpenAPI 3.1

**Fuente única:** [`openapi.yaml`](openapi.yaml) (decisión [PAPI-01](7-tickets.md#72-decisiones-adoptadas)). Este documento describe convenciones, mapa de endpoints y correspondencia con el modelo; la especificación ejecutable vive solo en ese archivo.

Validación local e integración en CI: `npx @redocly/cli lint docs/openapi.yaml`.


---

## 5.14. Decisiones adoptadas del contrato

Las preguntas originales del borrador quedaron cerradas al alinear el contrato con [7.2 Decisiones adoptadas](7-tickets.md#72-decisiones-adoptadas). Resumen:

| # | Decisión |
|---|---|
| **PAPI-01** | Fuente única `openapi.yaml`; este documento no embebe la especificación ([5.13](#513-especificación-openapi-31)). |
| **PAPI-02** | Límites adoptados: audio **2 MB** y **30 s**; validar en EP-06 con consultas del mostrador. |
| **PAPI-03** | ABM de sinónimos en `/api/v1/synonyms` (EP-03); notas técnicas en `/api/v1/technical-notes` (EP-05). |
| **PAPI-04** | Especificación primero; tests de `api` validan respuestas contra `openapi.yaml` (sin codegen). |
