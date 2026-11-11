# Spec Delta

Ticket padre: T-01-01. Change: T-01-01. HU: HU-01-04.

## Purpose

Define el esqueleto de la SPA Vectra en `vectra-web/`: documento HTML en español, módulos ES sin paso de *build*, *router* por *hash* entre la vista del operador y la de administración, y una vista de ruta no encontrada, de modo que cambiar `#/` por `#/admin/productos` actualice el contenido sin recargar la página.

## ADDED Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/css/` (directorio; MAY estar vacío salvo un marcador de árbol)
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
- `vectra-web/js/components/` (directorio; MAY estar vacío salvo un marcador de árbol)

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module. Este change MUST NOT agregar hojas de *design tokens* ni Web Components en `js/components/`.

#### Scenario: el documento está en español y carga el módulo versionado (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** el elemento `html` tiene `lang="es"`
- **AND** existe un script `type="module"` cuyo `src` es `/static/{{VERSION}}/js/app.js`
- **AND** existe el *outlet* de la aplicación
- **AND** el archivo no referencia un *bundler* ni un `package.json`

#### Scenario: el árbol incluye css, app, router y components (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existen `css/` y `js/components/` como directorios
- **AND** existen los archivos `js/app.js` y `js/router.js`

#### Scenario: falta index.html o lang=es (unhappy)

- **WHEN** no existe `vectra-web/index.html`, o el `html` no tiene `lang="es"`
- **THEN** el harness de verificación de `vectra-web` termina con código distinto de 0
- **AND** el mensaje nombra el archivo o el atributo que falta

#### Scenario: el script no es módulo versionado (unhappy)

- **WHEN** `index.html` no carga un ES Module en `/static/{{VERSION}}/js/app.js` (falta `type="module"`, falta `{{VERSION}}`, o el path no es ese)
- **THEN** el harness de verificación termina con código distinto de 0
- **AND** el mensaje indica el `src` o el `type` esperado

### Requirement: Router hash Operador y Administración sin recargar

El *router* MUST interpretar el *hash* de la URL (fragmento tras `#`) y MUST pintar una sola vista en el *outlet*:

| Hash | Vista | Atributo observable |
|---|---|---|
| vacío, `#` o `#/` | Operador | `data-view="operator"` |
| `#/admin/<sección>` con `<sección>` = un slug `[a-z0-9-]+` | Administración de esa sección | `data-view="admin"` y `data-section="<sección>"` |

La vista operador MUST incluir un enlace `<a href="#/admin/productos">` (texto en español). La vista administración MUST incluir un enlace `<a href="#/">` para volver al operador (texto en español). MUST NOT pedir credenciales ni mostrar un formulario de login al entrar a `#/admin/…`.

Al arrancar, el *router* MUST pintar la vista que corresponde al *hash* actual. Al cambiar el *hash* (navegación del usuario o asignación a `location.hash`), MUST actualizar el *outlet* y MUST NOT recargar el documento (`location.reload` / carga completa de `index.html`). `#/admin/productos` MUST ser una sección válida.

#### Scenario: navegar de operador a admin/productos cambia la vista sin recargar (happy)

- **WHEN** la app está iniciada con *hash* `#/` (vista operador visible)
- **AND** el *hash* pasa a `#/admin/productos` (clic en el enlace o cambio de `location.hash`)
- **THEN** el *outlet* muestra un elemento con `data-view="admin"` y `data-section="productos"`
- **AND** ya no está visible la vista `data-view="operator"`
- **AND** no se invoca recarga del documento

#### Scenario: volver de admin/productos al operador no recarga (happy)

- **WHEN** la app está iniciada con *hash* `#/admin/productos`
- **AND** el *hash* pasa a `#/`
- **THEN** el *outlet* muestra `data-view="operator"`
- **AND** ya no está visible `data-view="admin"`
- **AND** no se invoca recarga del documento

#### Scenario: hash vacío se trata como operador (happy)

- **WHEN** la app inicia con *hash* vacío o `#`
- **THEN** el *outlet* muestra `data-view="operator"`

#### Scenario: administración no pide credenciales (happy)

- **WHEN** se navega a `#/admin/productos`
- **THEN** el *outlet* muestra la vista administración
- **AND** no hay formulario de usuario/contraseña ni desafío de autenticación

### Requirement: Hash desconocido o mal formado muestra vista no encontrada

Un *hash* que no coincide con las filas de la tabla del requirement «Router hash Operador y Administración sin recargar» MUST pintar una vista de no encontrada en el *outlet* (`data-view="not-found"`) y MUST NOT recargar el documento. Eso incluye, al menos: `#/no-existe`, `#/admin` (sin sección), `#/admin/` (sección vacía), `#/admin/productos/extra` (segmento de más), un slug con mayúsculas (`#/admin/Productos`) y un slug con caracteres fuera de `[a-z0-9-]+`. La vista no encontrada MUST incluir un enlace `<a href="#/">` al operador. MUST NOT dejar el *outlet* vacío. MUST NOT pintar a la vez `data-view="operator"` o `data-view="admin"` con `data-view="not-found"`.

#### Scenario: ruta desconocida es not-found sin recargar (unhappy)

- **WHEN** la app está iniciada
- **AND** el *hash* pasa a `#/no-existe`
- **THEN** el *outlet* muestra `data-view="not-found"`
- **AND** no muestra `data-view="operator"` ni `data-view="admin"`
- **AND** no se invoca recarga del documento

#### Scenario: admin sin sección es not-found (unhappy)

- **WHEN** el *hash* es `#/admin` o `#/admin/`
- **THEN** el *outlet* muestra `data-view="not-found"`

#### Scenario: admin con segmento extra o slug inválido es not-found (unhappy)

- **WHEN** el *hash* es `#/admin/productos/extra` o `#/admin/Productos` o `#/admin/prod uctos`
- **THEN** el *outlet* muestra `data-view="not-found"`
- **AND** no muestra `data-section="productos"`

### Requirement: Tests del esqueleto y del router

Antes de archive, la suite MUST incluir estos tests (nombres estables; ubicación / tipo). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `index_html_declares_lang_es_and_versioned_module` | `scripts/check-vectra-web.sh` o fixture `scripts/fixtures/vectra-web/` | harness de árbol |
| `vectra_web_tree_includes_css_app_router_components` | `scripts/check-vectra-web.sh` | harness de árbol |
| `router_operator_to_admin_productos_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_admin_productos_to_operator_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_empty_hash_is_operator` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_unknown_hash_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_admin_without_section_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_invalid_admin_slug_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica) y que no hubo recarga. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js` y la presencia de `css/`, `js/app.js`, `js/router.js` y `js/components/`.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `#/` ↔ `#/admin/productos` sin recarga pasan
- **AND** los de *hash* vacío, desconocido, admin sin sección y slug inválido pasan
- **AND** los de `lang="es"` y el módulo versionado pasan

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
