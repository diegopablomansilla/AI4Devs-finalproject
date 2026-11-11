# Design

## Context

`develop` tiene *tokens*, `base.css`, *router* y harness `check-vectra-web.sh`. ADR-06: ES Modules, sin *build*. Los componentes se sirven bajo `/static/<versión>/js/components/…`; las URLs `../../css/tokens.css` desde ahí resuelven al CSS versionado.

## Decisions

### 1. shared.js con fetch + CSSStyleSheet

Carga única en caché de `tokens.css` y `base.css`; `replace()` async en navegador. Tests inyectan `setSharedStyleLoader(readFile)` antes de importar componentes.

### 2. Controles nativos en shadow

`button` / `input` dentro del shadow para foco y teclado sin reinventar. Estilos locales mínimos con `var(--vx-*)` y `--vx-touch-min`.

### 3. components.html relativo

Sin `{{VERSION}}`: se accede como `/static/<versión>/dev/components.html`. Enlaces `../css/` y `../js/`.

### 4. Tests con DOM stub en Node

Sin jsdom: `dom-stub.mjs` define `HTMLElement`, `customElements`, `CSSStyleSheet.replaceSync`, shadow root fake. Importar stub antes de los módulos de componente.

## Non-Goals

Modal/panel/toast (T-01-04), validación de formularios completa, integración en vistas del *router*.
