# Design

## Context

En `develop`, T-04-05 dejó `RelationService` / `RelationGateway` (ABM) y `RelationRepo` con `list_for_product` / `list_incoming_for_product` **sin** filtrar `activo` ni `stock`. El SQL de [4.6.2](../../../docs/4-modelo.md#462-consulta-de-recomendaciones) no está implementado. OpenAPI ya declara `GET /api/v1/products/{productId}/recommendations` y `RecommendedProduct.location` como string; T-04-05 dejó el path sin registrar. `format_ubicacion` vive privado en `domain::search_document`. See proposal.md — Why.

Constraints: MVP del curso; ~2 h; **No hace HTTP** (T-04-07); DC-05 (no en `catalog`); `recommend` no depende de `storage`; esquema `0007` no cambia; HU-04-05 «Latencia» es v2.

## Goals / Non-Goals

**Goals:**

- Una consulta Surreal 4.6.2 + ubicación en `storage`
- Tipos de dominio y método de puerto
- `RecommendationService` / `RecommendationGateway` delgados para T-04-07
- Tests HU-04-05 / HU-04-06 (happy + unhappy) en `storage` y `recommend`

**Non-Goals (diseño):**

- Handlers Axum, `assert_conforms`, métrica `recommendations_duration_seconds`
- Filtrar en el servicio o reusar `list_for_product` + N+1
- Medir p95 < 20 ms
- Cambiar el ABM ni el listado admin

## Decisions

### 1. Consulta única en el adaptador, no en el servicio

**Elección:** `RelationRepo::recommendations_for_product` ejecuta **un** `RETURN { destacar_sustitutos, similares, sustitutos, complementarios }` al estilo 4.6.2, más `id` de arista y `out.ubicacion`. El servicio de `recommend` solo delega.

**Por qué:** el Hace del ticket cita 4.6.2 («una sola ida a la base») y [3.4.5](../../../docs/3-componentes.md#345-recommendation-service). `list_for_product` incluye inactivos y stock 0 a propósito (admin).

**Alternativa:** filtrar en `recommend` sobre `list_for_product`. Se descartó: tres SELECT + hidratar stock/precio/ubicación aparte; viola «consulta única». **Alternativa:** `ProductRepo::get` + grafo. Se descartó: dos idas; el `not_found` puede resolverse en la misma query (`SELECT * FROM producto WHERE id = …`).

### 2. `not_found` en storage, no vía `ProductRepo`

**Elección:** si el record `producto:<id>` no existe, el adaptador devuelve `DomainError::not_found()`. Producto existente sin aristas → vectores vacíos y `destacar_sustitutos` según su stock. `RecommendationService` no inyecta `ProductRepo`.

**Por qué:** T-04-07 mapea 404; OpenAPI lo declara. Distinguir «no existe» de «cero recomendaciones» es del adaptador. El ABM sigue usando `ProductRepo` solo en `RelationService::list`.

**Alternativa:** 200 con bloques vacíos para id desconocido. Se descartó: T-04-07 no podría cumplir 404 sin un GET extra.

### 3. Trait `RecommendationGateway` aparte de `RelationGateway`

**Elección:** `get(Id<Producto>) -> Result<Recomendaciones, DomainError>` en un trait object-safe nuevo (`Pin<Box<dyn Future…>>`, mismo molde que `RelationGateway` / `SearchGateway`). `RecommendationService<R: RelationRepo>`.

**Por qué:** DTO distinto (`Recomendaciones` vs `RelacionConDestino`); T-04-07 inyecta solo lectura. Mezclar ABM y panel inflaría `RelationGateway`.

**Alternativa:** `fn recommendations` en `RelationGateway`. Se descartó: el Gateway HTTP de relaciones no necesita la consulta. **Alternativa:** un solo `RecommendGateway` con ABM + get. Se descartó: T-04-05 ya exportó `RelationGateway`; no reabrir su superficie.

### 4. `ubicacion` string, formateo compartido

**Elección:** promover `format_ubicacion` a `pub` en `domain` (p. ej. `Ubicacion::format_linea` o `search_document::format_ubicacion`) y usarlo al mapear `out.ubicacion`. Contrato: `P3 · E-B · 12`. Ausente → `None`.

**Por qué:** el ticket pide «más ubicación»; OpenAPI `RecommendedProduct.location` es `string | null`, no el objeto `aisle/shelf/position` de la ficha. El documento de búsqueda ya formatea igual (T-03-06).

**Alternativa:** concatenar en SurrealQL. Se descartó: duplica reglas de partes vacías. **Alternativa:** objeto `Ubicacion` en el tipo de recomendaciones. Se descartó: T-04-07 tendría que formatear en HTTP; el panel necesita el string.

### 5. Sin campo de imagen en dominio

**Elección:** `ProductoRecomendado` no tiene `imagen` / `imageUrl`. T-04-07 serializa `imageUrl: null`.

**Por qué:** recableo § 7.5.1 (sin T-02-28). Evita un `Option` que siempre es `None`.

### 6. Fake en `recommend` vs Surreal en `storage`

**Elección:** tests de `recommend` extienden el `InMemoryRelationRepo` con `recommendations_for_product` que aplica los mismos filtros en memoria. Tests de `storage` (`relacion_repo.rs`) cubren el SQL real (stock vía `stock_inicial`, baja vía `set_activo`, ubicación al crear).

**Por qué:** `#[cfg(test)]` de `recommend` no abre `storage::Db` (ya es requisito). Las HU se verifican dos veces: semántica del puerto (SQL) y del caso de uso (delegación + `not_found`).

### 7. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-04-06` y rename a `T-04-06`.

## Risks / Trade-offs

- **[SQL 4.6.2 + fetch de `out` en Surreal 2.7]** → el listado admin ya hace `out.sku` / `out.activo`; el riesgo es `out.ubicacion` objeto y `out.stock` decimal. Mitigación: tests de storage con ubicación y stock distintos; si el fetch falla, el mismo patrón de `RelacionListRecord` que T-04-04.
- **[`not_found` vs bloques vacíos]** → un ULID de producto borrado no es lo mismo que un SKU sin grafo. Mitigación: escenario unhappy explícito; T-04-07 mapea 404.
- **[Duplicar filtros en el fake de recommend]** → el fake podría divergir del SQL. Mitigación: las HU de filtro viven también en `storage`; el servicio no refiltra.
- **[~2 h]** → un método + query + servicio delgado. No se divide el ticket.

## Migration Plan

N/A: proceso local-first, sin datos que migrar. El schema `0007` no cambia. Rollback = no exponer el método (rama sin merge). T-04-07 sigue sin ruta hasta su change.

## Open Questions

Ninguna que altere specs, enfoque o tareas: consulta única, `not_found`, gateway aparte y `location` string quedan cerrados arriba.
