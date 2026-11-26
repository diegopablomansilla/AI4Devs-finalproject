DEFINE TABLE IF NOT EXISTS outbox SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS entidad ON outbox TYPE string ASSERT $value IN
  ['producto', 'categoria', 'marca', 'atributo_def', 'sinonimo', 'nota_tecnica', 'imagen_producto'];
DEFINE FIELD IF NOT EXISTS registro ON outbox TYPE record;
DEFINE FIELD IF NOT EXISTS operacion ON outbox TYPE string ASSERT $value IN ['upsert', 'delete'];
DEFINE FIELD IF NOT EXISTS motivo ON outbox TYPE option<string>;
DEFINE FIELD IF NOT EXISTS creado_en ON outbox TYPE datetime DEFAULT time::now() READONLY;

DEFINE TABLE IF NOT EXISTS sync_cursor SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS ultimo_evento ON sync_cursor TYPE option<record<outbox>>;
DEFINE FIELD IF NOT EXISTS actualizado_en ON sync_cursor TYPE datetime VALUE time::now();

DEFINE TABLE IF NOT EXISTS outbox_fallo SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS evento ON outbox_fallo TYPE record<outbox>;
DEFINE FIELD IF NOT EXISTS consumidor ON outbox_fallo TYPE string ASSERT $value IN ['meilisearch', 'vectores'];
DEFINE FIELD IF NOT EXISTS intentos ON outbox_fallo TYPE int DEFAULT 1;
DEFINE FIELD IF NOT EXISTS ultimo_error ON outbox_fallo TYPE string;
DEFINE FIELD IF NOT EXISTS estado ON outbox_fallo TYPE string ASSERT $value IN ['reintentando', 'fallido'];
DEFINE FIELD IF NOT EXISTS actualizado_en ON outbox_fallo TYPE datetime VALUE time::now();
DEFINE INDEX IF NOT EXISTS uq_outbox_fallo ON outbox_fallo FIELDS evento, consumidor UNIQUE;

IF !record::exists(sync_cursor:meilisearch) { CREATE sync_cursor:meilisearch; };
IF !record::exists(sync_cursor:vectores) { CREATE sync_cursor:vectores; };
