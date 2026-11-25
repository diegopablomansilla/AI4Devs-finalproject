//! Crate `domain` — entities, ports and domain errors.

pub mod atributo_def;
pub mod attribute_usage;
pub mod categoria;
pub mod decimal;
pub mod error;
pub mod health;
pub mod id;
pub mod marca;
pub mod movimiento_stock;
pub mod outbox;
pub mod outbox_repo;
pub mod producto;
pub mod search_document;
pub mod text_search;
pub mod timestamp;

pub use atributo_def::{
    AtributoDef, AtributoDefDraft, AttributeDefRepo, NewAtributoDef, TipoAtributo,
    UpdateAtributoDef,
};
pub use attribute_usage::AttributeUsage;
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
pub use outbox_repo::{OUTBOX_BATCH_LIMIT, OutboxRecord, OutboxRepo, OutboxRow};
pub use producto::{
    AtributoFicha, AtributoValor, Codigo, FichaProducto, FiltroProducto, NewProducto, ProductRepo,
    Producto, ProductoDraft, ResumenProducto, Sku, TipoCodigo, Ubicacion, UnidadMedida,
    UpdateProducto, ValorAtributo, valor_compatible_con,
};
pub use search_document::{CategoriaMeiliLevels, SearchDocument, build_search_document};
pub use text_search::{
    IndexSettings, IndexUid, SearchFacets, SearchFilters, SearchHit, SearchOrden, SearchPage,
    SearchQuery, TextSearch, validate_index_swap,
};
pub use timestamp::{InvalidTimestamp, Timestamp};
