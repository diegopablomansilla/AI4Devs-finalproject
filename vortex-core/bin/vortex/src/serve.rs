//! `vortex serve` — ordered boot (T-00-20) and graceful shutdown (T-00-10).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use api::{AppState, build_router};
use axum::serve;
use storage::{DB_DIR_NAME, Db};
use supervisor::{ComponentId, ComponentStatus, HealthRegistry};
use tokio::net::TcpListener;

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

    // Retain Db for the process lifetime (composition root; not injected into AppState yet).
    let _db = db;

    let bind = match config.server.bind.parse::<std::net::SocketAddr>() {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("config: server.bind: debe ser una dirección host:puerto válida");
            return 1;
        }
    };

    let api_start = Instant::now();
    let listener = match TcpListener::bind(bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("config: server.bind: {e}");
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
        .with_version(env!("CARGO_PKG_VERSION"));
    let router = build_router(state);
    if let Err(e) = serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
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
