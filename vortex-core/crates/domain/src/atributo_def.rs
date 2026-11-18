//! Technical attribute definition entity and `AttributeDefRepo` port (T-02-11).

use std::future::Future;
use std::str::FromStr;

use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::id::Id;
use crate::timestamp::Timestamp;

const MAX_ETIQUETA_LEN: usize = 60;
const MAX_UNIDAD_LEN: usize = 16;

/// Attribute value kind (model 4.4.3 — Spanish wire values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoAtributo {
    Texto,
    Numero,
    Booleano,
    Enum,
}

impl TipoAtributo {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Texto => "texto",
            Self::Numero => "numero",
            Self::Booleano => "booleano",
            Self::Enum => "enum",
        }
    }

    pub fn parse_wire(s: &str) -> Result<Self, DomainError> {
        match s {
            "texto" => Ok(Self::Texto),
            "numero" => Ok(Self::Numero),
            "booleano" => Ok(Self::Booleano),
            "enum" => Ok(Self::Enum),
            _ => Err(field_invalid_input("tipo", "invalid tipo")),
        }
    }
}

impl FromStr for TipoAtributo {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_wire(s)
    }
}

/// Snapshot used to construct a validated `AtributoDef` (e.g. hydration from storage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtributoDefDraft {
    pub id: Id<AtributoDef>,
    pub clave: String,
    pub etiqueta: String,
    pub tipo: TipoAtributo,
    pub unidad: Option<String>,
    pub opciones: Option<Vec<String>>,
    pub facetable: bool,
    pub buscable: bool,
    pub orden: i32,
    pub activo: bool,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

/// Technical attribute definition (model 4.4.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtributoDef {
    pub id: Id<AtributoDef>,
    pub clave: String,
    pub etiqueta: String,
    pub tipo: TipoAtributo,
    pub unidad: Option<String>,
    pub opciones: Option<Vec<String>>,
    pub facetable: bool,
    pub buscable: bool,
    pub orden: i32,
    pub activo: bool,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

impl AtributoDef {
    pub fn try_new(draft: AtributoDefDraft) -> Result<Self, DomainError> {
        let AtributoDefDraft {
            id,
            clave,
            etiqueta,
            tipo,
            unidad,
            opciones,
            facetable,
            buscable,
            orden,
            activo,
            creado_en,
            actualizado_en,
        } = draft;

        validate_clave(&clave)?;
        let etiqueta = validate_etiqueta(&etiqueta)?;
        let unidad = validate_unidad(unidad)?;
        let opciones = normalize_opciones(tipo, opciones)?;

        Ok(Self {
            id,
            clave,
            etiqueta,
            tipo,
            unidad,
            opciones,
            facetable,
            buscable,
            orden,
            activo,
            creado_en,
            actualizado_en,
        })
    }
}

/// Input for `AttributeDefRepo::create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAtributoDef {
    pub clave: String,
    pub etiqueta: String,
    pub tipo: TipoAtributo,
    pub unidad: Option<String>,
    pub opciones: Option<Vec<String>>,
    pub facetable: bool,
    pub buscable: bool,
    pub orden: i32,
}

/// Input for `AttributeDefRepo::update`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAtributoDef {
    pub id: Id<AtributoDef>,
    pub clave: String,
    pub etiqueta: String,
    pub tipo: TipoAtributo,
    pub unidad: Option<String>,
    pub opciones: Option<Vec<String>>,
    pub facetable: bool,
    pub buscable: bool,
    pub orden: i32,
}

impl NewAtributoDef {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_clave(&self.clave)?;
        validate_etiqueta(&self.etiqueta)?;
        validate_unidad(self.unidad.clone())?;
        normalize_opciones(self.tipo, self.opciones.clone())?;
        Ok(())
    }
}

impl UpdateAtributoDef {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_clave(&self.clave)?;
        validate_etiqueta(&self.etiqueta)?;
        validate_unidad(self.unidad.clone())?;
        normalize_opciones(self.tipo, self.opciones.clone())?;
        Ok(())
    }
}

