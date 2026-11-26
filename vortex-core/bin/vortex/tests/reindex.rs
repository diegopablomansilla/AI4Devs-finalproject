use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use storage::{DB_DIR_NAME, Db};

static REINDEX_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn vortex_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vortex"))
}

fn temp_root() -> PathBuf {
    let n = REINDEX_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir =
        std::env::temp_dir().join(format!("vortex-reindex-test-{}-{}", std::process::id(), n));
    fs::create_dir_all(&dir).expect("temp root");
    dir
}

fn write_config(datos: &Path, meili_url: &str) -> PathBuf {
    let root = datos.parent().expect("datos parent").to_path_buf();
    let web = root.join("web");
    fs::create_dir_all(&web).expect("web");
    let path = root.join("vortex.toml");
    let body = format!(
        r#"
[paths]
datos = "{datos}"
modelos = "/tmp/modelos"
backups = "/tmp/backups"
web = "{web}"

[server]
bind = "127.0.0.1:0"
hostname = "localhost"

[negocio]
moneda = "ARS"

[meilisearch]
url = "{meili_url}"

[ollama]
url = "http://127.0.0.1:11434"

[backup]
hora = "03:00"
retencion = 14

[logs]
nivel = "info"
retencion = 14
"#,
        datos = datos.display(),
        web = web.display(),
        meili_url = meili_url,
    );
    fs::write(&path, body.trim()).expect("config");
    path
}

async fn seed_one_product(db: &Db) {
    db.query(
        "CREATE categoria:cat CONTENT { nombre: 'C', slug: 'c', ruta: ['C'] }; \
         CREATE producto:prod CONTENT { \
         sku: 'PER-M8X40-ZN', nombre: 'Perno', tipo: 'Perno', categoria: categoria:cat, \
         precio: 1dec, activo: true, stock: 1 };",
    )
    .await
    .expect("seed");
}

#[tokio::test]
async fn vortex_reindex_meilisearch_down_is_error() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let config = write_config(&datos, "http://127.0.0.1:1");

    let output = Command::new(vortex_bin())
        .env("VORTEX_BOOT_MEILI_TIMEOUT_MS", "0")
        .args(["--config", config.to_str().expect("config"), "reindex"])
        .output()
        .expect("spawn");

    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
    assert!(
        stderr.to_lowercase().contains("meilisearch"),
        "stderr: {stderr}"
    );
}

#[tokio::test]
async fn vortex_reindex_locked_db_is_error() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let config = write_config(&datos, "http://127.0.0.1:7700");

    let _db = Db::open(datos.join(DB_DIR_NAME)).await.expect("hold lock");

    let output = Command::new(vortex_bin())
        .args(["--config", config.to_str().expect("config"), "reindex"])
        .output()
        .expect("spawn");

    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
    assert!(
        stderr.contains("storage") || stderr.to_lowercase().contains("abrir"),
        "stderr: {stderr}"
    );
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn vortex_reindex_exit_0() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let meili = std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".into());
    let config = write_config(&datos, &meili);

    let db = Db::open(datos.join(DB_DIR_NAME)).await.expect("open");
    db.migrate().await.expect("migrate");
    seed_one_product(&db).await;
    drop(db);

    let output = Command::new(vortex_bin())
        .args(["--config", config.to_str().expect("config"), "reindex"])
        .output()
        .expect("spawn");

    assert_eq!(output.status.code(), Some(0), "stderr: {:?}", output.stderr);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
}
