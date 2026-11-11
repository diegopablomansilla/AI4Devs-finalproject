//! `vortex serve` — HTTP listener and graceful shutdown (T-00-10).

use std::path::{Path, PathBuf};

use std::sync::Arc;

use api::{AppState, build_router};
use axum::serve;
use supervisor::HealthRegistry;
use tokio::net::TcpListener;

use crate::config::{Config, ConfigError};

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

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let exit_code = rt.block_on(serve_config(config));
    std::process::exit(exit_code);
}

async fn serve_config(config: Config) -> i32 {
    let bind = match config.server.bind.parse::<std::net::SocketAddr>() {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("config: server.bind: debe ser una dirección host:puerto válida");
            return 1;
        }
    };

    let listener = match TcpListener::bind(bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("config: server.bind: {e}");
            return 1;
        }
    };

    let health = Arc::new(HealthRegistry::new(env!("CARGO_PKG_VERSION")));
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
