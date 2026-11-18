//! Stock movement registration (T-02-25).

use std::future::Future;
use std::pin::Pin;

use domain::{
    DomainError, FieldError, InvalidInputCode, MovimientoStock, NewMovimientoStock, StockLedger,
    TipoMovimiento,
};

pub struct StockService<L> {
    ledger: L,
}

impl<L> StockService<L>
where
    L: StockLedger,
{
    pub fn new(ledger: L) -> Self {
        Self { ledger }
    }

    pub async fn register(
        &self,
        input: NewMovimientoStock,
    ) -> Result<MovimientoStock, DomainError> {
        reject_disallowed_tipo(input.tipo)?;
        self.ledger.append(input).await
    }
}

fn reject_disallowed_tipo(tipo: TipoMovimiento) -> Result<(), DomainError> {
    match tipo {
        TipoMovimiento::Ingreso | TipoMovimiento::Ajuste => Ok(()),
        TipoMovimiento::Inicial | TipoMovimiento::Venta => Err(DomainError::invalid_input(
            InvalidInputCode::InvalidInput,
            vec![
                FieldError::try_new("tipo", "tipo not allowed for manual registration")
                    .expect("field"),
            ],
        )),
    }
}

/// Object-safe facade for HTTP injection (`Arc<dyn StockGateway>`).
pub trait StockGateway: Send + Sync {
    fn register(
        &self,
        input: NewMovimientoStock,
    ) -> Pin<Box<dyn Future<Output = Result<MovimientoStock, DomainError>> + Send + '_>>;
}

impl<L> StockGateway for StockService<L>
where
    L: StockLedger + Send + Sync,
{
    fn register(
        &self,
        input: NewMovimientoStock,
    ) -> Pin<Box<dyn Future<Output = Result<MovimientoStock, DomainError>> + Send + '_>> {
        Box::pin(StockService::register(self, input))
    }
}

pub mod testing {
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    use domain::{
        Decimal, DomainError, Id, InvalidInputCode, MovimientoStock, MovimientoStockDraft,
        NewMovimientoStock, Producto, StockLedger, Timestamp, TipoMovimiento,
    };

    use super::StockService;

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    fn zero() -> Decimal {
        Decimal::from_str("0").expect("zero")
    }

    pub struct InMemoryStockLedger {
        products: Arc<Mutex<HashMap<String, Producto>>>,
    }

    impl InMemoryStockLedger {
        pub fn new(products: Arc<Mutex<HashMap<String, Producto>>>) -> Self {
            Self { products }
        }

        pub fn with_product(producto: Producto) -> Self {
            let products = Arc::new(Mutex::new(HashMap::new()));
            products
                .lock()
                .expect("lock")
                .insert(producto.id.to_string(), producto);
            Self { products }
        }
    }

    impl StockLedger for InMemoryStockLedger {
        fn append(
            &self,
            input: NewMovimientoStock,
        ) -> impl Future<Output = Result<MovimientoStock, DomainError>> + Send {
            let products = Arc::clone(&self.products);
            async move {
                let mut map = products.lock().expect("lock");
                let Some(product) = map.get(&input.producto.to_string()).cloned() else {
                    return Err(DomainError::not_found());
                };
                let stock_before = product.stock;
                let sum = stock_before.inner() + input.cantidad.inner();
                let nuevo: Decimal = sum.normalize().to_string().parse().expect("stock sum");
                if nuevo < zero() {
                    return Err(DomainError::invalid_input(
                        InvalidInputCode::NegativeStock,
                        vec![],
                    ));
                }
                let mov = MovimientoStock::try_new(MovimientoStockDraft {
                    id: Id::generate(),
                    producto: input.producto.clone(),
                    tipo: input.tipo,
                    cantidad: input.cantidad,
                    stock_resultante: nuevo,
                    motivo: input.motivo.clone(),
                    venta: input.venta,
                    terminal: input.terminal.clone(),
                    creado_en: ts(),
                })?;
                let mut updated = product;
                updated.stock = nuevo;
                map.insert(input.producto.to_string(), updated);
                Ok(mov)
            }
        }

        async fn list_by_producto(
            &self,
            _producto: Id<Producto>,
        ) -> Result<Vec<MovimientoStock>, DomainError> {
            Ok(vec![])
        }
    }

    pub fn sample_product(stock: &str) -> Producto {
        Producto {
            id: Id::generate(),
            sku: domain::Sku::try_new("SKU-TEST").expect("sku"),
            nombre: "Test".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            atributos: vec![],
            codigos: vec![],
            unidad_medida: domain::UnidadMedida::Unidad,
            precio: "1".parse().expect("precio"),
            ubicacion: None,
            stock: stock.parse().expect("stock"),
            activo: true,
            version: 1,
            creado_en: ts(),
            actualizado_en: ts(),
        }
    }

    pub fn ingreso(
        producto: Id<Producto>,
        cantidad: &str,
        terminal: Option<&str>,
    ) -> NewMovimientoStock {
        NewMovimientoStock {
            producto,
            tipo: TipoMovimiento::Ingreso,
            cantidad: cantidad.parse().expect("cantidad"),
            motivo: None,
            venta: None,
            terminal: terminal.map(str::to_string),
        }
    }

