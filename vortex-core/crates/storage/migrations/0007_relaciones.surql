DEFINE TABLE IF NOT EXISTS similar_a SCHEMAFULL TYPE RELATION IN producto OUT producto ENFORCED CHANGEFEED 3d;
DEFINE FIELD IF NOT EXISTS prioridad ON similar_a TYPE int DEFAULT 1 ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS nota ON similar_a TYPE option<string>;
DEFINE FIELD IF NOT EXISTS grupo_bidireccional ON similar_a TYPE option<string>;
DEFINE FIELD IF NOT EXISTS creado_en ON similar_a TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD IF NOT EXISTS actualizado_en ON similar_a TYPE datetime VALUE time::now();
DEFINE INDEX IF NOT EXISTS uq_similar_a ON similar_a FIELDS in, out UNIQUE;

DEFINE TABLE IF NOT EXISTS sustituible_por SCHEMAFULL TYPE RELATION IN producto OUT producto ENFORCED CHANGEFEED 3d;
DEFINE FIELD IF NOT EXISTS prioridad ON sustituible_por TYPE int DEFAULT 1 ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS nota ON sustituible_por TYPE option<string>;
DEFINE FIELD IF NOT EXISTS grupo_bidireccional ON sustituible_por TYPE option<string>;
DEFINE FIELD IF NOT EXISTS creado_en ON sustituible_por TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD IF NOT EXISTS actualizado_en ON sustituible_por TYPE datetime VALUE time::now();
DEFINE INDEX IF NOT EXISTS uq_sustituible_por ON sustituible_por FIELDS in, out UNIQUE;

DEFINE TABLE IF NOT EXISTS complementa SCHEMAFULL TYPE RELATION IN producto OUT producto ENFORCED CHANGEFEED 3d;
DEFINE FIELD IF NOT EXISTS prioridad ON complementa TYPE int DEFAULT 1 ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS nota ON complementa TYPE option<string>;
DEFINE FIELD IF NOT EXISTS grupo_bidireccional ON complementa TYPE option<string>;
DEFINE FIELD IF NOT EXISTS creado_en ON complementa TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD IF NOT EXISTS actualizado_en ON complementa TYPE datetime VALUE time::now();
DEFINE INDEX IF NOT EXISTS uq_complementa ON complementa FIELDS in, out UNIQUE;