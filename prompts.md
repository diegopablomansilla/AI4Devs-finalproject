> Detalla en esta sección los prompts principales utilizados durante la creación del proyecto, que justifiquen el uso de asistentes de código en todas las fases del ciclo de vida del desarrollo. Esperamos un máximo de 3 por sección, principalmente los de creación inicial o  los de corrección o adición de funcionalidades que consideres más relevantes.
Puedes añadir adicionalmente la conversación completa como link o archivo adjunto si así lo consideras


## Índice

1. [Descripción general del producto](#1-descripción-general-del-producto)
2. [Arquitectura del sistema](#2-arquitectura-del-sistema)
3. [Modelo de datos](#3-modelo-de-datos)
4. [Especificación de la API](#4-especificación-de-la-api)
5. [Historias de usuario](#5-historias-de-usuario)
6. [Tickets de trabajo](#6-tickets-de-trabajo)
7. [Pull requests](#7-pull-requests)

---

## 1. Descripción general del producto

**Prompt 1:**

### Rol: Product Owner (PO) y Arquitecto de Software Senior especializado en Retail Industrial, Sistemas POS Local-First de Alto Rendimiento y Búsqueda Multimodal.

### Objetivo: Elaborar una descripción exhaustiva del proyecto, especificación técnica detallada, arquitectura de componentes y estructura del Backlog inicial (MVP) para sistema llamado "Vectra". Dejar la descripcion en un documento 1-descripcion-general-del-producto.md en una carpeta /docs

### Especificaciones del Proyecto:

1. Visión General y Filosofía:
   - Filosofía: "Local-First" — 100% de operatividad local sin dependencia de la nube o internet.
   - Vision del producto:
        - **Vectra** es el sistema comercial definitivo de cara al usuario, diseñado para el retail moderno y entornos con inventarios complejos (Ferreterías, Farmacias, Repuestos Automotrices, etc.). Transforma cualquier mostrador estándar en una estación de inteligencia avanzada. Permite a los operadores encontrar piezas al instante mediante reconocimiento visual, consultar inventario por voz y recibir recomendaciones de productos alternativos y de stock, todo sin depender de una conexión a internet activa.
        - **Vortex Core** es el motor backend local, silencioso y de altísimo rendimiento que se ejecuta en el Nodo Maestro (Hardware Apple Silicon). Impulsa toda la indexación de datos, embeddings vectoriales, procesamiento local de voz, registros transaccionales (ledger) y el razonamiento de la Inteligencia Artificial local.

2. Arquitectura de Datos y Búsqueda Multimodal:
   - Backend Core: Rust + Axum como motor asíncrono de alta concurrencia (core, gestión de APIs, streaming local y coordinación).
   - Interfaz (Frontend): Single Page Application (SPA / PWA) con diseño "Glassmorphism" inspirado en Apple, modo oscuro nativo (#0A0A0A) y barra de búsqueda centralizada ("Omnibox").
   - Fuente Única de Verdad (Single Source of Truth) & POS/CRM: SurrealDB (embebido en Rust con RocksDB/Surrealkv). Manejo de datos maestros, transacciones y relaciones de grafos avanzadas (ej. `RELATE producto:tornillo -> sugiere -> producto:arandela`, productos sustitutos e historial de pedidos y clientes).
   - Búsqueda de Texto Instantánea: Meilisearch (Sidecar) para búsquedas tolerantes a errores ortográficos (typo-tolerance), jerarquización de relevancia, resaltado y sugerencias search-as-you-type en submilisegundos.
   - Búsqueda Visual / Identidad Visual (Vortex Vision): Qdrant (Sidecar) con índice HNSW y embeddings extraídos localmente (onnxruntime-rs en Rust) para reconocimiento de piezas en menos de 10ms.
   - Reconocimiento de Voz Local: Whisper local (Whisper.cpp / Whisper-rs) ejecutado en Rust sobre el Neural Engine/GPU. Ruteo automático de intenciones (Texto -> Meilisearch; Pregunta técnica -> RAG/LLM; "Busca esto" -> Qdrant).
   - Modo IA y Razonamiento Local: RAG (Retrieval-Augmented Generation) combinando SurrealDB + Qdrant con un LLM Local (Llama 3 / Mistral vía Ollama) para resolver consultas complejas del operador.

   Nota: El stack tecnologico esta basado en una recomendacion por parte de nuestro TL, sin embargo tu puedes opinar y proponer cualquier cambio.

3. Módulos de Poder del Ecosistema Vortex Core:
   - Dentro del alcance del MVP (Se centrara en Ferreterias y/ Repuestos)
        - Vectra Search Portal: Omnibox de búsqueda multimodal (Texto, Voz, Imagen y Modo IA). 
            - Features principales de la busqueda por Texto:
                - Búsqueda por relevancia, tolerancia a errores tipográficos (typo-tolerance) y búsqueda "mientras escribes" (search-as-you-type) de forma nativa
                - Ranking de Relevancia: si el usuario busca "Perno", el "Perno de Acero" es más relevante que una "Arandela para Perno".
                - Highlighting y Sugerencias: El buscador devuelve exactamente qué parte del texto coincidió y ofrece sugerencias instantáneas mientras escribes (Search-as-you-type).
            - Features principales luego de obtener resultados
                - Proponer productos similares en base a relaciones
                - Proponer opciones de productos si el stock es 0
                - Proponer opciones de productos cuando el producto buscado usualmente se utiliza juto a o tro producto. Ejemplo arandera con x tornillo.
   - Fuera de alcance del MVP (solo para resaltar la vision estrategica del producto o hacia donde se quiere evlucionar)
        - Vortex Talk: Socio consultor de IA conversacional local para métricas, diagnósticos y decisiones estratégicas. 
        - Predictive Supply: Oráculo de reposición predictiva de stock basado en ventas, patrones y factores externos.
        - Integrity Module: Auditor silencioso que detecta discrepancias y previene fraude cruzando visión y tickets POS.
        - Vortex Sentinel: Analítica física de tráfico y mapas de calor en el local mediante cámaras IP locales (privacidad total).
        - Vortex Briefing & Mirror: Dashboard gerencial diario y sincronización cifrada opcional con la nube.

4. Target de Hardware y Rendimiento:
   - Hardware de Validación/MVP: Apple Silicon (MacBook Air M4 con 24GB de Memoria Unificada).
   - Hardware de Producción: Mac Mini (serie M Pro).
   - Distribución de Memoria Objetivo: Residencia simultánea en RAM de SurrealDB, Meilisearch, Qdrant, Whisper y Llama 3 8B cuantizado sin hacer swap a disco.

   Nota: El Hardware esta basado en una recomendacion por parte de nuestra area de ventas, sin embargo tu puedes opinar y proponer cualquier cambio.

Antes de comenzar a redactar la especificación y el backlog, como Product Owner, pregúntame qué información adicional o aclaraciones necesitas para definir este proyecto.


**Prompt 2:**
Preguntas del agente:

¿Qué modos de búsqueda del Omnibox entran realmente en el MVP? (el proyecto final tiene tiempo acotado)

Todas, pero podemos aclarar que la de Imagen esta sujeto a lograr terminar las otras, y aclararia el orden es Texto y Recomendaciones por grafo, luego voz y por ultimo imagen
¿El MVP incluye funcionalidad POS transaccional (ventas, tickets, caja, clientes/CRM) o solo búsqueda + consulta de stock?

A y B
¿De dónde proviene el catálogo e inventario en el MVP?

Vectra es el sistema maestro (ABM de productos propio)
¿Cómo se generan las relaciones de grafo (sustituto, complementario, similar)?

Carga manual/curada por el comercio
¿Topología de despliegue en tienda?

Nodo Maestro + varias terminales/tablets vía LAN (PWA)
¿Tamaño esperado del catálogo por tienda?

A para el MVP
Para la búsqueda visual, ¿con qué imágenes de referencia contamos?

Fotos de catálogo del proveedor (1 por SKU aprox.)
Respecto al stack propuesto por el TL, ¿qué grado de cambio aceptas?

B pero tu propones y yo decido si apruebo o no
¿Framework preferido para la SPA/PWA?

SPA Vanilla Web (solo html, css y js vanilla)
Idioma y operadores: ¿para qué idioma se optimizan voz y búsqueda?

Español (Latinoamérica)
¿Qué formato quieres para el backlog inicial?

A, pero a las historias de usuarios te las voy a pedir mas adelante, por ahora no las escribiras, pero si podria ser las epicas

**Prompt 3:**

Preguntas del agente:

No mencionaste el Modo IA (RAG + LLM) en el orden de prioridad. ¿Dónde va?

Justo después de Texto + Grafo
Respondiste 'A y B' en POS. ¿El POS básico (carrito, venta, descuento de stock) es parte firme del MVP o condicionado como Imagen?

Condicionado: primero búsqueda/stock, POS si hay tiempo
PROPUESTA 1 — Eliminar Qdrant y usar el índice vectorial HNSW nativo de SurrealDB (con <10k SKUs sobra rendimiento). Reduce un sidecar y evita sincronizar dos stores. Riesgo: menor madurez que Qdrant.

Anotalo en la descripcion del proyecto, deja la pregunta abierta
PROPUESTA 2 — Motor LLM: Ollama en desarrollo, pero abstraído tras una interfaz para migrar a llama.cpp embebido/MLX en producción. Modelo: Llama 3.1 8B o Qwen2.5 7B Instruct (Q4_K_M, mejor español).

REaliza una propuesta en base al hardware pensado para el MVP
PROPUESTA 3 — Mantener Meilisearch como sidecar (lo recomiendo: es el mejor en typo-tolerance/highlighting en español). Alternativa: Tantivy embebido en Rust (cero sidecars, pero más trabajo propio).

Mantener Meilisearch (Recomendado)
PROPUESTA 4 — Hardware: MVP en MacBook Air M4 24GB OK. Producción: Mac Mini M4 Pro con 48GB (margen para Vortex Talk/Sentinel) + UPS obligatoria por ser nodo único.

Mantener Mac Mini M Pro 24GB; documentar 48GB como opcional
¿Se requiere autenticación y roles en el MVP (terminales LAN)?

Para el MVP no hace falta
¿Periféricos a contemplar en el MVP?

Cámara (webcam/tablet) para búsqueda visual, Micrófono en la terminal para voz
---

## 2. Arquitectura del Sistema

### **2.1. Diagrama de arquitectura:**

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

### **2.2. Descripción de componentes principales:**

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

### **2.3. Descripción de alto nivel del proyecto y estructura de ficheros**

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

### **2.4. Infraestructura y despliegue**

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

### **2.5. Seguridad**

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

### **2.6. Tests**

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

---

### 3. Modelo de Datos

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

---

### 4. Especificación de la API

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

---

### 5. Historias de Usuario

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

---

### 6. Tickets de Trabajo

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**

---

### 7. Pull Requests

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**