    pub fn ajuste(
        producto: Id<Producto>,
        cantidad: &str,
        motivo: Option<&str>,
    ) -> NewMovimientoStock {
        NewMovimientoStock {
            producto,
            tipo: TipoMovimiento::Ajuste,
            cantidad: cantidad.parse().expect("cantidad"),
            motivo: motivo.map(str::to_string),
            venta: None,
            terminal: None,
        }
    }

    pub fn invalid_fields(err: &DomainError) -> &[domain::FieldError] {
        match err {
            DomainError::InvalidInput { errors, .. } => errors,
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    pub type TestStockService = StockService<InMemoryStockLedger>;

    pub fn svc(ledger: InMemoryStockLedger) -> TestStockService {
        StockService::new(ledger)
    }
}

#[cfg(test)]
mod tests {
    use domain::{DomainCode, Id, NewMovimientoStock, TipoMovimiento, Venta};

    use super::testing::{
        InMemoryStockLedger, ajuste, ingreso, invalid_fields, sample_product, svc,
    };

    #[tokio::test]
    async fn catalog_register_inbound_sets_resulting_stock_and_terminal() {
        let product = sample_product("10");
        let product_id = product.id.clone();
        let service = svc(InMemoryStockLedger::with_product(product));
        let mov = service
            .register(ingreso(product_id, "20", Some("caja-1")))
            .await
            .expect("register");
        assert_eq!(mov.tipo, TipoMovimiento::Ingreso);
        assert_eq!(mov.cantidad.to_string(), "20");
        assert_eq!(mov.stock_resultante.to_string(), "30");
        assert_eq!(mov.terminal.as_deref(), Some("caja-1"));
    }

    #[tokio::test]
    async fn catalog_register_adjustment_to_zero_with_reason() {
        let product = sample_product("5");
        let product_id = product.id.clone();
        let service = svc(InMemoryStockLedger::with_product(product));
        let mov = service
            .register(ajuste(product_id, "-5", Some("Conteo físico")))
            .await
            .expect("register");
        assert_eq!(mov.stock_resultante.to_string(), "0");
        assert_eq!(mov.motivo.as_deref(), Some("Conteo físico"));
    }

    #[tokio::test]
    async fn catalog_register_ajuste_sin_motivo_is_invalid_input() {
        let product = sample_product("5");
        let product_id = product.id.clone();
        let service = svc(InMemoryStockLedger::with_product(product));
        let err = service
            .register(ajuste(product_id, "-1", None))
            .await
            .expect_err("no motivo");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "motivo");
    }

    #[tokio::test]
    async fn catalog_register_ajuste_minus_8_on_stock_5_is_negative_stock() {
        let product = sample_product("5");
        let product_id = product.id.clone();
        let products = {
            let mut map = std::collections::HashMap::new();
            map.insert(product_id.to_string(), product);
            std::sync::Arc::new(std::sync::Mutex::new(map))
        };
        let service = svc(InMemoryStockLedger::new(products.clone()));
        let err = service
            .register(ajuste(product_id.clone(), "-8", Some("Conteo físico")))
            .await
            .expect_err("negative");
        assert_eq!(err.code(), DomainCode::NegativeStock);
        let stock = products
            .lock()
            .expect("lock")
            .get(&product_id.to_string())
            .expect("product")
            .stock
            .to_string();
        assert_eq!(stock, "5");
    }

    #[tokio::test]
    async fn catalog_register_inicial_is_invalid_input() {
        let product = sample_product("5");
        let product_id = product.id.clone();
        let service = svc(InMemoryStockLedger::with_product(product));
        let err = service
            .register(NewMovimientoStock {
                producto: product_id,
                tipo: TipoMovimiento::Inicial,
                cantidad: "1".parse().expect("c"),
                motivo: None,
                venta: None,
                terminal: None,
            })
            .await
            .expect_err("inicial");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "tipo");
    }

    #[tokio::test]
    async fn catalog_register_venta_is_invalid_input() {
        let product = sample_product("5");
        let product_id = product.id.clone();
        let service = svc(InMemoryStockLedger::with_product(product));
        let err = service
            .register(NewMovimientoStock {
                producto: product_id,
                tipo: TipoMovimiento::Venta,
                cantidad: "-1".parse().expect("c"),
                motivo: None,
                venta: Some(Id::<Venta>::generate()),
                terminal: None,
            })
            .await
            .expect_err("venta");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "tipo");
    }

    #[tokio::test]
    async fn catalog_register_unknown_product_is_not_found() {
        let service = svc(InMemoryStockLedger::with_product(sample_product("0")));
        let err = service
            .register(ingreso(Id::generate(), "1", None))
            .await
            .expect_err("missing");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_register_ingreso_cantidad_negativa_is_invalid_input() {
        let product = sample_product("5");
        let product_id = product.id.clone();
        let service = svc(InMemoryStockLedger::with_product(product));
        let err = service
            .register(ingreso(product_id, "-1", None))
            .await
            .expect_err("sign");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "cantidad");
    }
}
