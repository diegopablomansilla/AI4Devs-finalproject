DEFINE TABLE IF NOT EXISTS atributo_def SCHEMAFULL CHANGEFEED 3d;
DEFINE FIELD IF NOT EXISTS clave          ON atributo_def TYPE string;
DEFINE FIELD IF NOT EXISTS etiqueta       ON atributo_def TYPE string;
DEFINE FIELD IF NOT EXISTS tipo           ON atributo_def TYPE string ASSERT $value IN ['texto', 'numero', 'booleano', 'enum'];
DEFINE FIELD IF NOT EXISTS unidad         ON atributo_def TYPE option<string>;
DEFINE FIELD IF NOT EXISTS opciones       ON atributo_def TYPE option<array<string>>;
DEFINE FIELD IF NOT EXISTS facetable      ON atributo_def TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS buscable       ON atributo_def TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS orden          ON atributo_def TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS activo         ON atributo_def TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS creado_en      ON atributo_def TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD IF NOT EXISTS actualizado_en ON atributo_def TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS uq_atributo_clave ON atributo_def FIELDS clave UNIQUE;