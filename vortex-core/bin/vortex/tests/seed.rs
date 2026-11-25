use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use catalog::{
    CategoryService, FiltroProducto, ProductService, ProductionOccupancy, StockLedger,
    TipoMovimiento,
};
use storage::{CategoriaRepo, DB_DIR_NAME, Db, MarcaRepo, MovimientoStockRepo, ProductoRepo};

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
    fs::write(
        dir.join("productos.jsonl"),
        r#"{"sku":"PER-M8X40-ZN","nombre":"Perno hexagonal M8 x 40","tipo":"Perno","categoria":"pernos","marca":"fixser","precio":"120.50","stock_inicial":"50","atributos":[{"clave":"medida","valor":"M8"}]}
"#,
    )
    .expect("productos");
}

fn repo_seed_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../seed")
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

async fn count_products(db: &Arc<Db>) -> usize {
    let gw: Arc<dyn catalog::ProductGateway> = Arc::new(ProductService::new(
        ProductoRepo::new(Arc::clone(db)),
        CategoriaRepo::new(Arc::clone(db)),
        MarcaRepo::new(Arc::clone(db)),
        storage::AtributoDefRepo::new(Arc::clone(db)),
    ));
    let mut total = 0usize;
    for activo in [true, false] {
        let mut after = None;
        loop {
            let batch = gw
                .list(
                    FiltroProducto {
                        categoria: None,
                        marca: None,
                        activo,
                    },
                    after,
                    500,
                )
                .await
                .expect("list");
            if batch.is_empty() {
                break;
            }
            total += batch.len();
            if batch.len() < 500 {
                break;
            }
            after = batch.last().map(|p| p.id.clone());
        }
    }
    total
}

#[test]
fn vortex_seed_imports_masters_and_products_exit_0() {
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
    assert!(stderr.contains("importados=1"));
    assert!(stderr.contains("rechazados=0"));
}

#[tokio::test]
async fn vortex_seed_twice_does_not_duplicate_products() {
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
    assert_eq!(count_products(&db).await, 1);

    let ledger = MovimientoStockRepo::new(Arc::clone(&db));
    let gw: Arc<dyn catalog::ProductGateway> = Arc::new(ProductService::new(
        ProductoRepo::new(Arc::clone(&db)),
        CategoriaRepo::new(Arc::clone(&db)),
        MarcaRepo::new(Arc::clone(&db)),
        storage::AtributoDefRepo::new(Arc::clone(&db)),
    ));
    let batch = gw
        .list(
            FiltroProducto {
                categoria: None,
                marca: None,
                activo: true,
            },
            None,
            10,
        )
        .await
        .expect("list");
    let moves = ledger
        .list_by_producto(batch[0].id.clone())
        .await
        .expect("moves");
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].tipo, TipoMovimiento::Inicial);

    let categories: Arc<dyn catalog::CategoryGateway> = Arc::new(CategoryService::new(
        CategoriaRepo::new(Arc::clone(&db)),
        ProductionOccupancy,
    ));
    let cats = categories.list_all().await.expect("list cats");
    assert_eq!(cats.len(), 2);
}

#[tokio::test]
async fn vortex_seed_dev_dataset_exit_0() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let config = write_config(&datos);
    let seed_dir = repo_seed_dir();

    let output = Command::new(vortex_bin())
        .args([
            "--config",
            config.to_str().expect("config"),
            "seed",
            seed_dir.to_str().expect("seed"),
        ])
        .output()
        .expect("spawn");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let db = Arc::new(Db::open(datos.join(DB_DIR_NAME)).await.expect("open db"));
    let count = count_products(&db).await;
    assert!((180..=220).contains(&count));
}

#[test]
fn vortex_seed_invalid_precio_rejected_alone() {
    let root = temp_root();
    let datos = root.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let config = write_config(&datos);
    let seed_dir = root.join("seed");
    copy_dir_all(&repo_seed_dir(), &seed_dir).expect("copy seed");
    let path = seed_dir.join("productos.jsonl");
    let valid_product_lines = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(
        r#"{"sku":"BAD-PRICE","nombre":"X","tipo":"Perno","categoria":"pernos","marca":"fixser","precio":"barato","stock_inicial":"1","atributos":[{"clave":"medida","valor":"M8"}]}
"#,
    );
    fs::write(&path, content).expect("append bad row");

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
    assert!(stderr.contains("productos.jsonl"));
    assert!(stderr.contains("precio"));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt");
    let db = rt.block_on(async { Db::open(datos.join(DB_DIR_NAME)).await.expect("open") });
    let db = Arc::new(db);
    let count = rt.block_on(count_products(&db));
    assert!(
        (valid_product_lines.saturating_sub(1)..=valid_product_lines).contains(&count),
        "expected ~{valid_product_lines} valid products, got {count}"
    );
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
