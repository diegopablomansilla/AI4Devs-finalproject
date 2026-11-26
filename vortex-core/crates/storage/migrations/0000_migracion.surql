DEFINE TABLE IF NOT EXISTS migracion SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS version     ON migracion TYPE int;
DEFINE FIELD IF NOT EXISTS nombre      ON migracion TYPE string;
DEFINE FIELD IF NOT EXISTS checksum    ON migracion TYPE string;
DEFINE FIELD IF NOT EXISTS aplicada_en ON migracion TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX IF NOT EXISTS uq_migracion_version ON migracion FIELDS version UNIQUE;
