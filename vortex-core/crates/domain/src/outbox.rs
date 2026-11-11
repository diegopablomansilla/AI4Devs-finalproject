//! Outbox change event for catalog sync (HU-03-01, T-03-02).

use std::fmt;

use crate::id::{Id, InvalidId};

/// Target entity named in an outbox row (model 4.8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutboxEntity {
    Producto,
    Categoria,
    Marca,
    AtributoDef,
    Sinonimo,
    NotaTecnica,
    ImagenProducto,
}

impl OutboxEntity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Producto => "producto",
            Self::Categoria => "categoria",
            Self::Marca => "marca",
            Self::AtributoDef => "atributo_def",
            Self::Sinonimo => "sinonimo",
            Self::NotaTecnica => "nota_tecnica",
            Self::ImagenProducto => "imagen_producto",
        }
    }

    pub fn try_from_str(s: &str) -> Result<Self, OutboxEventError> {
        match s {
            "producto" => Ok(Self::Producto),
            "categoria" => Ok(Self::Categoria),
            "marca" => Ok(Self::Marca),
            "atributo_def" => Ok(Self::AtributoDef),
            "sinonimo" => Ok(Self::Sinonimo),
            "nota_tecnica" => Ok(Self::NotaTecnica),
            "imagen_producto" => Ok(Self::ImagenProducto),
            _ => Err(OutboxEventError::InvalidEntity),
        }
    }
}

impl fmt::Display for OutboxEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Outbox operation (`upsert` or `delete`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutboxOp {
    Upsert,
    Delete,
}

impl OutboxOp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upsert => "upsert",
            Self::Delete => "delete",
        }
    }

    pub fn try_from_str(s: &str) -> Result<Self, OutboxEventError> {
        match s {
            "upsert" => Ok(Self::Upsert),
            "delete" => Ok(Self::Delete),
            _ => Err(OutboxEventError::InvalidOperation),
        }
    }
}

impl fmt::Display for OutboxOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutboxRecord;

/// Immutable outbox payload emitted with catalog writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEvent {
    pub entidad: OutboxEntity,
    pub registro: String,
    pub operacion: OutboxOp,
    pub motivo: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxEventError {
    InvalidEntity,
    InvalidOperation,
    InvalidRegistro,
}

impl OutboxEvent {
    pub fn try_new(
        entidad: OutboxEntity,
        operacion: OutboxOp,
        registro: impl AsRef<str>,
        motivo: Option<String>,
    ) -> Result<Self, OutboxEventError> {
        let registro = registro.as_ref();
        if registro.contains(':') {
            return Err(OutboxEventError::InvalidRegistro);
        }
        let _: Id<OutboxRecord> = registro
            .parse()
            .map_err(|InvalidId| OutboxEventError::InvalidRegistro)?;
        Ok(Self {
            entidad,
            registro: registro.to_string(),
            operacion,
            motivo,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

    #[test]
    fn outbox_event_valid_producto_upsert() {
        let ev = OutboxEvent::try_new(
            OutboxEntity::Producto,
            OutboxOp::Upsert,
            CANONICAL_ULID,
            Some("stock".into()),
        )
        .expect("valid event");
        assert_eq!(ev.entidad, OutboxEntity::Producto);
        assert_eq!(ev.operacion, OutboxOp::Upsert);
        assert_eq!(ev.registro, CANONICAL_ULID);
        assert_eq!(ev.motivo.as_deref(), Some("stock"));
    }

    #[test]
    fn outbox_event_motivo_none() {
        let ev = OutboxEvent::try_new(
            OutboxEntity::Producto,
            OutboxOp::Upsert,
            CANONICAL_ULID,
            None,
        )
        .expect("valid event");
        assert!(ev.motivo.is_none());
    }

    #[test]
    fn outbox_event_rejects_table_prefix() {
        assert!(
            OutboxEvent::try_new(
                OutboxEntity::Producto,
                OutboxOp::Upsert,
                "producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8",
                None,
            )
            .is_err()
        );
    }

    #[test]
    fn outbox_event_rejects_unknown_entidad_or_operacion() {
        assert!(OutboxEntity::try_from_str("foo").is_err());
        assert!(OutboxOp::try_from_str("update").is_err());
    }
}
