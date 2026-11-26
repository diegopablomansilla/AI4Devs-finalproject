//! In-memory `SearchGateway` for HTTP integration tests (T-03-13).

use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use domain::error::DomainError;
use domain::producto::UnidadMedida;
use domain::{
    Id, SearchFilters, SearchHighlight, SearchHit, SearchOrden, SearchPage, UnavailableCode,
};
use search::SearchGateway;

const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

#[derive(Debug, Clone)]
pub struct RecordedSearch {
    pub q: String,
    pub filtros: SearchFilters,
    pub orden: SearchOrden,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FakeSearchMode {
    Respond,
    Unavailable,
    HangUntilDrop,
    Slow,
}

struct FakeSearchInner {
    mode: FakeSearchMode,
    page: SearchPage,
    last: Option<RecordedSearch>,
    dropped: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct FakeSearchGateway {
    inner: Arc<Mutex<FakeSearchInner>>,
}

impl FakeSearchGateway {
    pub fn perno_hit() -> Self {
        let hit = SearchHit {
            id: Id::from_str(CANONICAL_ULID).expect("ulid"),
            sku: "SKU-PERNO".into(),
            nombre: "Perno hexagonal".into(),
            tipo: "perno".into(),
            marca: Some("Fixser".into()),
            codigos: vec![],
            atributos_texto: vec![],
            precio: 12.5,
            unidad_medida: UnidadMedida::Unidad.as_str().to_string(),
            stock_disponible: 40.0,
            con_stock: true,
            ubicacion: None,
            imagen: None,
            resaltado: Some(SearchHighlight {
                nombre: Some("<mark>Perno</mark> hexagonal".into()),
                descripcion: None,
            }),
        };
        let page = SearchPage {
            hits: vec![hit],
            estimado_total: 1,
            offset: 0,
            limit: 20,
            tiempo_ms: 3,
            facetas: None,
        };
        Self::with_page(page)
    }

    pub fn with_page(page: SearchPage) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FakeSearchInner {
                mode: FakeSearchMode::Respond,
                page,
                last: None,
                dropped: Arc::new(AtomicBool::new(false)),
            })),
        }
    }

    pub fn unavailable() -> Self {
        let mut gw = Self::with_page(empty_page());
        gw.set_mode(FakeSearchMode::Unavailable);
        gw
    }

    pub fn hang_until_drop() -> Self {
        let mut gw = Self::with_page(empty_page());
        gw.set_mode(FakeSearchMode::HangUntilDrop);
        gw
    }

    pub fn slow() -> Self {
        let mut gw = Self::with_page(empty_page());
        gw.set_mode(FakeSearchMode::Slow);
        gw
    }

    pub fn set_mode(&mut self, mode: FakeSearchMode) {
        self.inner.lock().expect("lock").mode = mode;
    }

    pub fn take_last(&self) -> Option<RecordedSearch> {
        self.inner.lock().expect("lock").last.clone()
    }

    pub fn was_dropped(&self) -> bool {
        self.inner
            .lock()
            .expect("lock")
            .dropped
            .load(Ordering::SeqCst)
    }

    pub fn search_started(&self) -> bool {
        self.inner.lock().expect("lock").last.is_some()
    }
}

fn empty_page() -> SearchPage {
    SearchPage {
        hits: Vec::new(),
        estimado_total: 0,
        offset: 0,
        limit: 20,
        tiempo_ms: 1,
        facetas: None,
    }
}

struct DropSignal(Arc<AtomicBool>);

impl Drop for DropSignal {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl SearchGateway for FakeSearchGateway {
    fn search(
        &self,
        q: String,
        filtros: SearchFilters,
        orden: SearchOrden,
        offset: u32,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<SearchPage, DomainError>> + Send + '_>> {
        let inner = Arc::clone(&self.inner);
        Box::pin(async move {
            let mode = {
                let mut guard = inner.lock().expect("lock");
                guard.last = Some(RecordedSearch {
                    q: q.clone(),
                    filtros: filtros.clone(),
                    orden,
                    offset,
                    limit,
                });
                guard.mode
            };

            match mode {
                FakeSearchMode::Respond => {
                    let page = inner.lock().expect("lock").page.clone();
                    Ok(page)
                }
                FakeSearchMode::Unavailable => {
                    Err(DomainError::unavailable(UnavailableCode::Unavailable))
                }
                FakeSearchMode::HangUntilDrop => {
                    let dropped = {
                        let guard = inner.lock().expect("lock");
                        Arc::clone(&guard.dropped)
                    };
                    let _signal = DropSignal(dropped);
                    std::future::pending::<()>().await;
                    unreachable!("hang future")
                }
                FakeSearchMode::Slow => {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    Ok(inner.lock().expect("lock").page.clone())
                }
            }
        })
    }
}

pub fn test_search_gateway() -> Arc<dyn SearchGateway> {
    Arc::new(FakeSearchGateway::perno_hit())
}
