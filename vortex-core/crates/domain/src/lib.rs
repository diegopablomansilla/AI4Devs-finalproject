//! Crate `domain` — entities, ports and domain errors.

pub mod atributo_def;
pub mod categoria;
pub mod decimal;
pub mod error;
pub mod health;
pub mod id;
pub mod marca;
pub mod movimiento_stock;
pub mod outbox;
pub mod producto;
pub mod timestamp;

pub use atributo_def::{
    AtributoDef, AtributoDefDraft, AttributeDefRepo, NewAtributoDef, TipoAtributo,
    UpdateAtributoDef,
};
pub use categoria::{
    Categoria, CategoriaDraft, CategoryRepo, MAX_DEPTH, NewCategoria, Slug, UpdateCategoria,
    category_depth, reject_cycle, would_form_cycle,
};
pub use decimal::{Decimal, InvalidDecimal};
pub use error::{
    ConflictCode, DomainCode, DomainError, FieldError, FieldErrorValidationError, InvalidInputCode,
    UnavailableCode, UnknownDomainCode,
};
pub use health::{
    AggregateStatus, ComponentHealth, ComponentId, ComponentStatus, HealthComponents, HealthReport,
    HealthSnapshot, aggregate,
};
pub use id::{Id, InvalidId};
pub use marca::{BrandRepo, Marca, MarcaDraft, NewMarca, UpdateMarca};
pub use movimiento_stock::{
    MovimientoStock, MovimientoStockDraft, NewMovimientoStock, StockLedger, TipoMovimiento, Venta,
};
pub use outbox::{OutboxEntity, OutboxEvent, OutboxEventError, OutboxOp};
pub use producto::{
    AtributoValor, Codigo, NewProducto, ProductRepo, Producto, ProductoDraft, Sku, TipoCodigo,
    Ubicacion, UnidadMedida, ValorAtributo,
};
pub use timestamp::{InvalidTimestamp, Timestamp};
