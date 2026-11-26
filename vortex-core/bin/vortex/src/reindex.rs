//! `vortex reindex` — rebuild Meilisearch `productos` from SurrealDB (T-03-11).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use domain::IndexUid;
use search::MeiliTextSearch;
use storage::{AtributoDefRepo, DB_DIR_NAME, Db, OutboxStore, ProductoRepo};
use sync::{default_staging_uid, reindex_meilisearch};

use crate::boot::wait_meilisearch_text_search;
use crate::config::Config;
use crate::serve::resolve_config_path;

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

    let exit_code = rt.block_on(run_reindex(config));
    std::process::exit(exit_code);
}

async fn run_reindex(config: Config) -> i32 {
    let db_path = Path::new(&config.paths.datos).join(DB_DIR_NAME);
    let db = match Db::open(&db_path).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("storage: {e}");
            return 1;
        }
    };
    if let Err(e) = db.migrate().await {
        eprintln!("storage: {e}");
        return 1;
    }

    let text_search = match MeiliTextSearch::new(
        config.meilisearch.url.clone(),
        config.meilisearch.api_key.clone(),
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("meilisearch: {e}");
            return 1;
        }
    };

    if !wait_meilisearch_text_search(&text_search).await {
        eprintln!("meilisearch: unavailable at {}", config.meilisearch.url);
        return 1;
    }

    let db = Arc::new(db);
    let live = match IndexUid::try_new("productos") {
        Ok(uid) => uid,
        Err(e) => {
            eprintln!("meilisearch: {e}");
            return 1;
        }
    };
    let staging = match default_staging_uid() {
        Ok(uid) => uid,
        Err(e) => {
            eprintln!("meilisearch: {e}");
            return 1;
        }
    };

    let outbox = OutboxStore::new(Arc::clone(&db));
    let products = ProductoRepo::new(Arc::clone(&db));
    let attr_defs = AtributoDefRepo::new(Arc::clone(&db));

    tracing::info!(phase = "reindex.start", live = %live.as_str(), staging = %staging.as_str());

    if let Err(e) = reindex_meilisearch(
        &outbox,
        &products,
        &attr_defs,
        &text_search,
        &live,
        &staging,
    )
    .await
    {
        eprintln!("reindex: {e}");
        tracing::error!(phase = "reindex", error = %e, "reindex failed");
        return 1;
    }

    tracing::info!(phase = "reindex.complete");
    0
}
