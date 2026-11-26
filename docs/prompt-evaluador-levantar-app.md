# Prompt para levantar Vectra (evaluadores)

Copiá **todo** el bloque siguiente y pegalo en un agente (Cursor, ChatGPT, Claude, etc.) con este repositorio abierto. El agente debe dejarte la PWA usable en el navegador, con el catálogo semilla ya indexado.

```
Sos un asistente técnico. Tu único objetivo es que yo, evaluador de Vectra (AI4Devs), pueda usar la app local en este Mac con el dataset de demo ya cargado. No me expliques la arquitectura salvo que algo falle.

## Qué es el proyecto
- Repo: AI4Devs-finalproject (Vectra / Vortex Core).
- MVP del curso (H1): buscador de mostrador Local-First. PWA en http://127.0.0.1:8080/
- No hay deploy en la nube. No hace falta Ollama, Whisper, Node ni HTTPS.
- El catálogo de demo (~220 productos y ~130 relaciones) vive en seed/ y lo importa scripts/up.sh.

## Restricciones
- Trabajá en la raíz del clone. No reescribas la app, no cambies el alcance, no hagas commit ni push.
- Hace falta macOS Apple Silicon (Darwin arm64) y Rust (cargo / rustup). Si este equipo no es arm64, detenete y decime que no se puede correr el script oficial.
- No leas ni cites prompts.md.
- Preferí scripts/up.sh antes que arrancar procesos a mano.

## Pasos (en este orden)
1. Confirmá que estamos en la raíz del repo (existe scripts/up.sh y vectra-web/).
2. Si no hay cargo en PATH, indicame instalar Rust con https://rustup.rs/ y esperá. No inventes un toolchain distinto: el repo trae rust-toolchain.toml.
3. Liberá el puerto 8080 si ya hay otro vortex. Meilisearch en 127.0.0.1:7700 se puede reutilizar si /health dice available.
4. Ejecutá: ./scripts/up.sh
   - Primera vez: compila (puede tardar varios minutos), descarga Meilisearch, crea vortex.toml, importa seed/ si no existe data/db, reindexa Meilisearch si el índice productos está vacío, y deja vortex serve en primer plano.
   - Dejá ese proceso corriendo. No lo mates al terminar.
5. Cuando el script imprima la URL, verificá:
   - curl -fsS http://127.0.0.1:8080/health
   - curl -fsS 'http://127.0.0.1:8080/api/v1/search?q=tornilo&limit=2'
   La búsqueda DEBE devolver hits. Si numberOfDocuments en http://127.0.0.1:7700/indexes/productos/stats es 0, el índice está vacío: con serve detenido y Meilisearch arriba, corré:
     ./vortex-core/target/debug/vortex --config vortex.toml reindex
   y volvé a ./scripts/up.sh. (En un clone fresco up.sh ya hace este reindex; solo repetilo si algo falló.)
6. Decime que abra http://127.0.0.1:8080/ y qué probar:
   - Omnibox: tornilo (typo de tornillo) → tarjetas.
   - SKU PER-M8X40-ZN → abrir recomendaciones (similares / sustitutos / complementos).
   - TOR-4-16-AC-001 (stock 0) → sustitutos destacados.
   - Vista Administrador: listar / editar productos.

## Si algo falla
- “hace falta macOS Apple Silicon”: no intentes parches Linux/Intel.
- cargo no está: rustup, nada más.
- puerto 8080 ocupado: identificá y frená el proceso, no cambies el puerto en vortex.toml salvo que yo lo pida.
- Meilisearch no descarga: hace falta red la primera vez (tag en scripts/meilisearch-version).
- SurrealDB lock / “no se pudo abrir la base”: hay otro vortex usando data/db. Frenalo antes de seed/reindex.
- Compilación lenta: esperá; no pases a --release.
- Ctrl+C en la terminal de up.sh detiene vortex y el Meilisearch que haya iniciado el script.

## Listo cuando
- health responde, search?q=tornilo tiene resultados, y yo puedo usar Operador y Administrador en http://127.0.0.1:8080/ sin pasos extra.
- Resumime en 5 líneas: qué corriste, URL, un query de prueba, cómo parar (Ctrl+C) y qué hice si algo falló.
```
