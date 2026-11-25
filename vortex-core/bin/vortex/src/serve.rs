//! `vortex serve` — ordered boot (T-00-20, T-00-27) and graceful shutdown (T-00-10).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use std::sync::Arc;

use api::{AppState, PrometheusSyncTelemetry, build_router};
use axum::serve;
use catalog::{
    AttributeDefService, BrandService, CategoryService, ProductService, ProductionOccupancy,
};
use domain::{IndexUid, SyncTelemetry};
use search::{MeiliTextSearch, SearchService};
use storage::{
    AtributoDefRepo, CategoriaRepo, DB_DIR_NAME, Db, MarcaRepo, OutboxStore, ProductAttributeUsage,
    ProductoRepo,
};
use supervisor::{ComponentId, ComponentStatus, HealthRegistry};
use sync::{DEFAULT_POLL_INTERVAL, MeiliProductWorker};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::boot::wait_meilisearch_text_search;
use crate::config::{Config, ConfigError};
use crate::logging::{self, LoggingGuard};

pub fn resolve_config_path(flag: Option<&PathBuf>) -> Result<PathBuf, ConfigError> {
    if let Some(path) = flag {
        return Ok(path.clone());
    }
    match std::env::var("VORTEX_CONFIG") {
        Ok(p) if !p.is_empty() => Ok(PathBuf::from(p)),
        _ => Err(ConfigError::new(
            "--config",
            "falta el path de configuración (--config o VORTEX_CONFIG)",
        )),
    }
}