pub trait AttributeDefRepo {
    fn create(
        &self,
        input: NewAtributoDef,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send;

    fn get(
        &self,
        id: Id<AtributoDef>,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send;

    fn list_all(&self) -> impl Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send;

    fn update(
        &self,
        input: UpdateAtributoDef,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send;

    fn set_activo(
        &self,
        id: Id<AtributoDef>,
        activo: bool,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send;
}

fn validate_clave(clave: &str) -> Result<(), DomainError> {
    if matches_clave_pattern(clave) {
        Ok(())
    } else {
        Err(field_invalid_input("clave", "invalid clave format"))
    }
}

fn matches_clave_pattern(clave: &str) -> bool {
    if clave.is_empty() {
        return false;
    }
    let mut chars = clave.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn validate_etiqueta(etiqueta: &str) -> Result<String, DomainError> {
    let trimmed = etiqueta.trim();
    if trimmed.is_empty() {
        return Err(field_invalid_input(
            "etiqueta",
            "etiqueta must not be empty",
        ));
    }
    if trimmed.len() > MAX_ETIQUETA_LEN {
        return Err(field_invalid_input(
            "etiqueta",
            "etiqueta must be at most 60 characters",
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_unidad(unidad: Option<String>) -> Result<Option<String>, DomainError> {
    match unidad {
        None => Ok(None),
        Some(u) => {
            let t = u.trim();
            if t.len() > MAX_UNIDAD_LEN {
                return Err(field_invalid_input(
                    "unidad",
                    "unidad must be at most 16 characters",
                ));
            }
            if t.is_empty() {
                Ok(None)
            } else {
                Ok(Some(t.to_string()))
            }
        }
    }
}

fn normalize_opciones(
    tipo: TipoAtributo,
    opciones: Option<Vec<String>>,
) -> Result<Option<Vec<String>>, DomainError> {
    match tipo {
        TipoAtributo::Enum => {
            let Some(raw) = opciones else {
                return Err(field_invalid_input(
                    "opciones",
                    "enum type requires at least one option",
                ));
            };
            let out: Vec<String> = raw
                .into_iter()
                .map(|o| o.trim().to_string())
                .filter(|o| !o.is_empty())
                .collect();
            if out.is_empty() {
                return Err(field_invalid_input(
                    "opciones",
                    "enum type requires at least one option",
                ));
            }
            Ok(Some(out))
        }
        _ => {
            if let Some(ref opts) = opciones {
                let non_empty: Vec<_> = opts.iter().filter(|o| !o.trim().is_empty()).collect();
                if !non_empty.is_empty() {
                    return Err(field_invalid_input(
                        "opciones",
                        "options only allowed for enum type",
                    ));
                }
            }
            Ok(None)
        }
    }
}

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DomainCode, UnavailableCode};

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    fn draft_grado() -> AtributoDefDraft {
        AtributoDefDraft {
            id: Id::<AtributoDef>::generate(),
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: Some(vec!["2".into(), "5".into(), "8".into()]),
            facetable: true,
            buscable: true,
            orden: 0,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        }
    }

    #[test]
    fn atributo_def_constructs_grado_enum() {
        let draft = draft_grado();
        let id = draft.id.clone();
        let def = AtributoDef::try_new(draft).expect("grado");
        assert!(!id.to_string().contains(':'));
        assert_eq!(def.clave, "grado");
        assert_eq!(def.tipo, TipoAtributo::Enum);
        assert_eq!(def.opciones, Some(vec!["2".into(), "5".into(), "8".into()]));
        assert!(def.facetable);
    }

    #[test]
    fn atributo_def_constructs_medida_texto() {
        let id = Id::<AtributoDef>::generate();
        let def = AtributoDef::try_new(AtributoDefDraft {
            id,
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("medida");
        assert_eq!(def.tipo, TipoAtributo::Texto);
        assert_eq!(def.unidad.as_deref(), Some("mm"));
        assert!(def.opciones.is_none());
    }

    #[test]
    fn atributo_def_rejects_enum_without_options() {
        let err = AtributoDef::try_new(AtributoDefDraft {
            id: Id::<AtributoDef>::generate(),
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect_err("no options");
        assert_eq!(err.code(), DomainCode::InvalidInput);

        let err = AtributoDef::try_new(AtributoDefDraft {
            id: Id::<AtributoDef>::generate(),
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: Some(vec![]),
            facetable: false,
            buscable: true,
            orden: 0,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect_err("empty options");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn atributo_def_rejects_invalid_clave() {
        for clave in ["Grado", "medida-mm", "1grado"] {
            let err = AtributoDef::try_new(AtributoDefDraft {
                id: Id::<AtributoDef>::generate(),
                clave: clave.into(),
                etiqueta: "X".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
                activo: true,
                creado_en: ts(),
                actualizado_en: ts(),
            })
            .expect_err(clave);
            assert_eq!(err.code(), DomainCode::InvalidInput);
        }
    }

    #[test]
    fn atributo_def_rejects_blank_etiqueta() {
        let err = AtributoDef::try_new(AtributoDefDraft {
            id: Id::<AtributoDef>::generate(),
            clave: "medida".into(),
            etiqueta: "   ".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect_err("blank");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    struct DummyAttributeDefRepo;

    impl AttributeDefRepo for DummyAttributeDefRepo {
        fn create(
            &self,
            _input: NewAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn get(
            &self,
            _id: Id<AtributoDef>,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async { Err(DomainError::not_found()) }
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send {
            async { Ok(Vec::new()) }
        }

        fn update(
            &self,
            _input: UpdateAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn set_activo(
            &self,
            _id: Id<AtributoDef>,
            _activo: bool,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }
    }

    #[test]
    fn attribute_def_repo_trait_exposes_create_get_list_all_update_set_activo() {
        fn assert_repo<R: AttributeDefRepo>(r: &R) {
            let _f1 = r.create(NewAtributoDef {
                clave: "grado".into(),
                etiqueta: "Grado".into(),
                tipo: TipoAtributo::Enum,
                unidad: None,
                opciones: Some(vec!["2".into()]),
                facetable: false,
                buscable: true,
                orden: 0,
            });
            let _f2 = r.get(Id::generate());
            let _f3 = r.list_all();
            let _f4 = r.update(UpdateAtributoDef {
                id: Id::generate(),
                clave: "grado".into(),
                etiqueta: "Grado".into(),
                tipo: TipoAtributo::Enum,
                unidad: None,
                opciones: Some(vec!["2".into()]),
                facetable: false,
                buscable: true,
                orden: 0,
            });
            let _f5 = r.set_activo(Id::generate(), false);
        }
        assert_repo(&DummyAttributeDefRepo);
    }
}
