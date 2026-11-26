//! Stock movement entity, sign rules and immutable `StockLedger` port (T-02-17).

use std::future::Future;
use std::str::FromStr;

use crate::decimal::Decimal;
use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::id::Id;
use crate::producto::Producto;
use crate::timestamp::Timestamp;

const MAX_MOTIVO_LEN: usize = 500;

/// Sale entity marker until T-08-01.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Venta;

/// Movement type (model 4.5.1 — Spanish wire values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoMovimiento {
    Inicial,
    Ingreso,
    Ajuste,
    Venta,
}

impl TipoMovimiento {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inicial => "inicial",
            Self::Ingreso => "ingreso",
            Self::Ajuste => "ajuste",
            Self::Venta => "venta",
        }
    }

    pub fn parse_wire(s: &str) -> Result<Self, DomainError> {
        match s {
            "inicial" => Ok(Self::Inicial),
            "ingreso" => Ok(Self::Ingreso),
            "ajuste" => Ok(Self::Ajuste),
            "venta" => Ok(Self::Venta),
            _ => Err(field_invalid_input("tipo", "invalid tipo movimiento")),
        }
    }
}

impl FromStr for TipoMovimiento {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_wire(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovimientoStockDraft {
    pub id: Id<MovimientoStock>,
    pub producto: Id<Producto>,
    pub tipo: TipoMovimiento,
    pub cantidad: Decimal,
    pub stock_resultante: Decimal,
    pub motivo: Option<String>,
    pub venta: Option<Id<Venta>>,
    pub terminal: Option<String>,
    pub creado_en: Timestamp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovimientoStock {
    pub id: Id<MovimientoStock>,
    pub producto: Id<Producto>,
    pub tipo: TipoMovimiento,
    pub cantidad: Decimal,
    pub stock_resultante: Decimal,
    pub motivo: Option<String>,
    pub venta: Option<Id<Venta>>,
    pub terminal: Option<String>,
    pub creado_en: Timestamp,
}

impl MovimientoStock {
    pub fn try_new(draft: MovimientoStockDraft) -> Result<Self, DomainError> {
        let MovimientoStockDraft {
            id,
            producto,
            tipo,
            cantidad,
            stock_resultante,
            motivo,
            venta,
            terminal,
            creado_en,
        } = draft;

        validate_cantidad_sign(tipo, cantidad)?;
        validate_motivo(tipo, motivo.as_deref())?;
        validate_venta_ref(tipo, venta.as_ref())?;
        validate_stock_resultante(stock_resultante)?;

        let motivo = normalize_motivo(motivo)?;

        Ok(Self {
            id,
            producto,
            tipo,
            cantidad,
            stock_resultante,
            motivo,
            venta,
            terminal,
            creado_en,
        })
    }
}

/// Input for `StockLedger::append` (adapter computes `stock_resultante` in T-02-24).
#[derive(Debug, Clone, PartialEq)]
pub struct NewMovimientoStock {
    pub producto: Id<Producto>,
    pub tipo: TipoMovimiento,
    pub cantidad: Decimal,
    pub motivo: Option<String>,
    pub venta: Option<Id<Venta>>,
    pub terminal: Option<String>,
}

pub trait StockLedger {
    fn append(
        &self,
        input: NewMovimientoStock,
    ) -> impl Future<Output = Result<MovimientoStock, DomainError>> + Send;

    fn list_by_producto(
        &self,
        producto: Id<Producto>,
    ) -> impl Future<Output = Result<Vec<MovimientoStock>, DomainError>> + Send;
}

fn zero() -> Decimal {
    Decimal::from_str("0").expect("zero")
}

fn validate_cantidad_sign(tipo: TipoMovimiento, cantidad: Decimal) -> Result<(), DomainError> {
    let z = zero();
    if cantidad == z {
        return Err(field_invalid_input("cantidad", "cantidad must not be zero"));
    }
    let bad_sign = match tipo {
        TipoMovimiento::Inicial | TipoMovimiento::Ingreso => cantidad < z,
        TipoMovimiento::Ajuste => false,
        TipoMovimiento::Venta => cantidad > z,
    };
    if bad_sign {
        return Err(field_invalid_input(
            "cantidad",
            "invalid cantidad sign for tipo",
        ));
    }
    Ok(())
}

fn validate_motivo(tipo: TipoMovimiento, motivo: Option<&str>) -> Result<(), DomainError> {
    match tipo {
        TipoMovimiento::Ajuste => {
            let Some(raw) = motivo else {
                return Err(field_invalid_input(
                    "motivo",
                    "motivo is required for ajuste",
                ));
            };
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(field_invalid_input("motivo", "motivo must not be empty"));
            }
            if trimmed.len() > MAX_MOTIVO_LEN {
                return Err(field_invalid_input(
                    "motivo",
                    "motivo must be at most 500 characters",
                ));
            }
        }
        _ => {
            if let Some(raw) = motivo {
                let trimmed = raw.trim();
                if !trimmed.is_empty() && trimmed.len() > MAX_MOTIVO_LEN {
                    return Err(field_invalid_input(
                        "motivo",
                        "motivo must be at most 500 characters",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn normalize_motivo(motivo: Option<String>) -> Result<Option<String>, DomainError> {
    match motivo {
        None => Ok(None),
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
    }
}

fn validate_venta_ref(tipo: TipoMovimiento, venta: Option<&Id<Venta>>) -> Result<(), DomainError> {
    match tipo {
        TipoMovimiento::Venta => {
            if venta.is_none() {
                return Err(field_invalid_input(
                    "venta",
                    "venta is required for tipo venta",
                ));
            }
        }
        _ => {
            if venta.is_some() {
                return Err(field_invalid_input(
                    "venta",
                    "venta must be absent unless tipo is venta",
                ));
            }
        }
    }
    Ok(())
}

fn validate_stock_resultante(stock_resultante: Decimal) -> Result<(), DomainError> {
    if stock_resultante < zero() {
        return Err(DomainError::invalid_input(
            InvalidInputCode::NegativeStock,
            vec![FieldError::try_new("stock_resultante", "must not be negative").expect("field")],
        ));
    }
    Ok(())
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

    fn dec(s: &str) -> Decimal {
        s.parse().expect("decimal")
    }

    fn draft(
        tipo: TipoMovimiento,
        cantidad: &str,
        stock_resultante: &str,
        motivo: Option<&str>,
        venta: Option<Id<Venta>>,
    ) -> MovimientoStockDraft {
        MovimientoStockDraft {
            id: Id::generate(),
            producto: Id::generate(),
            tipo,
            cantidad: dec(cantidad),
            stock_resultante: dec(stock_resultante),
            motivo: motivo.map(str::to_string),
            venta,
            terminal: None,
            creado_en: ts(),
        }
    }

    fn invalid_fields(err: &DomainError) -> &[FieldError] {
        match err {
            DomainError::InvalidInput { errors, .. } => errors,
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn movimiento_ingreso_guarda_terminal_caja_1() {
        let mov = MovimientoStock::try_new(MovimientoStockDraft {
            id: Id::generate(),
            producto: Id::generate(),
            tipo: TipoMovimiento::Ingreso,
            cantidad: dec("20"),
            stock_resultante: dec("30"),
            motivo: None,
            venta: None,
            terminal: Some("caja-1".into()),
            creado_en: ts(),
        })
        .expect("ingreso");
        assert_eq!(mov.tipo, TipoMovimiento::Ingreso);
        assert_eq!(mov.cantidad.to_string(), "20");
        assert_eq!(mov.stock_resultante.to_string(), "30");
        assert_eq!(mov.terminal.as_deref(), Some("caja-1"));
        assert!(!mov.id.to_string().contains(':'));
    }

    #[test]
    fn movimiento_inicial_cantidad_positiva() {
        let mov = MovimientoStock::try_new(draft(TipoMovimiento::Inicial, "50", "50", None, None))
            .expect("inicial");
        assert_eq!(mov.tipo, TipoMovimiento::Inicial);
        assert_eq!(mov.cantidad.to_string(), "50");
    }

    #[test]
    fn movimiento_ajuste_negativo_con_motivo() {
        let mov = MovimientoStock::try_new(draft(
            TipoMovimiento::Ajuste,
            "-2",
            "0",
            Some("Conteo físico del pasillo 3"),
            None,
        ))
        .expect("ajuste");
        assert_eq!(mov.tipo, TipoMovimiento::Ajuste);
        assert_eq!(mov.cantidad.to_string(), "-2");
        assert_eq!(mov.motivo.as_deref(), Some("Conteo físico del pasillo 3"));
    }

    #[test]
    fn movimiento_venta_cantidad_negativa() {
        let venta_id = Id::<Venta>::generate();
        let mov = MovimientoStock::try_new(draft(
            TipoMovimiento::Venta,
            "-1",
            "0",
            None,
            Some(venta_id),
        ))
        .expect("venta");
        assert_eq!(mov.tipo, TipoMovimiento::Venta);
        assert_eq!(mov.cantidad.to_string(), "-1");
        assert_eq!(mov.venta, Some(venta_id));
    }

    #[test]
    fn movimiento_rejects_ingreso_cantidad_negativa() {
        let err = MovimientoStock::try_new(draft(TipoMovimiento::Ingreso, "-5", "0", None, None))
            .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "cantidad");
    }

    #[test]
    fn movimiento_rejects_inicial_cantidad_negativa() {
        let err = MovimientoStock::try_new(draft(TipoMovimiento::Inicial, "-1", "0", None, None))
            .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "cantidad");
    }

    #[test]
    fn movimiento_rejects_venta_cantidad_positiva() {
        let err = MovimientoStock::try_new(draft(
            TipoMovimiento::Venta,
            "3",
            "3",
            None,
            Some(Id::generate()),
        ))
        .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "cantidad");
    }

    #[test]
    fn movimiento_rejects_cantidad_cero() {
        let err = MovimientoStock::try_new(draft(TipoMovimiento::Ingreso, "0", "10", None, None))
            .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "cantidad");
    }

    #[test]
    fn tipo_movimiento_rejects_english_or_unknown() {
        assert!(TipoMovimiento::parse_wire("inbound").is_err());
        assert!(TipoMovimiento::parse_wire("adjustment").is_err());
        let err = TipoMovimiento::parse_wire("inbound").unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn movimiento_rejects_ajuste_sin_motivo() {
        let err = MovimientoStock::try_new(draft(TipoMovimiento::Ajuste, "-8", "0", None, None))
            .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "motivo");
    }

    #[test]
    fn movimiento_rejects_stock_resultante_negativo() {
        let err = MovimientoStock::try_new(draft(TipoMovimiento::Ingreso, "5", "-1", None, None))
            .unwrap_err();
        assert_eq!(err.code(), DomainCode::NegativeStock);
    }

    #[test]
    fn movimiento_rejects_venta_sin_referencia() {
        let err = MovimientoStock::try_new(draft(TipoMovimiento::Venta, "-1", "0", None, None))
            .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "venta");
    }

    #[test]
    fn movimiento_rejects_ingreso_con_venta() {
        let err = MovimientoStock::try_new(draft(
            TipoMovimiento::Ingreso,
            "10",
            "10",
            None,
            Some(Id::generate()),
        ))
        .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "venta");
    }

    struct DummyStockLedger;

    impl StockLedger for DummyStockLedger {
        fn append(
            &self,
            _input: NewMovimientoStock,
        ) -> impl Future<Output = Result<MovimientoStock, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn list_by_producto(
            &self,
            _producto: Id<Producto>,
        ) -> impl Future<Output = Result<Vec<MovimientoStock>, DomainError>> + Send {
            async { Ok(vec![]) }
        }
    }

    #[test]
    fn stock_ledger_trait_exposes_append_and_list() {
        fn assert_ledger<L: StockLedger>(l: &L) {
            let _f1 = l.append(NewMovimientoStock {
                producto: Id::generate(),
                tipo: TipoMovimiento::Ingreso,
                cantidad: dec("1"),
                motivo: None,
                venta: None,
                terminal: None,
            });
            let _f2 = l.list_by_producto(Id::generate());
        }
        assert_ledger(&DummyStockLedger);
    }

    #[test]
    fn stock_ledger_trait_has_no_update_or_delete() {
        let src = include_str!("movimiento_stock.rs");
        let prod = src.split("#[cfg(test)]").next().expect("production source");
        assert!(!prod.contains("fn update("));
        assert!(!prod.contains("fn delete("));
    }
}
