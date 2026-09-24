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

Nota: implemente una rule para que no se lea y contemple este archivo.
Nota: por lo general comence con un prompt modo plan y en auto, que explora el contenido de la carpeta doc, luegop en un modelo como opus, realizo la creacion de cada documento de la carpeta /docs

## 1. Descripción general del producto

**Prompt 1:**

### Rol: Product Owner (PO) y Arquitecto de Software Senior especializado en Retail Industrial, Sistemas POS Local-First de Alto Rendimiento y Búsqueda Multimodal.

### Objetivo: Elaborar una descripción exhaustiva del proyecto, especificación técnica detallada, arquitectura de componentes y estructura del Backlog inicial (MVP) para sistema llamado "Vectra". Dejar la descripcion en un documento 1-descripcion-general-del-producto.md en una carpeta /docs

### Especificaciones del Proyecto:

Cursor Opus 5.5

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

Cursor Opus 5.5

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

Cursor Opus 5.5

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

Cursor Opus 5.5

Rol: Arquitecto de Software especializado en aplicaciones web Rust, y sistemas Local-First con IA.
Objetivo: Eres la persona experta en la arquitectura del proyecto Vectra.
A partir del documento "1-descripcion-general-del-producto.md" definido anteriormente, crea un documento llamado 2-arquitectura.md (en la carpeta /docs) que incluya el Diagrama de arquitectura.   Antes de empezar, pregúntame qué necesitas saber.
Si ves que algo quedo redundante en 1-descripcion-general-del-producto.md, muevelo y dejalo en 2-arquitectura.md.

**Prompt 2:**

Cursor Opus 5.5

Mencionás dos nombres: '2-arquitectura.md' y '2-arquitectura-modelo.md'. ¿Cuál uso? ¿Incluye también el modelo de datos (tablas/relaciones SurrealDB)?

2-arquitectura.md, solo arquitectura (modelo de datos en un doc aparte más adelante)
Del doc 1, las secciones 1.8 (Arquitectura de componentes), 1.9 (Stack y ADRs) y 1.10 (Hardware y memoria) son claramente de arquitectura. ¿Qué hago con ellas?

Moverlas al doc 2 y dejar en el doc 1 un resumen breve + enlace (Recomendado)
PA-01 (Qdrant sidecar vs HNSW nativo de SurrealDB) afecta el diagrama. ¿Cómo lo reflejo?

Decidir ya: HNSW de SurrealDB detrás del trait VectorIndex (elimina un sidecar)
¿Qué nivel de diagramas querés? (todos en Mermaid)

C4 Nivel 1: Contexto del sistema

**Prompt 3:**

Cursor Auto

para que necesitamos "Descubrimiento: el nodo se anuncia vía mDNS como vectra.local."?a la SPA  la va a servir axum, no tenemos apps descktop ni apps nativas mobile

Respuesta del modelo:

No lo necesitamos para “descubrir” una app nativa: la SPA la sirve Axum y el operador abre una URL en el navegador.

vectra.local vía mDNS resuelve otro problema de la LAN:

Nombre estable para HTTPS. Cámara y micrófono solo funcionan en contexto seguro. El certificado tiene que coincidir con lo que el usuario escribe en la barra. Si usás solo la IP (https://192.168.x.x), el IP puede cambiar con DHCP y renovar el cert / reconfigurar terminales es un lío.
URL humana y memorizable. En cada terminal: https://vectra.local en lugar de “preguntale al dueño la IP del Mac”.
Sin DNS del router. mDNS evita tocar el router o editar /etc/hosts en cada PC/tablet.
No es descubrimiento de servicios tipo Bonjour para apps desktop/mobile. Es un hostname LAN barato para TLS + bookmark.

Si preferís simplificar el MVP, se puede sacar mDNS y usar, por ejemplo:

IP fija en el Nodo Maestro + certificado con SAN de IP, o
hostname fijo configurado a mano en las terminales.
El costo es peor DX de instalación (cada terminal necesita saber la IP o editar hosts).

¿Querés que en 2-arquitectura.md deje mDNS como está, o lo marco como opcional / lo reemplazo por IP fija + cert?

### **2.2. Descripción de componentes principales:**

**Prompt 1:**

Cursor Opus 5.5

Rol: Arquitecto de Software especializado en aplicaciones web Rust, y sistemas Local-First con IA.
Objetivo: Eres la persona experta en la arquitectura del proyecto Vectra.
A partir de los documentos en /docs, crea un documento llamado 3-componentes.md (en la carpeta /docs) que incluya el los componentes principales.   Antes de empezar, pregúntame si necesitas saber algo.
Si ves que algo quedo redundante en los .md que estan en /docs, muevelo y dejalo en este nuevo documento que te pido.

**Prompt 2:**

Cursor Opus 5.5

¿Qué nivel de detalle querés para cada componente?

C4 nivel 3: responsabilidad, interfaces expuestas y consumidas, dependencias, datos que maneja, errores y degradación, requisitos no funcionales que le aplican y épica asociada (Recomendado)

¿Qué hago con el contenido redundante de 2-arquitectura.md (2.3 diagrama, 2.4 responsabilidades, 2.6 puertos y adaptadores)?

Moverlo a 3-componentes.md y dejar en 2-arquitectura.md el diagrama de alto nivel más un enlace (Recomendado)

¿Qué entra en el documento?

Componentes de Vortex Core (servicios, workers, supervisor)

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

Cursor Opus 5.5

Rol: Arquitecto de Software especializado en aplicaciones web Rust, y sistemas Local-First con IA.
Objetivo: Eres la persona experta en la arquitectura del proyecto Vectra.
A partir de los documentos en /docs, crea un documento llamado 4-modelo.md (en la carpeta /docs) que incluya el modelo de datos en SurrealDB, ten en cuenta relaciones, schemas o shemasless, etc.   Antes de empezar, pregúntame si necesitas saber algo.
Si ves que algo quedo redundante en los .md que estan en /docs, muevelo y dejalo en este nuevo documento que te pido.

**Prompt 2:**

Cursor Opus 5.5

¿Qué usamos como ID de registro de los productos?

ULID generado (producto:01J...) y SKU como campo con índice UNIQUE; soporta que el SKU cambie y da IDs globales estables (Recomendado)

¿Cómo modelamos los atributos técnicos tipados (medida, rosca, material, norma, grado)?

Tabla `atributo_def` (clave, tipo, unidad, facetable) y en el producto un array de valores validados contra ella: esquema estricto y facetas dinámicas (Recomendado)

¿Las categorías son jerárquicas?

Sí, árbol (Herramientas > Eléctricas > Taladros) con referencia al padre (Recomendado)

¿Qué nivel de detalle tiene el precio en el MVP?

Solo el precio actual, sin historial

¿Dónde se guardan las imágenes de producto?

Archivos en disco del Nodo Maestro; en SurrealDB, tabla `imagen_producto` con ruta, hash, origen (catálogo/operador) y su embedding (Recomendado)
Sobre la redundancia: veo que en doc 1 (sección 1.7.3) está el ejemplo SurrealQL de relaciones, y en doc 3 (3.4.2, 3.4.12) aparecen entidades y reglas de datos. ¿Cómo lo movemos?

Mover el detalle de datos (ejemplo SurrealQL, lista de entidades, reglas de esquema) a 4-modelo.md y dejar en los otros docs un resumen de 1–2 líneas con enlace (Recomendado)

**Prompt 3:**

---

### 4. Especificación de la API

**Prompt 1:**

Cursor Opus 5.5

Rol: Arquitecto de Software especializado en aplicaciones web Rust, y sistemas Local-First con IA.
Objetivo: Eres la persona experta en la arquitectura del proyecto Vectra.
A partir de los documentos en /docs, crea un documento llamado 5-contract-api.md (en la carpeta /docs) que incluya el contrato de apis principales en openapi.   Antes de empezar, pregúntame si necesitas saber algo.
Si ves que algo quedo redundante en los .md que estan en /docs, muevelo y dejalo en este nuevo documento que te pido.

**Prompt 2:**

Cursor Opus 5.5

¿Qué grupos de endpoints incluyo en el contrato?

Alcance firme: catálogo e inventario, búsqueda por texto, recomendaciones y relaciones, Modo IA por SSE, voz, y health/metrics (recomendado)
¿En qué idioma van las rutas y los campos JSON?

Inglés (/api/v1/products, price, stock) y la documentación en español
¿Cómo entrego la especificación OpenAPI?

OpenAPI 3.1 en YAML dentro del .md y además un archivo docs/openapi.yaml válido para Swagger (recomendado)
¿Qué formato de error y de paginación uso?

Errores RFC 9457 (application/problem+json) con código de dominio; paginación por offset/limit en búsqueda y por cursor ULID en listados (recomendado)
Encontré contenido de API repartido en los otros docs: interfaces del API Gateway y mapeo de errores a HTTP (3.4.1), el 409 por bloqueo optimista (4.3) y detalles de SSE y AbortController (1.7). ¿Qué hago con eso?

Moverlo a 5-contract-api.md y dejar en su lugar un resumen breve con enlace (recomendado)

**Prompt 3:**

---

### 5. Historias de Usuario

**Prompt 1:**

Cursor Opus 5.5

Rol: PO / PM con apoyo de un rol de Arquitecto de Software especializado en aplicaciones web Rust, y sistemas Local-First con IA.
Objetivo: Eres la persona experta en gestion de proyectos y product owner del proyecto Vectra.
A partir de los documentos en /docs, crea un documento llamado 6-historias.md (en la carpeta /docs) que incluya las epicas e historias principales.   Antes de empezar, pregúntame si necesitas saber algo.
Si ves que algo quedo redundante en los .md que estan en /docs, muevelo y dejalo en este nuevo documento que te pido.




**Prompt 2:**

Cursor Opus 5.5

¿Qué épicas deben tener historias detalladas?

Todas las épicas con el mismo nivel de detalle
¿Qué formato y nivel de detalle querés para cada historia?

'Como / quiero / para' + criterios de aceptación en Gherkin, sin estimación
Las épicas de plataforma (EP-00) y de calidad (EP-09) no tienen un usuario final claro. ¿Cómo las redacto?

Como historias técnicas (enablers) desde el rol 'equipo de desarrollo' u 'operador del sistema' (Recomendado)
¿Muevo la sección 1.14 (backlog de épicas + hitos) del documento 1 a 6-historias.md y dejo un enlace en su lugar? También reviso otras redundancias (por ejemplo, la trazabilidad componente-épica de la sección 3.7).

Sí, mover la 1.14 y dejar un enlace; la 3.7 se queda en el documento 3 porque es una vista técnica (Recomendado)

**Prompt 3:**

---

### 6. Tickets de Trabajo

**Prompt 1:**

Cursor Opus 5.5

Rol: TL con apoyo de un rol de Arquitecto de Software especializado en aplicaciones web Rust, y sistemas Local-First con IA.
Objetivo: Eres la persona experta en el stack definido para proyecto Vectra.
A partir de los documentos en /docs, crea un documento llamado 7-tickets.md (en la carpeta /docs) que incluya el trabajo a realizar para cumplir con cada historia, quiero que definas el orden de ejecucion de cada ticket, teniendo en cuenta las dependencias de cada uno, es decir si primero debemos tener creada una tabla para poder crear por ejemplo un service en axum, primero iria el ticket de la tabla. Trata de que los tickets sean concretos, claros, con limites muy bien definidos y de resolucion rapida.   Antes de empezar, pregúntame si necesitas saber algo.
Si ves que algo quedo redundante en los .md que estan en /docs, muevelo y dejalo en este nuevo documento que te pido.




**Prompt 2:**

Cursor Opus 5.5

¿Qué épicas cubro con tickets? EP-07 (Visión) y EP-08 (POS) son 'Could' / condicionadas.

Todas (EP-00 a EP-09), marcando EP-07 y EP-08 como condicionadas al final del orden (Recomendado)
¿Qué tamaño máximo debería tener un ticket para considerarlo 'de resolución rápida'?

Hasta ~2 h (muy atómico, muchos más tickets)
¿Cuántas personas ejecutan los tickets? Esto define si el orden es una lista lineal o carriles en paralelo.

Una sola persona (con asistencia de IA): orden lineal único (Recomendado)
Hay decisiones abiertas (PM-03 RocksDB vs SurrealKV, PM-05 renombrar sustituye_a, PC-01 crate storage único, PC-03 crate pos, PAPI-03 sinónimos en API, PM-04 notas técnicas). ¿Cómo las trato?

Adopto la opción recomendada en cada doc y lo dejo explícito en el ticket (Recomendado)
Sobre mover lo redundante al nuevo documento: detecté solapamientos como el backlog de épicas en 1.14 vs 6.2, la trazabilidad componente-épica en 3.7, la tabla de preguntas abiertas en 1.13 vs 6.14, y la OpenAPI embebida completa en 5.13 que duplica openapi.yaml. ¿Qué alcance querés?

No tocar los otros documentos, solo crear 7-tickets.md

**Prompt 3:**

---

### 7. Pull Requests

**Prompt 1:**

**Prompt 2:**

**Prompt 3:**
