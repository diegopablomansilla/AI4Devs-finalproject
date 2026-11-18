DEFINE TABLE IF NOT EXISTS marca SCHEMAFULL CHANGEFEED 3d;
DEFINE FIELD IF NOT EXISTS nombre         ON marca TYPE string ASSERT string::len($value) > 0;
DEFINE FIELD IF NOT EXISTS slug           ON marca TYPE string;
DEFINE FIELD IF NOT EXISTS activo         ON marca TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS creado_en      ON marca TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD IF NOT EXISTS actualizado_en ON marca TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS uq_marca_slug ON marca FIELDS slug UNIQUE;