//! Canned `RecommendationGateway` for HTTP integration tests (T-04-07).

use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use domain::error::DomainError;
use domain::{Id, Producto, ProductoRecomendado, Recomendaciones, UnavailableCode};
use recommend::RecommendationGateway;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FakeRecommendationMode {
    Respond,
    NotFound,
    Unavailable,
}

type PanelMap = Vec<(Id<Producto>, Recomendaciones)>;

pub struct FakeRecommendationGateway {
    mode: Arc<Mutex<FakeRecommendationMode>>,
    panels: Arc<Mutex<PanelMap>>,
}

impl FakeRecommendationGateway {
    pub fn with_panel(product: Id<Producto>, panel: Recomendaciones) -> Self {
        Self {
            mode: Arc::new(Mutex::new(FakeRecommendationMode::Respond)),
            panels: Arc::new(Mutex::new(vec![(product, panel)])),
        }
    }

    pub fn empty_edges(product: Id<Producto>) -> Self {
        Self::with_panel(
            product.clone(),
            Recomendaciones {
                producto_id: product,
                destacar_sustitutos: false,
                similares: vec![],
                sustitutos: vec![],
                complementarios: vec![],
            },
        )
    }

    pub fn unavailable() -> Self {
        let gw = Self::empty_edges(Id::generate());
        gw.set_mode(FakeRecommendationMode::Unavailable);
        gw
    }

    pub fn not_found_all() -> Self {
        let gw = Self {
            mode: Arc::new(Mutex::new(FakeRecommendationMode::NotFound)),
            panels: Arc::new(Mutex::new(vec![])),
        };
        gw
    }

    pub fn set_mode(&self, mode: FakeRecommendationMode) {
        *self.mode.lock().expect("lock") = mode;
    }
}

impl RecommendationGateway for FakeRecommendationGateway {
    fn get(
        &self,
        product: Id<Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<Recomendaciones, DomainError>> + Send + '_>> {
        let mode = *self.mode.lock().expect("lock");
        let panels = Arc::clone(&self.panels);
        Box::pin(async move {
            match mode {
                FakeRecommendationMode::Unavailable => {
                    Err(DomainError::unavailable(UnavailableCode::Unavailable))
                }
                FakeRecommendationMode::NotFound => Err(DomainError::not_found()),
                FakeRecommendationMode::Respond => panels
                    .lock()
                    .expect("lock")
                    .iter()
                    .find(|(id, _)| id == &product)
                    .map(|(_, panel)| panel.clone())
                    .ok_or_else(DomainError::not_found),
            }
        })
    }
}

pub fn sample_item(
    sku: &str,
    name: &str,
    stock: &str,
    price: &str,
    priority: i32,
) -> ProductoRecomendado {
    ProductoRecomendado {
        relacion_id: Id::generate(),
        id: Id::generate(),
        sku: sku.into(),
        nombre: name.into(),
        stock: stock.parse().expect("stock"),
        precio: price.parse().expect("price"),
        ubicacion: Some("P3 · E-B · 12".into()),
        prioridad: priority,
        nota: None,
    }
}

pub fn three_blocks_panel(product: Id<Producto>) -> Recomendaciones {
    let mut similar = sample_item("SIM-1", "Similar 1", "5", "10", 2);
    let mut similar2 = sample_item("SIM-2", "Similar 2", "3", "9", 1);
    similar.prioridad = 2;
    similar2.prioridad = 1;
    Recomendaciones {
        producto_id: product,
        destacar_sustitutos: false,
        similares: vec![similar2, similar],
        sustitutos: vec![sample_item("SUB-1", "Sub 1", "12", "8", 1)],
        complementarios: vec![sample_item("CMP-1", "Comp 1", "20", "2", 1)],
    }
}

pub fn highlight_substitutes_panel(product: Id<Producto>) -> Recomendaciones {
    Recomendaciones {
        producto_id: product,
        destacar_sustitutos: true,
        similares: vec![],
        sustitutos: vec![sample_item("SUB-A", "Sub A", "12", "8", 1)],
        complementarios: vec![],
    }
}

pub fn panel_excludes_inactive(product: Id<Producto>) -> Recomendaciones {
    Recomendaciones {
        producto_id: product,
        destacar_sustitutos: false,
        similares: vec![sample_item("ACT", "Activo", "1", "1", 1)],
        sustitutos: vec![],
        complementarios: vec![],
    }
}

pub const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

pub fn canonical_product_id() -> Id<Producto> {
    Id::from_str(CANONICAL_ULID).expect("ulid")
}
