DEFINE TABLE IF NOT EXISTS categoria SCHEMAFULL CHANGEFEED 3d;
DEFINE FIELD IF NOT EXISTS nombre         ON categoria TYPE string ASSERT string::len($value) > 0;
DEFINE FIELD IF NOT EXISTS slug           ON categoria TYPE string;
DEFINE FIELD IF NOT EXISTS padre          ON categoria TYPE option<record<categoria>>;
DEFINE FIELD IF NOT EXISTS ancestros      ON categoria TYPE array<record<categoria>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS ruta           ON categoria TYPE array<string>;
DEFINE FIELD IF NOT EXISTS activo         ON categoria TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS creado_en      ON categoria TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD IF NOT EXISTS actualizado_en ON categoria TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS uq_categoria_slug ON categoria FIELDS padre, slug UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_categoria_ancestros ON categoria FIELDS ancestros;
