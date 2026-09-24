## Índice

0. [Ficha del proyecto](#0-ficha-del-proyecto)
1. [Descripción general del producto](#1-descripción-general-del-producto)
2. [Arquitectura del sistema](#2-arquitectura-del-sistema)
3. [Modelo de datos](#3-modelo-de-datos)
4. [Especificación de la API](#4-especificación-de-la-api)
5. [Historias de usuario](#5-historias-de-usuario)
6. [Tickets de trabajo](#6-tickets-de-trabajo)
7. [Pull requests](#7-pull-requests)

---

## 0. Ficha del proyecto

### **0.1. Tu nombre completo:**

Diego Pablo Mansilla

### **0.2. Nombre del proyecto:**

Vectra

### **0.3. Descripción breve del proyecto:**

**Vectra** es un buscador inteligente de mostrador *Local-First* para ferreterías y repuestos automotrices. Desde una PWA en la red local, el vendedor localiza productos al instante por texto, voz o lenguaje natural (Modo IA) y recibe sustitutos, similares y complementarios según el stock, sin depender de internet. El backend **Vortex Core** (Rust) corre en un Mac en el local y concentra catálogo, búsqueda y modelos de IA en el **Nodo Maestro**.

### **0.4. URL del proyecto:**

https://github.com/diegopablomansilla/AI4Devs-finalproject



---

## 1. Descripción general del producto


### **1.1. Objetivo:**

Dar al mostrador de ferreterías y repuestos una forma rápida y fiable de encontrar cualquier SKU —aunque el nombre sea impreciso o haya errores de tipeo— y de actuar cuando no hay stock, **sin depender de internet ni de la nube**.

**Qué soluciona:** catálogos enormes y técnicos, búsquedas del POS que fallan con un typo, ventas perdidas por desconocer sustitutos y pocas ventas cruzadas, además del costo y la fragilidad de soluciones con IA en la nube.

**Para quién:** el **vendedor** gana un asistente en el mostrador (texto, voz, preguntas en lenguaje natural y recomendaciones); el **dueño** reduce ventas perdidas y sube el ticket sin pagar servicios externos; el **encargado** mantiene catálogo, stock y relaciones entre productos en el mismo sistema.

### **1.2. Características y funcionalidades principales:**

- **Omnibox multimodal:** barra de búsqueda central en la PWA; modos texto, voz, Modo IA y (en alcance extendido) cámara, con foco permanente para uso en mostrador.
- **Búsqueda por texto:** resultados mientras se escribe, tolerancia a typos, ranking por relevancia y stock, resaltado de coincidencias, sugerencias, sinónimos del rubro, búsqueda por SKU/código fabricante/OEM y filtros (categoría, marca, atributos, stock).
- **Recomendaciones por grafo:** al elegir un producto, panel con similares, sustitutos (destacados si no hay stock) y complementos, definidos y curados por el administrador.
- **Modo IA (RAG local):** preguntas en lenguaje natural con respuesta en streaming; solo cita productos reales del catálogo con stock, sin inventar SKUs.
- **Búsqueda por voz:** transcripción local (Whisper), normalización de medidas y jerga, ruteo a búsqueda, IA, stock o cámara; transcripción editable en el Omnibox.
- **Catálogo e inventario nativos:** ABM de productos, categorías, marcas y atributos técnicos; stock mediante ledger inmutable (ingresos, ajustes, ventas); imágenes, sinónimos y dataset semilla para arranque.
- **Operación local:** varias terminales en la LAN contra un **Nodo Maestro**; micrófono y cámara en el navegador; degradación controlada si falla un motor (p. ej. Meilisearch u Ollama) sin tumbar la búsqueda básica.
- **Extensiones condicionadas del MVP:** búsqueda visual por foto (**Vortex Vision**) y **POS básico** (carrito y confirmación de venta con descuento atómico de stock, sin pagos ni facturación).

### **1.3. Diseño y experiencia de usuario:**

**Idea de interfaz:** Vectra es una PWA de mostrador con **modo oscuro** (`#0A0A0A`) y estética **Glassmorphism** (tarjetas translúcidas, desenfoque de fondo): legible bajo luz de local, con contraste accesible (WCAG AA) y botones grandes (≥ 44 px) para uso táctil con las manos ocupadas. El **Omnibox** ocupa el centro de la pantalla del **Operador** y recupera el foco tras cada acción; a su alrededor van resultados en tarjetas (foto, stock, precio, ubicación) y, al elegir un ítem, un panel lateral con recomendaciones. Iconos discretos activan voz, Modo IA (`?`) y, si está habilitado, cámara. La zona **Administrador** reutiliza el mismo lenguaje visual para ABM de catálogo, stock, relaciones y sinónimos (en el MVP no hay login: solo cambio de vista).

**Ejemplos de uso (Operador):**

1. Un cliente pide “un tornillo así” pero no sabe el nombre: el vendedor escribe *tornilo hexagonal m8* en el Omnibox, corrige el typo gracias a la búsqueda tolerante, abre la ficha y ofrece arandela y tuerca desde **Se usa junto con**.
2. El producto elegido tiene **stock 0**: el panel resalta **Sustitutos** con alternativas disponibles ordenadas por prioridad del encargado.
3. Pregunta técnica en el mostrador: escribe *¿qué broca para taco de 8 en hormigón?* o activa Modo IA; la respuesta llega en streaming con tarjetas de productos reales con stock.
4. Manos sucias o lejos del teclado: mantiene **Espacio** o el micrófono, dice *“discos de corte 115”*; la transcripción aparece en el Omnibox, la edita si hace falta y ejecuta la búsqueda.

*(Capturas de pantalla o un videotutorial del flujo Operador pueden añadirse aquí cuando la UI esté implementada.)*

### **1.4. Instrucciones de instalación:**
> Documenta de manera precisa las instrucciones para instalar y poner en marcha el proyecto en local (librerías, backend, frontend, servidor, base de datos, migraciones y semillas de datos, etc.)

---

## 2. Arquitectura del Sistema

### **2.1. Diagrama de arquitectura:**

El diagrama oficial del MVP (modelo **C4**, enfoque **Local-First**, terminales **Vectra** + **Vortex Core** en el Nodo Maestro) está en **[`docs/2-arquitectura.md`](docs/2-arquitectura.md)**:

| Nivel C4 | Sección | Contenido |
|---|---|---|
| Contexto (1) | [§ 2.2](docs/2-arquitectura.md#22-diagrama-de-contexto-c4--nivel-1) | Operador, administrador, red local, backups e internet solo en instalación |
| Contenedores (2) | [§ 2.3](docs/2-arquitectura.md#23-diagrama-de-contenedores-c4--nivel-2) | SPA, API Gateway, SurrealDB, Meilisearch, Ollama, inferencia en Core |

Patrón **hexagonal** (puertos/adaptadores), drivers y ADRs: [§ 2.5–2.6 y § 2.10](docs/2-arquitectura.md#25-principios-de-diseño). El desglose interno de Vortex Core (C4 nivel 3) figura en [`docs/3-componentes.md`](docs/3-componentes.md#31-vista-general-de-componentes).

### **2.2. Descripción de componentes principales:**

La descripción detallada de cada pieza de **Vortex Core** (tecnología, responsabilidades y crates Rust) está en **[`docs/3-componentes.md`](docs/3-componentes.md)**:

| Tema | Sección |
|---|---|
| Diagrama y vista general | [§ 3.1](docs/3-componentes.md#31-vista-general-de-componentes) |
| Crates del workspace y dependencias | [§ 3.2](docs/3-componentes.md#32-mapa-de-crates-y-regla-de-dependencias) |
| Puertos (`traits`) e adaptadores | [§ 3.3](docs/3-componentes.md#33-puertos-y-adaptadores) |
| Ficha por componente (API Gateway, Search, Catalog, AI, Sync, …) | [§ 3.4](docs/3-componentes.md#34-ficha-de-cada-componente) |
| EP-07 / EP-08 (visión, POS) | [§ 3.5](docs/3-componentes.md#35-componentes-condicionados) |

Responsabilidades a nivel **contenedor** (SPA, sidecars, SurrealDB): [`docs/2-arquitectura.md` § 2.4](docs/2-arquitectura.md#24-responsabilidades-de-los-contenedores). Stack resumido: [§ 2.9](docs/2-arquitectura.md#29-stack-tecnológico).

### **2.3. Descripción de alto nivel del proyecto y estructura de ficheros**

> Representa la estructura del proyecto y explica brevemente el propósito de las carpetas principales, así como si obedece a algún patrón o arquitectura específica.

### **2.4. Infraestructura y despliegue**

> Detalla la infraestructura del proyecto, incluyendo un diagrama en el formato que creas conveniente, y explica el proceso de despliegue que se sigue

### **2.5. Seguridad**

> Enumera y describe las prácticas de seguridad principales que se han implementado en el proyecto, añadiendo ejemplos si procede

### **2.6. Tests**

> Describe brevemente algunos de los tests realizados

---

## 3. Modelo de Datos

### **3.1. Diagrama del modelo de datos:**

El diagrama **entidad-relación** (Mermaid, claves, relaciones y grafo de productos) está en **[`docs/4-modelo.md`](docs/4-modelo.md)** — [§ 4.2 Diagrama entidad-relación](docs/4-modelo.md#42-diagrama-entidad-relación). Principios (SurrealDB como verdad, índices derivados): [§ 4.1](docs/4-modelo.md#41-principios-del-modelo).

### **3.2. Descripción de entidades principales:**

Atributos, restricciones, tipos y reglas de negocio por dominio en el mismo **[`docs/4-modelo.md`](docs/4-modelo.md)**:

| Dominio | Sección |
|---|---|
| Convenciones (IDs, NS/DB, soft delete) | [§ 4.3](docs/4-modelo.md#43-convenciones) |
| Catálogo (`producto`, `categoria`, `marca`, `atributo_def`, imágenes, sinónimos) | [§ 4.4](docs/4-modelo.md#44-catálogo) |
| Inventario (`movimiento_stock`, ledger) | [§ 4.5](docs/4-modelo.md#45-inventario-ledger-de-stock) |
| Grafo (`similar_a`, `sustituye_a`, `complementa`) | [§ 4.6](docs/4-modelo.md#46-grafo-de-relaciones-entre-productos) |
| Embeddings y outbox / sync | [§ 4.7](docs/4-modelo.md#47-vectores-e-índices-hnsw) · [§ 4.8](docs/4-modelo.md#48-sincronización-outbox-y-cursores) |
| Índice Meilisearch derivado | [§ 4.11](docs/4-modelo.md#411-modelo-derivado-en-meilisearch) |

Ventas (alcance condicionado): [§ 4.9](docs/4-modelo.md#49-ventas-condicionado).

---

## 4. Especificación de la API

Contrato HTTP de **Vortex Core** (OpenAPI 3.1, prefijo `/api/v1`, errores RFC 9457): **[`docs/5-contract-api.md`](docs/5-contract-api.md)**. Especificación ejecutable: **[`docs/openapi.yaml`](docs/openapi.yaml)** ([§ 5.13](docs/5-contract-api.md#513-especificación-openapi-31)).

| Tema | Sección |
|---|---|
| Convenciones (base URL, IDs, decimales, auth MVP) | [§ 5.1](docs/5-contract-api.md#51-convenciones-generales) |
| Mapa completo de endpoints | [§ 5.2](docs/5-contract-api.md#52-mapa-de-endpoints) |
| API ↔ modelo de datos | [§ 5.3](docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) |
| SSE Modo IA / voz / degradación | [§ 5.8](docs/5-contract-api.md#58-streaming-sse-del-modo-ia) · [§ 5.9](docs/5-contract-api.md#59-voz) · [§ 5.11](docs/5-contract-api.md#511-degradación-vista-desde-la-api) |

**Endpoints representativos del mostrador** (detalle y ejemplos en el contrato):

1. `GET /api/v1/search` — búsqueda por texto, facetas y tolerancia a errores ([EP-03](docs/6-historias.md)).
2. `GET /api/v1/products/{productId}/recommendations` — similares, sustitutos y complementos ([EP-04](docs/6-historias.md)).
3. `POST /api/v1/ai/ask` — pregunta en lenguaje natural con respuesta en streaming SSE ([EP-05](docs/6-historias.md)).

---

## 5. Historias de Usuario

Backlog del MVP (épicas `EP-XX`, historias `HU-XX-YY`, criterios Gherkin y DoD): **[`docs/6-historias.md`](docs/6-historias.md)** — fuente única del backlog.

| Tema | Sección |
|---|---|
| Convenciones, roles y DoD | [§ 6.1](docs/6-historias.md#61-convenciones) |
| Épicas MoSCoW y dependencias | [§ 6.2](docs/6-historias.md#62-backlog-de-épicas) |
| Hitos H1–H4 | [§ 6.3](docs/6-historias.md#63-hitos) |
| Detalle por épica (EP-00 … EP-09) | [§ 6.4–6.13](docs/6-historias.md#64-ep-00--plataforma-vortex-core) |

**Tres historias representativas del mostrador** (enunciado y escenarios completos en el documento):

1. **HU-03-03** — búsqueda mientras se escribe, tolerante a errores ([EP-03](docs/6-historias.md#67-ep-03--búsqueda-por-texto)).
2. **HU-04-05** — panel de recomendaciones ([EP-04](docs/6-historias.md#68-ep-04--grafo-de-relaciones-y-recomendaciones)).
3. **HU-05-05** — preguntar en lenguaje natural con respuesta en streaming ([EP-05](docs/6-historias.md#69-ep-05--modo-ia-rag-local)).

---

## 6. Tickets de Trabajo

Desglose técnico de las historias en tickets `T-XX-YY` (~2 h, orden lineal, criterios *Listo cuando*): **[`docs/7-tickets.md`](docs/7-tickets.md)**.

| Tema | Sección |
|---|---|
| Convenciones y DoD | [§ 7.1](docs/7-tickets.md#71-convenciones) |
| Fases, dependencias y orden global | [§ 7.4–7.5](docs/7-tickets.md#74-fases-y-dependencias) |
| Tickets por fase (0 … 8) | [§ 7.6–7.17](docs/7-tickets.md#76-fase-0--fundaciones-de-vortex-core) |
| Trazabilidad `HU` → `T` | [§ 7.18](docs/7-tickets.md#718-trazabilidad-historia--tickets) |

**Tres tickets representativos** (alcance completo en el documento; backend, frontend y datos):

1. **T-00-08** — apertura de SurrealDB embebido y migraciones ([Fase 0](docs/7-tickets.md#76-fase-0--fundaciones-de-vortex-core), *bases de datos*).
2. **T-03-13** — endpoint `GET /search` ([Fase 3](docs/7-tickets.md#79-fase-3--búsqueda-por-texto), *backend*).
3. **T-01-08** — vista del operador y componente `vx-omnibox` ([Fase 1](docs/7-tickets.md#77-fase-1--shell-de-vectra), *frontend*).

---

## 7. Pull Requests

> Documenta 3 de las Pull Requests realizadas durante la ejecución del proyecto

**Pull Request 1**

**Pull Request 2**

**Pull Request 3**

