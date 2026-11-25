use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use catalog::{CategoryService, ProductionOccupancy};
use storage::{CategoriaRepo, DB_DIR_NAME, Db};

static SEED_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn vortex_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vortex"))
}

fn temp_root() -> PathBuf {
    let n = SEED_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("vortex-seed-test-{}-{}", std::process::id(), n));
    fs::create_dir_all(&dir).expect("temp root");
    dir
}

fn write_config(datos: &Path) -> PathBuf {
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
url = "http://127.0.0.1:7700"

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
    );
    fs::write(&path, body.trim()).expect("config");
    path
}

fn write_master_seed(dir: &Path) {
    fs::create_dir_all(dir).expect("seed dir");
    fs::write(
        dir.join("categorias.jsonl"),
        r#"{"slug":"fijaciones","nombre":"Fijaciones"}
{"slug":"pernos","nombre":"Pernos","padre":"fijaciones"}
"#,
    )
    .expect("categorias");
    fs::write(
        dir.join("marcas.jsonl"),
        r#"{"slug":"fixser","nombre":"Fixser"}
"#,
    )
    .expect("marcas");
    fs::write(
        dir.join("atributos.jsonl"),
        r#"{"clave":"medida","etiqueta":"Medida","tipo":"texto"}
"#,
    )
    .expect("atributos");
}

#[test]
fn vortex_seed_imports_masters_exit_0() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let config = write_config(&datos);
    let seed_dir = root.join("seed");
    write_master_seed(&seed_dir);

    let output = Command::new(vortex_bin())
        .args([
            "--config",
            config.to_str().expect("config path"),
            "seed",
            seed_dir.to_str().expect("seed path"),
        ])
        .output()
        .expect("spawn");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"));
}

#[tokio::test]
async fn vortex_seed_twice_does_not_duplicate() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let config = write_config(&datos);
    let seed_dir = root.join("seed");
    write_master_seed(&seed_dir);
    let config_s = config.to_str().expect("config");
    let seed_s = seed_dir.to_str().expect("seed");

    for _ in 0..2 {
        let output = Command::new(vortex_bin())
            .args(["--config", config_s, "seed", seed_s])
            .output()
            .expect("spawn");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let db = Arc::new(Db::open(datos.join(DB_DIR_NAME)).await.expect("open db"));
    let categories: Arc<dyn catalog::CategoryGateway> = Arc::new(CategoryService::new(
        CategoriaRepo::new(Arc::clone(&db)),
        ProductionOccupancy,
    ));
    let cats = categories.list_all().await.expect("list cats");
    assert_eq!(cats.len(), 2);
}

#[tokio::test]
async fn vortex_seed_locked_db_is_error() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let config = write_config(&datos);
    let seed_dir = root.join("seed");
    write_master_seed(&seed_dir);

    let _db = Db::open(datos.join(DB_DIR_NAME)).await.expect("hold lock");

    let output = Command::new(vortex_bin())
        .args([
            "--config",
            config.to_str().expect("config"),
            "seed",
            seed_dir.to_str().expect("seed"),
        ])
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
