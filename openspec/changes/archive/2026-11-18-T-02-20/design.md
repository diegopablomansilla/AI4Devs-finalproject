# Design

## Context

En `develop`, `domain` expone `Producto`, `NewProducto`, `UpdateProducto` (snapshot completo, sin `stock` ni `activo`), `FichaProducto` y `ProductRepo` (`create` / `get` / `update`). `Producto::try_new` valida atributos contra `AtributoDef` aportadas y **no** comprueba categoría/marca. `storage::ProductoRepo` persiste alta + movimiento `inicial`, hidrata la ficha y hace `update` con `version_conflict`; el esquema `record<categoria>` / `record<marca>` no distingue inactivas. `catalog` ya tiene `CategoryService`, `BrandService` y `AttributeDefService` con fachadas object-safe y fakes en memoria (`marca`/`atributo_def` públicos bajo `#[cfg(test)]`; `categoria` no). See proposal.md — Why.

Constraints: MVP del curso; ~2 h; sin HTTP (T-02-21); sin `activo` (T-02-22); sin imágenes (T-02-30); `catalog` ya puede depender de `domain` + `tokio` (dev).

## Goals / Non-Goals

**Goals:**

- `ProductService` sobre cuatro puertos: producto, categoría, marca, `AtributoDef`
- Validar refs activas y defs **antes** de `create`/`update`
- Merge de parche de ficha en `catalog`; el puerto sigue recibiendo snapshot
- Fakes en memoria + `ProductGateway` para T-02-21

**Non-Goals (diseño):**

- Cambiar `ProductRepo` / `UpdateProducto` / hidratación Surreal
- Merge en el handler HTTP (eso dejaría T-02-21 con lógica de ficha)
- `async-trait`
- Extraer un helper genérico de maestros compartido con marca/categoría

## Decisions

### 1. `ProductService<P, C, B, A>` con cuatro puertos

**Elección:** servicio genérico `P: ProductRepo`, `C: CategoryRepo`, `B: BrandRepo`, `A: AttributeDefRepo`. Métodos `create` / `get` / `update`. `get` es wrapper de `ProductRepo::get`. `create` valida refs y defs, llama `ProductRepo::create` y luego `get` para devolver `FichaProducto`.

**Por qué:** T-02-21 necesita la ficha hidratada en el 201; el puerto de `create` sigue devolviendo `Producto` (T-02-18). Cuatro puertos evitan acoplar maestros al `ProductRepo`.

**Alternativa:** un solo `ProductRepo` que valide categoría/marca. Se descartó: contradice vortex-storage T-02-19 («MUST NOT comprobar activo; T-02-20»). **Alternativa:** `create` devuelve `Producto` y el Gateway hace `get`. Se descartó: duplica un round-trip en cada alta.

### 2. Parche de ficha en `catalog`, no en `domain`

**Elección:** tipo de aplicación `PatchProducto` (`id`, `version`, campos de ficha como `Option`; **sin** `stock` ni `activo`). `update` hace `get` → merge → validar refs/defs del snapshot resultante → `ProductRepo::update(UpdateProducto { … })`. Si se envía `atributos` o `codigos`, el array entero reemplaza. Al menos un campo de ficha además de `version`; si no, `invalid_input`. T-02-21 solo mapea JSON inglés → `PatchProducto`.

**Por qué:** T-02-19 dejó el merge fuera del repo (`UpdateProducto` es snapshot). OpenAPI `ProductUpdate` es parcial (`minProperties: 2`). Marca/atributo fusionaron en el handler porque esos tickets incluían HTTP; aquí HTTP es T-02-21.

**Alternativa:** snapshot obligatorio en el caso de uso y merge en T-02-21. Se descartó: T-02-19 asignó el merge a T-02-20 y el Gateway quedaría gordo. **Alternativa:** `Option` en `UpdateProducto` de `domain`. Se descartó: `None` vs «borrar descripción» ya se resolvió como snapshot en T-02-19.

### 3. Categoría/marca inexistente o inactiva → `invalid_input`

**Elección:** `CategoryRepo::get` / `BrandRepo::get`; si `not_found` o `activo == false` → `DomainError::invalid_input` con `FieldError.field` `categoria` o `marca`. Marca ausente (`None`) no se consulta. T-02-21 remapeará a `categoryId` / `brandId` y HTTP 400 (OpenAPI `POST /products` no declara 404 de alta).

