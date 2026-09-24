# 1. Descripción General del Producto — Vectra

> **Estado:** Borrador v0.1 — MVP
> **Roles autores:** Product Owner + Arquitecto de Software
> **Vertical inicial del MVP:** Ferreterías y Repuestos Automotrices
> **Idioma objetivo:** Español (Latinoamérica)

---

## Índice

1. [Resumen ejecutivo](#11-resumen-ejecutivo)
2. [Visión y filosofía del producto](#12-visión-y-filosofía-del-producto)
3. [Problema y propuesta de valor](#13-problema-y-propuesta-de-valor)
4. [Usuarios y contexto de uso](#14-usuarios-y-contexto-de-uso)
5. [Alcance del MVP](#15-alcance-del-mvp)
6. [Visión estratégica (fuera del MVP)](#16-visión-estratégica-fuera-del-mvp)
7. [Especificación funcional](#17-especificación-funcional)
8. [Arquitectura de componentes](#18-arquitectura-de-componentes)
9. [Stack tecnológico y decisiones de arquitectura](#19-stack-tecnológico-y-decisiones-de-arquitectura)
10. [Hardware, rendimiento y presupuesto de memoria](#110-hardware-rendimiento-y-presupuesto-de-memoria)
11. [Requisitos no funcionales](#111-requisitos-no-funcionales)
12. [Riesgos y mitigaciones](#112-riesgos-y-mitigaciones)
13. [Preguntas abiertas](#113-preguntas-abiertas)
14. [Backlog inicial del MVP — Épicas](#114-backlog-inicial-del-mvp--épicas)
15. [Glosario](#115-glosario)

---

## 1.1. Resumen ejecutivo

**Vectra** es un sistema comercial de mostrador *Local-First* para retail con inventarios complejos (ferreterías, repuestos automotrices, farmacias). Permite al operador encontrar cualquier producto en milisegundos por **texto**, **voz**, **imagen** o **pregunta en lenguaje natural (Modo IA)**, y le propone automáticamente **sustitutos**, **complementarios** y **alternativas con stock**, todo **sin conexión a internet**.

El sistema se compone de dos piezas:

| Pieza | Descripción |
|---|---|
| **Vectra** (frontend) | Aplicación web (SPA/PWA) en HTML, CSS y JavaScript vanilla, estética *Glassmorphism* estilo Apple, modo oscuro nativo (`#0A0A0A`) y una barra de búsqueda central: el **Omnibox**. Se ejecuta en el navegador de cualquier terminal de la red local. |
| **Vortex Core** (backend) | Motor local en Rust + Axum que corre en el **Nodo Maestro** (Apple Silicon). Orquesta la base de datos, los índices de búsqueda, la transcripción de voz, los embeddings visuales y el LLM local. |

---

## 1.2. Visión y filosofía del producto

### Visión

> Transformar cualquier mostrador estándar en una **estación de inteligencia avanzada**, donde el operador encuentra la pieza correcta al instante —aunque no sepa su nombre exacto— y el sistema le sugiere qué más necesita el cliente.

### Filosofía *Local-First*

1. **100 % operativo sin internet.** Ninguna funcionalidad del MVP depende de servicios en la nube. Los modelos de IA, índices y datos residen en el Nodo Maestro.
2. **Privacidad por diseño.** Voz, imágenes y datos comerciales nunca salen del local.
3. **Latencia percibida cero.** El cuello de botella debe ser el humano, no el sistema.
4. **Una única fuente de verdad.** SurrealDB es el dato maestro; todo índice (texto, vectores) es **derivado y reconstruible** desde él.
5. **Operación sin técnico.** Un solo servicio que arranca, supervisa y recupera a los demás componentes.

---

## 1.3. Problema y propuesta de valor

### Problema

- Los catálogos de ferretería/repuestos tienen miles de SKUs con nombres técnicos, abreviaturas y variantes (medida, rosca, material, norma). Encontrar "el tornillo que el cliente trae en la mano" depende de la **memoria del vendedor experto**.
- Los sistemas POS tradicionales ofrecen búsqueda por coincidencia exacta: un error de tipeo ("tornilo") devuelve cero resultados.
- Cuando no hay stock, la venta se pierde porque el vendedor no conoce el **sustituto equivalente**.
- Se pierden ventas cruzadas obvias (arandela + tuerca con el perno).
- Las soluciones con IA existentes dependen de la nube: costo recurrente, latencia y caída ante cortes de internet.

### Propuesta de valor

| Para | Valor |
|---|---|
| **Vendedor de mostrador** | Encuentra productos por texto con errores, por voz o mostrando la pieza a la cámara. Recibe sugerencias listas para ofrecer. |
| **Dueño del comercio** | Menos ventas perdidas por falta de stock, más ticket promedio por venta cruzada, menor dependencia del vendedor experto, cero costo de nube. |
| **Vendedor nuevo** | Curva de aprendizaje drásticamente reducida: el conocimiento del catálogo está en el sistema. |

---

## 1.4. Usuarios y contexto de uso

### Personas

| Persona | Descripción | Necesidad principal |
|---|---|---|
| **Operador / Vendedor** | Atiende el mostrador, con el cliente enfrente y frecuentemente con las manos ocupadas o sucias. | Encontrar el producto rápido y responder "¿tenés?" y "¿qué más necesito?". |
| **Encargado / Administrador** | Mantiene el catálogo, precios, stock y relaciones entre productos. | Cargar y curar el catálogo y las relaciones de forma simple. |

> **Nota MVP:** no hay autenticación ni roles. Se asume una **red local de confianza**. La separación Operador/Administrador es solo de navegación en la interfaz.

### Topología de uso

- **Un Nodo Maestro** (MacBook Air M4 en MVP; Mac Mini en producción) que ejecuta Vortex Core.
- **N terminales** (PCs, tablets, notebooks) conectadas por **LAN/Wi-Fi local** que abren Vectra en el navegador como PWA.
- Las terminales aportan **micrófono** (voz) y **cámara** (búsqueda visual).

---

## 1.5. Alcance del MVP

### Orden de construcción y compromiso

| Prioridad | Capacidad | Compromiso |
|---|---|---|
| 1 | **Búsqueda por Texto** (Omnibox + Meilisearch) | Firme |
| 1 | **Recomendaciones por Grafo** (similares, sustitutos sin stock, complementarios) | Firme |
| 2 | **Modo IA** (RAG + LLM local) | Firme |
| 3 | **Búsqueda por Voz** (Whisper local + ruteo de intención) | Firme |
| 4 | **Búsqueda por Imagen** (embeddings visuales + búsqueda vectorial) | **Condicionado** a completar lo anterior |
| 4 | **POS básico** (carrito, venta, descuento de stock) | **Condicionado** a completar lo anterior |

Transversal y firme: **Catálogo e Inventario nativo** (Vectra es el sistema maestro de productos), **plataforma Vortex Core** y **shell de la interfaz**.

### Dentro del alcance

- ABM (alta, baja, modificación) de productos, categorías, marcas, atributos técnicos y stock.
- Carga **manual/curada** de relaciones entre productos (sustituto, complementario, similar).
- Omnibox con búsqueda multimodal (texto, voz, IA; imagen condicionada).
- Panel de resultados con stock, precio, ubicación y panel de recomendaciones.
- Acceso desde múltiples terminales en la LAN.
- Captura de audio y video desde el navegador de la terminal.

### Fuera del alcance del MVP

- Autenticación, usuarios, roles y auditoría de accesos.
- CRM (clientes, historial de pedidos por cliente).
- Facturación fiscal, medios de pago, caja, impresora de tickets.
- Lector de código de barras (se evaluará post-MVP; técnicamente trivial por ser HID/teclado).
- Integración o sincronización con ERPs externos.
- Inferencia automática de relaciones (por ventas, atributos o embeddings).
- Multi-sucursal y sincronización con la nube.
- Idiomas distintos del español.

---

## 1.6. Visión estratégica (fuera del MVP)

Módulos que definen la evolución del ecosistema **Vortex Core** y condicionan decisiones de arquitectura hoy (extensibilidad, presupuesto de memoria, modelo de datos):

| Módulo | Descripción | Implicancia arquitectónica temprana |
|---|---|---|
| **Vortex Talk** | Socio consultor de IA conversacional local para métricas, diagnósticos y decisiones estratégicas. | El proveedor de LLM debe estar abstraído y soportar *tool calling*. |
| **Predictive Supply** | Oráculo de reposición predictiva de stock basado en ventas, patrones y factores externos. | Los movimientos de stock deben registrarse como **ledger inmutable** desde el MVP. |
| **Integrity Module** | Auditor silencioso que detecta discrepancias y previene fraude cruzando visión y tickets POS. | Embeddings visuales por SKU reutilizables; eventos de venta con marca de tiempo. |
| **Vortex Sentinel** | Analítica física de tráfico y mapas de calor mediante cámaras IP locales (privacidad total). | Requiere hardware con más memoria/GPU (ver [2-arquitectura.md, 2.12](2-arquitectura.md#212-hardware-y-presupuesto-de-memoria)). |
| **Vortex Briefing & Mirror** | Dashboard gerencial diario y sincronización cifrada opcional con la nube. | IDs globales estables y *change feed* en la base de datos. |

Verticales futuras: farmacias, repuestos de motos/maquinaria agrícola, bazares y distribuidoras.

---

## 1.7. Especificación funcional

### 1.7.1. Vectra Search Portal — Omnibox

Barra de búsqueda única, siempre visible y con foco por defecto. Modos:

| Modo | Disparador | Motor |
|---|---|---|
| **Texto** | Escribir | Meilisearch |
| **Voz** | Botón micrófono o atajo (mantener `Espacio` con Omnibox vacío) | Whisper → ruteo de intención |
| **IA** | Prefijo `?`, toggle "Modo IA" o intención detectada | RAG (SurrealDB + índice vectorial) + LLM |
| **Imagen** *(condicionado)* | Botón cámara | Embedding visual → búsqueda vectorial |

### 1.7.2. Búsqueda por Texto

- **Search-as-you-type:** resultados en cada pulsación, cancelando la petición anterior (`AbortController`).
- **Tolerancia a errores tipográficos:** "tornilo", "arandla", "bujia" encuentran sus productos.
- **Ranking de relevancia:** buscar "Perno" prioriza "Perno de Acero" (coincidencia en el nombre/tipo) sobre "Arandela para Perno" (coincidencia secundaria); a igual relevancia, primero los productos con más stock. La configuración del índice está en [4-modelo.md, 4.11](4-modelo.md#411-modelo-derivado-en-meilisearch).
- **Highlighting:** se resalta el fragmento que coincidió (`_formatted`).
- **Sugerencias instantáneas:** autocompletado de términos y productos.
- **Sinónimos del rubro:** diccionario editable (ej. "llave francesa" ↔ "llave ajustable", "bulón" ↔ "perno", "tarugo" ↔ "taco").
- **Búsqueda por código:** SKU, código de fabricante y código OEM (repuestos).
- **Filtros facetados:** categoría, marca, medida, material, con/sin stock.

### 1.7.3. Recomendaciones post-búsqueda (Grafo)

Al seleccionar un producto se muestra un panel lateral con tres bloques, resueltos mediante relaciones de grafo en SurrealDB:

| Bloque | Relación | Regla |
|---|---|---|
| **Similares** | `producto -> similar_a -> producto` | Productos equivalentes o alternativos (otra marca, otra calidad). |
| **Sustitutos (sin stock)** | `producto -> sustituye_a -> producto` | Se muestra **destacado** cuando `stock = 0`; solo sustitutos con `stock > 0`, ordenados por prioridad definida por el administrador. |
| **Se usa junto con** | `producto -> complementa -> producto` | Venta cruzada: "Perno M8 → Arandela M8, Tuerca M8". |

- Las relaciones son **dirigidas**; el administrador puede marcarlas como **bidireccionales** al crearlas.
- Carga **manual/curada** desde la vista de administración del producto.
- El modelado en SurrealDB y la consulta de recomendaciones están en [4-modelo.md, 4.6](4-modelo.md#46-grafo-de-relaciones-entre-productos).

### 1.7.4. Modo IA (RAG local)

Para consultas complejas en lenguaje natural del operador, por ejemplo:

- "¿Qué broca uso para hacer un agujero para un taco de 8 en hormigón?"
- "Necesito algo para pegar PVC que aguante presión, ¿qué tenemos?"
- "¿Qué diferencia hay entre el perno grado 5 y el grado 8?"

Flujo:

1. **Recuperación:** búsqueda híbrida — Meilisearch (léxica) + índice vectorial de texto (semántica) sobre fichas de producto y notas técnicas.
2. **Enriquecimiento:** SurrealDB aporta stock, precio y relaciones de grafo de los candidatos.
3. **Generación:** el LLM local redacta la respuesta **citando solo productos del catálogo** (con SKU y stock), en streaming (SSE, eventos en [5-contract-api.md, 5.8](5-contract-api.md#58-streaming-sse-del-modo-ia)).
4. **Guardarraíles:** si no hay productos relevantes, el LLM lo declara; nunca inventa SKUs. Las tarjetas de producto de la respuesta se validan contra la base de datos antes de renderizarse.

### 1.7.5. Búsqueda por Voz

1. La terminal captura audio (`MediaRecorder`, Opus/WebM o PCM 16 kHz) mientras se mantiene el botón o atajo.
2. Vortex Core transcribe con **Whisper local** (Metal) usando un *prompt inicial* con vocabulario del rubro (marcas, medidas, "M8", "3/8", "cabeza hexagonal") para mejorar la precisión.
3. **Ruteo de intención** (primero reglas, luego clasificador ligero con LLM si es ambiguo):

| Intención | Ejemplo | Destino |
|---|---|---|
| Búsqueda directa | "tornillo autoperforante 10 por 1" | Meilisearch (texto) |
| Pregunta técnica | "¿qué sellador sirve para alta temperatura?" | Modo IA (RAG) |
| Búsqueda visual | "busca esto", "¿qué es esta pieza?" | Abre la cámara (búsqueda visual) |
| Consulta de stock | "¿cuántos discos de corte de 115 quedan?" | Meilisearch + stock SurrealDB |

4. La transcripción se muestra en el Omnibox y es **editable** antes o después de ejecutarse.

### 1.7.6. Búsqueda por Imagen — Vortex Vision *(condicionado)*

1. El operador apunta la cámara de la terminal a la pieza y captura un cuadro.
2. Vortex Core extrae el embedding visual localmente (ONNX Runtime con CoreML).
3. Búsqueda de vecinos más cercanos (HNSW) sobre los embeddings de las **fotos de catálogo del proveedor** (≈ 1 por SKU).
4. Se devuelven los *top-k* candidatos con nivel de confianza; el operador confirma.
5. **Limitación conocida:** con una sola foto de estudio por SKU, la brecha entre "foto de catálogo" y "foto real en mostrador" reduce la precisión. Mitigación: aumento de datos (*data augmentation*) al indexar y opción de que el operador **agregue la foto confirmada** como nueva referencia del SKU (aprendizaje incremental).

### 1.7.7. Catálogo e Inventario (nativo)

- ABM de productos: SKU, nombre, tipo, marca, categoría, descripción, atributos técnicos (clave/valor tipados: medida, rosca, material, norma, grado), códigos alternativos (fabricante/OEM), precio, ubicación física (pasillo/estante), imagen(es).
- Stock actual derivado de un **ledger de movimientos** (ingreso, ajuste, venta), nunca editado directamente.
- ABM de relaciones entre productos.
- Diccionario de sinónimos del rubro.
- Carga inicial mediante **dataset semilla** para demo y validación (mismo formato que usará la importación futura).
- Modelo de datos del catálogo e inventario: [4-modelo.md, 4.4 y 4.5](4-modelo.md#44-catálogo).

### 1.7.8. POS básico *(condicionado)*

- Carrito en la terminal a partir de los resultados de búsqueda y recomendaciones.
- Confirmar venta → registra la venta y los movimientos de stock en una **transacción atómica** en SurrealDB.
- Sin medios de pago, facturación ni impresión en el MVP.

---

## 1.8. Arquitectura de componentes

> La arquitectura (diagramas de contexto y contenedores, principios de diseño, red local y estructura de repositorio) se documenta en [2. Arquitectura del Sistema](2-arquitectura.md). Los componentes internos de Vortex Core, con sus puertos y adaptadores, se detallan en [3. Componentes de Vortex Core](3-componentes.md). El modelo de datos está en [4. Modelo de Datos](4-modelo.md), el contrato de la API en [5. Contrato de API](5-contract-api.md) y el backlog en [6. Épicas e Historias de Usuario](6-historias.md).

En resumen, Vectra se compone de una **SPA vanilla** que corre en el navegador de cada terminal de la LAN y de **Vortex Core**, un binario Rust en el Nodo Maestro que embebe SurrealDB (fuente única de verdad, grafo e índice vectorial HNSW) y supervisa dos sidecars: **Meilisearch** (búsqueda de texto) y **Ollama** (LLM local). Voz y embeddings se ejecutan dentro de Vortex Core sobre Metal/CoreML.

---

## 1.9. Stack tecnológico y decisiones de arquitectura

> El stack completo, el registro de decisiones (ADR-01 a ADR-09) y la selección de modelos de IA están en [2-arquitectura.md, secciones 2.9 a 2.11](2-arquitectura.md#29-stack-tecnológico).

Stack resumido: **Rust + Axum + Tokio**, **HTML/CSS/JS vanilla** con Web Components (PWA), **SurrealDB embebido**, **Meilisearch**, **ort + CoreML** para embeddings, **whisper-rs** con Metal y **Ollama** con un LLM de 7–8 B en Q4_K_M.

---

## 1.10. Hardware, rendimiento y presupuesto de memoria

> El detalle de hardware y el presupuesto de memoria se documentan en [2-arquitectura.md, sección 2.12](2-arquitectura.md#212-hardware-y-presupuesto-de-memoria).

- **MVP / validación:** MacBook Air M4 · 24 GB (limitación térmica bajo carga sostenida de LLM).
- **Producción:** Mac Mini M Pro · 24 GB (opcional recomendado 48 GB para los módulos de la visión estratégica), con UPS.
- **Consumo estimado:** ≈ 14–17 GB de 24 GB, con todos los modelos residentes.

---

## 1.11. Requisitos no funcionales

### Rendimiento (medido en MacBook Air M4 · 24 GB, LAN Wi-Fi, catálogo ≤ 10 k SKUs)

| Métrica | Objetivo (p95) |
|---|---|
| Búsqueda por texto — tiempo del motor (Meilisearch) | < 10 ms |
| Búsqueda por texto — de extremo a extremo (tecla → resultados en pantalla) | < 100 ms |
| Recomendaciones de grafo (consulta SurrealDB) | < 20 ms |
| Búsqueda vectorial — solo consulta HNSW | < 10 ms |
| Búsqueda por imagen — extremo a extremo (captura → candidatos) | < 300 ms |
| Transcripción de voz (audio de 5 s) | < 1,5 s |
| Modo IA — tiempo al primer token | < 2 s |
| Modo IA — velocidad de generación | ≥ 15 tokens/s |
| Propagación de cambios SurrealDB → índices | < 1 s |
| Arranque en frío del Nodo Maestro hasta "listo" | < 60 s |

### Disponibilidad y operación

- **100 % offline** para todas las funcionalidades del MVP.
- Soporte de **al menos 5 terminales concurrentes**.
- Reinicio automático de sidecars caídos en < 5 s.
- **Backups** automáticos diarios de SurrealDB a disco externo o carpeta configurable; los índices no requieren backup (reconstruibles).
- Logs estructurados (`tracing`) con rotación local; endpoint `/health` y `/metrics`.

### Usabilidad

- Omnibox con foco automático; operación completa por teclado.
- Contraste accesible sobre fondo `#0A0A0A` pese al efecto *Glassmorphism* (WCAG AA en texto).
- Objetivos táctiles ≥ 44 px para tablets.

### Privacidad

- Audio e imágenes se procesan en memoria y **no se persisten** salvo confirmación explícita (p. ej., agregar foto de referencia a un SKU).

---

## 1.12. Riesgos y mitigaciones

| Riesgo | Impacto | Mitigación |
|---|---|---|
| *Thermal throttling* del MacBook Air bajo carga de LLM | Medio | Respuestas acotadas, modelo 7–8 B Q4, medición sostenida en el *benchmark*; producción en Mac Mini con ventilador. |
| Madurez de SurrealDB embebido (APIs y rendimiento cambiantes entre versiones) | Medio | Fijar versión, encapsular acceso en repositorios, backups diarios, tests de integración. |
| Cámara y micrófono bloqueados en la LAN sin HTTPS | Alto | CA local + certificado `vectra.local` desde la épica de plataforma. |
| Whisper confunde jerga técnica y medidas ("tres octavos", "M8") | Medio | *Prompt* inicial con vocabulario del rubro, normalización posterior (texto → "3/8", "M8"), transcripción editable. |
| Brecha entre foto de catálogo y foto real (búsqueda visual) | Alto | *Data augmentation*, fotos de referencia incrementales, mostrar *top-k* con confianza; funcionalidad condicionada. |
| LLM inventa productos o datos técnicos | Alto | RAG estricto, validación de SKUs contra la base, instrucciones de "no sé" explícitas, citas visibles. |
| Nodo Maestro como punto único de falla | Alto | UPS, backups, reinicio automático, degradación elegante. |
| Complejidad operativa de múltiples sidecars | Medio | Supervisor integrado; índice vectorial dentro de SurrealDB, lo que deja solo dos sidecars (ADR-08). |
| Rendimiento o madurez insuficiente del índice HNSW de SurrealDB | Medio | *Spike* al inicio de EP-05; acceso detrás del *trait* `VectorIndex` para migrar a Qdrant si no se cumple el objetivo. |
| Carga manual de relaciones costosa para el comercio | Medio | UI de carga rápida (buscar y relacionar en un paso), relaciones bidireccionales; inferencia automática como evolución post-MVP. |

---

## 1.13. Preguntas abiertas

| # | Pregunta | Opciones | Recomendación del Arquitecto | Decide |
|---|---|---|---|---|
| **PA-01** | ¿Qdrant o índice vectorial nativo de SurrealDB? | **A)** Mantener Qdrant como sidecar. **B)** Usar HNSW nativo de SurrealDB. **C)** Qdrant detrás de un *trait* para poder migrar. | **Resuelta:** opción **B**, HNSW nativo de SurrealDB detrás del *trait* `VectorIndex` (ver [ADR-08](2-arquitectura.md#adr-08--índice-vectorial-en-surrealdb)). | PO + TL |
| **PA-02** | Modelo LLM final | Qwen 7–8 B vs. Llama 3.1 8B (ver [2-arquitectura.md, 2.11.1](2-arquitectura.md#2111-llm-para-el-hardware-del-mvp-macbook-air-m4--24-gb)). | Decidir por *benchmark*. | PO + TL |
| **PA-03** | Modelo de embedding visual | DINOv2 vs. CLIP/SigLIP. | Decidir por *benchmark* sobre el dataset semilla. | Arquitecto |
| **PA-04** | ¿La PWA debe funcionar si la terminal pierde la conexión con el Nodo Maestro? | Solo *app shell* con aviso vs. caché de lectura del catálogo. | *App shell* con aviso en el MVP. | PO |
| **PA-05** | Fuente del dataset semilla (≈ 2–5 k SKUs reales de ferretería/repuestos con fotos) | Catálogo de un comercio piloto vs. dataset generado. | Comercio piloto si es posible: valida la búsqueda con datos reales. | PO |

---

## 1.14. Backlog inicial del MVP — Épicas

> El backlog de épicas (priorización MoSCoW y dependencias), los hitos y las historias de usuario con sus criterios de aceptación están en [6. Épicas e Historias de Usuario](6-historias.md).

Resumen: diez épicas, de **EP-00** (Plataforma Vortex Core) a **EP-09** (Calidad, rendimiento y operación), agrupadas en cuatro hitos: **H1** Buscador inteligente, **H2** Asistente IA, **H3** Manos libres y **H4** Extensiones condicionadas.

---

## 1.15. Glosario

| Término | Definición |
|---|---|
| **Local-First** | Arquitectura donde los datos y el cómputo residen localmente y el sistema funciona completo sin internet. |
| **Nodo Maestro** | Equipo Apple Silicon que ejecuta Vortex Core y aloja datos, índices y modelos. |
| **Omnibox** | Barra de búsqueda única y central de Vectra que acepta texto, voz, imagen y preguntas. |
| **Sidecar** | Proceso auxiliar gestionado por Vortex Core (Meilisearch, Ollama). |
| **RAG** | *Retrieval-Augmented Generation*: el LLM responde apoyándose en datos recuperados del catálogo. |
| **Embedding** | Representación vectorial numérica de un texto o imagen que permite medir similitud. |
| **HNSW** | *Hierarchical Navigable Small World*: algoritmo de búsqueda aproximada de vecinos cercanos. |
| **Ledger** | Registro inmutable de movimientos (stock, ventas) del que se deriva el estado actual. |
| **Outbox** | Patrón que registra eventos de cambio en la misma transacción que el dato, para sincronizar sistemas derivados. |
| **Glassmorphism** | Estilo visual con superficies translúcidas y desenfoque de fondo (`backdrop-filter`). |
| **Q4_K_M** | Formato de cuantización de 4 bits de llama.cpp que reduce memoria con baja pérdida de calidad. |
| **SKU** | *Stock Keeping Unit*: código único de producto. |
