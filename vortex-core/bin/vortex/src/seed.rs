//! `vortex seed` — import master catalog rows, products and relations (T-02-42, T-02-43, T-04-11).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use catalog::{
    AttributeDefService, BrandService, CategoryService, ProductService, ProductionOccupancy,
    RelationSeedGateway, import_masters, import_products, import_relations,
};
use domain::NewRelacion;
use recommend::RelationService;
use storage::{
    AtributoDefRepo, CategoriaRepo, DB_DIR_NAME, Db, MarcaRepo, ProductAttributeUsage,
    ProductoRepo, RelacionRepo,
};

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

fn print_issues(issues: &[catalog::SeedIssue]) {
    for issue in issues {
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
        Ok(()) => {}
        Err(issues) => {
            print_issues(&issues);
            return 1;
        }
    }

    let products: Arc<dyn catalog::ProductGateway> = Arc::new(ProductService::new(
        ProductoRepo::new(Arc::clone(&db)),
        CategoriaRepo::new(Arc::clone(&db)),
        MarcaRepo::new(Arc::clone(&db)),
        AtributoDefRepo::new(Arc::clone(&db)),
    ));

    let summary = import_products(
        dir,
        categories.as_ref(),
        brands.as_ref(),
        attribute_defs.as_ref(),
        products.as_ref(),
    )
    .await;

    if !summary.issues.is_empty() {
        print_issues(&summary.issues);
    }
    eprintln!(
        "productos: importados={} rechazados={}",
        summary.importados, summary.rechazados
    );

    let relation_seed = RelationSeedCli {
        inner: RelationService::new(
            RelacionRepo::new(Arc::clone(&db)),
            ProductoRepo::new(Arc::clone(&db)),
        ),
    };
    let relations: Arc<dyn RelationSeedGateway> = Arc::new(relation_seed);

    let rel_summary = import_relations(dir, products.as_ref(), relations.as_ref()).await;

    if !rel_summary.issues.is_empty() {
        print_issues(&rel_summary.issues);
    }
    eprintln!(
        "relaciones: importados={} omitidas={} rechazados={}",
        rel_summary.importados, rel_summary.omitidas, rel_summary.rechazados
    );

    if summary.rechazados > 0 || rel_summary.rechazados > 0 {
        return 1;
    }
    0
}

struct RelationSeedCli {
    inner: RelationService<RelacionRepo, ProductoRepo>,
}

impl RelationSeedGateway for RelationSeedCli {
    fn create(
        &self,
        input: NewRelacion,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), domain::DomainError>> + Send + '_>,
    > {
        let svc = &self.inner;
        Box::pin(async move { RelationService::create(svc, input).await.map(|_| ()) })
    }
}