pub fn run(config_flag: Option<&PathBuf>) -> ! {
    let boot_config = Instant::now();
    let path = match resolve_config_path(config_flag) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let config = match Config::from_path(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let config_ms = boot_config.elapsed().as_millis() as u64;

    let logging_start = Instant::now();
    let logging_guard = match logging::init_logging(&config) {
        Ok(g) => {
            let logging_ms = logging_start.elapsed().as_millis() as u64;
            tracing::info!(
                phase = "boot.config",
                duration_ms = config_ms,
                "boot.config"
            );
            tracing::info!(
                phase = "boot.logging",
                duration_ms = logging_ms,
                "boot.logging"
            );
            g
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let exit_code = rt.block_on(serve_config(config, logging_guard));
    std::process::exit(exit_code);
}

/// Forwards metrics to Prometheus and mirrors sync stats on [`HealthRegistry`].
#[derive(Clone)]
struct RegistrySyncTelemetry {
    prom: PrometheusSyncTelemetry,
    health: Arc<HealthRegistry>,
    failed: Arc<AtomicU64>,
    last_lag: Arc<AtomicU64>,
}

impl RegistrySyncTelemetry {
    fn new(health: Arc<HealthRegistry>) -> Self {
        Self {
            prom: PrometheusSyncTelemetry,
            health,
            failed: Arc::new(AtomicU64::new(0)),
            last_lag: Arc::new(AtomicU64::new(0)),
        }
    }

    fn push_health_stats(&self) {
        self.health.set_sync_stats(
            self.last_lag.load(Ordering::Relaxed),
            self.failed.load(Ordering::Relaxed),
        );
    }
}

impl SyncTelemetry for RegistrySyncTelemetry {
    fn set_outbox_lag(&self, consumidor: &str, lag: u64) {
        self.prom.set_outbox_lag(consumidor, lag);
        self.last_lag.store(lag, Ordering::Relaxed);
        self.push_health_stats();
    }

    fn inc_outbox_failed(&self) {
        self.prom.inc_outbox_failed();
        self.failed.fetch_add(1, Ordering::Relaxed);
        self.push_health_stats();
    }

    fn observe_sync_propagation_seconds(&self, seconds: f64) {
        self.prom.observe_sync_propagation_seconds(seconds);
    }
}

async fn serve_config(config: Config, _logging_guard: LoggingGuard) -> i32 {
    let health_start = Instant::now();
    let health = Arc::new(HealthRegistry::new(env!("CARGO_PKG_VERSION")));
    health.set(ComponentId::Storage, ComponentStatus::Starting);
    tracing::info!(
        phase = "boot.health",
        duration_ms = health_start.elapsed().as_millis() as u64,
        "boot.health"
    );

    let storage_start = Instant::now();
    let db_path = Path::new(&config.paths.datos).join(DB_DIR_NAME);
    let db = match Db::open(&db_path).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("storage: {e}");
            tracing::error!(phase = "boot.storage", error = %e, "storage open failed");
            return 1;
        }
    };
    if let Err(e) = db.migrate().await {
        eprintln!("storage: {e}");
        tracing::error!(phase = "boot.storage", error = %e, "storage migrate failed");
        return 1;
    }
    health.set(ComponentId::Storage, ComponentStatus::Up);
    tracing::info!(
        phase = "boot.storage",
        duration_ms = storage_start.elapsed().as_millis() as u64,
        "boot.storage"
    );

    let db = Arc::new(db);

    let meili_start = Instant::now();
    health.set(ComponentId::Meilisearch, ComponentStatus::Starting);
    let text_search = match MeiliTextSearch::new(
        config.meilisearch.url.clone(),
        config.meilisearch.api_key.clone(),
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("meilisearch: {e}");
            tracing::error!(phase = "boot.meilisearch", error = %e, "meilisearch client failed");
            return 1;
        }
    };

    let meili_up = wait_meilisearch_text_search(&text_search).await;
    if meili_up {
        health.set(ComponentId::Meilisearch, ComponentStatus::Up);
    } else {
        health.set(ComponentId::Meilisearch, ComponentStatus::Down);
    }
    tracing::info!(
        phase = "boot.meilisearch",
        duration_ms = meili_start.elapsed().as_millis() as u64,
        meili_up,
        "boot.meilisearch"
    );

    let sync_start = Instant::now();
    let index = match IndexUid::try_new("productos") {
        Ok(i) => i,
        Err(e) => {
            eprintln!("sync: {e}");
            return 1;
        }
    };
    let telemetry = RegistrySyncTelemetry::new(Arc::clone(&health));
    let outbox = OutboxStore::new(Arc::clone(&db));
    let search_gateway: Arc<dyn search::SearchGateway> =
        Arc::new(SearchService::new(text_search.clone(), index.clone()));
    let worker = MeiliProductWorker::new(
        outbox,
        ProductoRepo::new(Arc::clone(&db)),
        AtributoDefRepo::new(Arc::clone(&db)),
        text_search,
        telemetry.clone(),
        db.outbox_notifier(),
        index,
        DEFAULT_POLL_INTERVAL,
    );
    let worker_stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&worker_stop);
    let worker_handle: JoinHandle<()> = tokio::spawn(async move {
        worker
            .run_while(|| !stop_flag.load(Ordering::Relaxed))
            .await;
    });
    health.set(ComponentId::Sync, ComponentStatus::Up);
    telemetry.push_health_stats();
    tracing::info!(
        phase = "boot.sync",
        duration_ms = sync_start.elapsed().as_millis() as u64,
        "boot.sync"
    );

    let category_gateway: Arc<dyn catalog::CategoryGateway> = Arc::new(CategoryService::new(
        CategoriaRepo::new(Arc::clone(&db)),
        ProductionOccupancy,
    ));
    let brand_gateway: Arc<dyn catalog::BrandGateway> =
        Arc::new(BrandService::new(MarcaRepo::new(Arc::clone(&db))));
    let attribute_def_gateway: Arc<dyn catalog::AttributeDefGateway> =
        Arc::new(AttributeDefService::new(
            AtributoDefRepo::new(Arc::clone(&db)),
            ProductAttributeUsage::new(Arc::clone(&db)),
        ));
    let product_gateway: Arc<dyn catalog::ProductGateway> = Arc::new(ProductService::new(
        ProductoRepo::new(Arc::clone(&db)),
        CategoriaRepo::new(Arc::clone(&db)),
        MarcaRepo::new(Arc::clone(&db)),
        AtributoDefRepo::new(Arc::clone(&db)),
    ));
    let stock_gateway: Arc<dyn catalog::StockGateway> = Arc::new(catalog::StockService::new(
        storage::MovimientoStockRepo::new(Arc::clone(&db)),
    ));

    let bind = match config.server.bind.parse::<std::net::SocketAddr>() {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("config: server.bind: debe ser una dirección host:puerto válida");
            worker_stop.store(true, Ordering::Relaxed);
            worker_handle.abort();
            return 1;
        }
    };

    let api_start = Instant::now();
    let listener = match TcpListener::bind(bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("config: server.bind: {e}");
            worker_stop.store(true, Ordering::Relaxed);
            worker_handle.abort();
            return 1;
        }
    };
    tracing::info!(
        phase = "boot.api",
        duration_ms = api_start.elapsed().as_millis() as u64,
        "boot.api"
    );

    let web_root = PathBuf::from(&config.paths.web);
    let media_root = Path::new(&config.paths.datos).join("media");
    let state = AppState::with_health(health)
        .with_static_roots(web_root, media_root)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_categories(category_gateway)
        .with_brands(brand_gateway)
        .with_attribute_defs(attribute_def_gateway)
        .with_products(product_gateway)
        .with_stock(stock_gateway)
        .with_search(search_gateway);
    let router = build_router(state);

    let stop_worker = Arc::clone(&worker_stop);
    let graceful = async move {
        shutdown_signal().await;
        stop_worker.store(true, Ordering::Relaxed);
        worker_handle.abort();
    };

    if let Err(e) = serve(listener, router)
        .with_graceful_shutdown(graceful)
        .await
    {
        eprintln!("serve: {e}");
        return 1;
    }
    0
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("ctrl_c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
