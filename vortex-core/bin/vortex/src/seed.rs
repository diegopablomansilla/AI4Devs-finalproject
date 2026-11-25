//! `vortex seed` — import master catalog rows (T-02-42).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use catalog::{
    AttributeDefService, BrandService, CategoryService, ProductionOccupancy, import_masters,
};
use storage::{AtributoDefRepo, CategoriaRepo, DB_DIR_NAME, Db, MarcaRepo, ProductAttributeUsage};

use crate::config::Config;
use crate::serve::resolve_config_path;

pub fn run(config_flag: Option<&PathBuf>, dir: PathBuf) -> ! {
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

    let exit_code = rt.block_on(run_import(config, &dir));
    std::process::exit(exit_code);
}

async fn run_import(config: Config, dir: &Path) -> i32 {
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

    let db = Arc::new(db);
    let categories: Arc<dyn catalog::CategoryGateway> = Arc::new(CategoryService::new(
        CategoriaRepo::new(Arc::clone(&db)),
        ProductionOccupancy,
    ));
    let brands: Arc<dyn catalog::BrandGateway> =
        Arc::new(BrandService::new(MarcaRepo::new(Arc::clone(&db))));
    let attribute_defs: Arc<dyn catalog::AttributeDefGateway> = Arc::new(AttributeDefService::new(
        AtributoDefRepo::new(Arc::clone(&db)),
        ProductAttributeUsage::new(Arc::clone(&db)),
    ));

    match import_masters(
        dir,
        categories.as_ref(),
        brands.as_ref(),
        attribute_defs.as_ref(),
    )
    .await
    {
        Ok(()) => 0,
        Err(issues) => {
            for issue in &issues {
                if issue.linea == 0 {
                    eprintln!(
                        "{}: {}",
                        issue.archivo,
                        if issue.mensaje.is_empty() {
                            issue.campo.as_str()
                        } else {
                            issue.mensaje.as_str()
                        }
                    );
                } else if issue.campo.is_empty() {
                    eprintln!("{}:{} {}", issue.archivo, issue.linea, issue.mensaje);
                } else {
                    eprintln!(
                        "{}:{} {}: {}",
                        issue.archivo, issue.linea, issue.campo, issue.mensaje
                    );
                }
            }
            1
        }
    }
}