**Por qué:** un `not_found` en create se leería como producto inexistente y mapearía a 404. El ticket exige existencia **y** activo; un código cubre ambos.

**Alternativa:** `not_found` si falta, `invalid_input` si inactiva. Se descartó: dos códigos para el mismo rechazo de referencia. **Alternativa:** `conflict`. Se descartó: 5.4 reserva `conflict` para ocupación/duplicados, no para FK inactiva.

### 4. Cargar defs por `id` y reusar `Producto::try_new` / validación de dominio

**Elección:** para cada `atributos[n].def`, `AttributeDefRepo::get`. Ausente → `invalid_input` `attributes[n].definitionId`. Con las defs cargadas, construir un `ProductoDraft` mínimo o invocar la misma validación de atributos que `Producto::try_new` **antes** del repo, para que `"cuarenta"` en `numero` falle en `catalog` aunque el fake sea permisivo. Def inactiva: **se acepta** (el ticket solo exige activo en categoría y marca; T-02-12 permite baja de defs sin occupancy).

**Por qué:** el Hace del ticket es «cargando las `AtributoDef` para validar». Storage también valida al persistir; el caso de uso no debe depender de eso para el unhappy de HU-02-04.

**Alternativa:** solo existencia de defs y dejar el tipo al repo. Se descartó: el ticket nombra la validación en `catalog`.

### 5. Fake `ProductRepo` en tests de `catalog`

**Elección:** `InMemoryProductRepo` en `#[cfg(test)]` de `producto.rs`: `HashMap` de `Producto`, UNIQUE de SKU → `duplicate_sku`, `update` con `version` esperada → `version_conflict`, `get` hidrata `FichaProducto` consultando los fakes de categoría/marca/def (inyectados por `Arc`). `create` pone `version` 1, `activo` true, `stock = stock_inicial` **sin** insertar `movimiento_stock`. Reusar `marca::testing::InMemoryBrandRepo` y `atributo_def::testing::InMemoryAttributeDefRepo`; para categoría, exponer `pub mod testing` (mismo patrón) o un fake mínimo local si el recálculo de `ruta` no hace falta más que seed de una hija.

**Por qué:** Listo cuando = tests en memoria. El ledger no es este ticket.

**Alternativa:** tests contra Surreal embedded. Se descartó: el ticket lo prohíbe y ya está cubierto en T-02-18/T-02-19.

### 6. `ProductGateway` object-safe, sin baja

**Elección:** trait con futures boxed (`Pin<Box<dyn Future + Send>>`), métodos `create` / `get` / `update`. `ProductService` lo implementa. Sin `deactivate` / `reactivate` / `list`.

**Por qué:** `AppState` no puede ser genérico; T-02-06/T-02-09/T-02-12 fijaron el patrón. T-02-22 añadirá baja.

**Alternativa:** `async-trait`. Se descartó: el workspace no lo usa. **Alternativa:** omitir la fachada hasta T-02-21. Se descartó: T-02-05/T-02-09 la dejaron en el ticket de casos de uso para no reabrir `catalog`.

### 7. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-20` y rename a `T-02-20` (igual que T-02-19).

## Risks / Trade-offs

- **[Fake de producto duplica hidratación de T-02-19]** → Mitigación: hidratar lo mínimo (marca.nombre, categoria.ruta, clave/etiqueta/unidad) para los escenarios; storage sigue siendo la fuente de persistencia.
- **[Merge en catalog vs HTTP]** → Mitigación: `PatchProducto` usa nombres en español; T-02-21 solo traduce el DTO. Si el maintainer prefiere merge en el Gateway, se mueve en T-02-21 sin cambiar el puerto.
- **[`create` + `get` no es atómico en el fake]** → Mitigación: el fake es síncrono bajo mutex; en storage el producto ya existe al terminar `create`. Aceptable para MVP.
- **[Ámbito ~2 h]** → Mitigación: sin HTTP, sin `set_activo`, sin cambiar domain/storage. No se divide el ticket.

## Migration Plan

Sin migración `.surql`. Despliegue = merge a `develop`. Rollback = revertir `catalog/src/producto.rs` y el reexport; las tablas no cambian.

## Open Questions

Ninguna que altere spec, enfoque o tasks.
