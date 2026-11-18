//! Integration tests for `ProductAttributeUsage` (T-02-23).

use std::sync::Arc;

use domain::{
    AtributoDef, AtributoValor, AttributeDefRepo, AttributeUsage, BrandRepo, Categoria,
    CategoryRepo, Codigo, Id, Marca, NewAtributoDef, NewCategoria, NewMarca, NewProducto,
    ProductRepo, TipoAtributo, TipoCodigo, UnidadMedida, UpdateAtributoDef, ValorAtributo,
};

use storage::testing::test_db;
use storage::{AtributoDefRepo, CategoriaRepo, MarcaRepo, ProductAttributeUsage, ProductoRepo};

async fn seed_ficha_catalog(db: Arc<storage::Db>) -> (Id<Categoria>, Id<Marca>, Id<AtributoDef>) {
    let parent = CategoriaRepo::new(Arc::clone(&db))
        .create(NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("parent categoria");
    let child = CategoriaRepo::new(Arc::clone(&db))
        .create(NewCategoria {
            nombre: "Pernos".into(),
            slug: None,
            padre: Some(parent.id),
        })
        .await
        .expect("child categoria");
    let marca = MarcaRepo::new(Arc::clone(&db))
        .create(NewMarca {
            nombre: "Fixser".into(),
            slug: None,
        })
        .await
        .expect("marca");
    let medida = AtributoDefRepo::new(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect("medida def");
    (child.id, marca.id, medida.id)
}

fn new_perno_rich(
    categoria: Id<Categoria>,
    marca: Id<Marca>,
    medida_def: Id<AtributoDef>,
    sku: &str,
) -> NewProducto {
    NewProducto {
        sku: sku.into(),
        nombre: "Perno hexagonal M8 x 40 zincado".into(),
        tipo: "Perno".into(),
        descripcion: None,
        marca: Some(marca),
        categoria,
        atributos: vec![AtributoValor {
            def: medida_def,
            valor: ValorAtributo::Texto("M8".into()),
        }],
        codigos: vec![Codigo::try_new(TipoCodigo::Fabricante, "HX-0840Z").expect("codigo")],
        unidad_medida: UnidadMedida::Unidad,
        precio: "120.50".parse().expect("precio"),
        ubicacion: None,
        stock_inicial: "0".parse().expect("stock"),
        terminal: None,
    }
}

#[tokio::test]
async fn attribute_usage_returns_medida_m8() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    ProductoRepo::new(Arc::clone(&db))
        .create(new_perno_rich(
            categoria,
            marca,
            medida.clone(),
            "PER-M8X40-ZN",
        ))
        .await
        .expect("create");

    let usage = ProductAttributeUsage::new(Arc::clone(&db));
    let vals = usage.valores(medida.clone()).await.expect("valores");
    assert_eq!(vals.len(), 1);
    assert!(matches!(vals[0], ValorAtributo::Texto(ref s) if s == "M8"));
}

#[tokio::test]
async fn attribute_usage_empty_when_unused() {
    let db = Arc::new(test_db().await);
    let (_, _, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let usage = ProductAttributeUsage::new(db);
    let vals = usage.valores(medida.clone()).await.expect("valores");
    assert!(vals.is_empty());
}

#[tokio::test]
async fn attribute_usage_includes_inactive_product() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let created = ProductoRepo::new(Arc::clone(&db))
        .create(new_perno_rich(
            categoria,
            marca,
            medida.clone(),
            "PER-INACTIVE",
        ))
        .await
        .expect("create");
    ProductoRepo::new(Arc::clone(&db))
        .set_activo(created.id, false)
        .await
        .expect("deactivate");

    let usage = ProductAttributeUsage::new(Arc::clone(&db));
    let vals = usage.valores(medida.clone()).await.expect("valores");
    assert_eq!(vals.len(), 1);
}

#[tokio::test]
async fn attribute_def_repo_update_does_not_apply_in_use() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    ProductoRepo::new(Arc::clone(&db))
        .create(new_perno_rich(categoria, marca, medida.clone(), "PER-KEY"))
        .await
        .expect("create");

    let medida_def = AtributoDefRepo::new(Arc::clone(&db))
        .get(medida)
        .await
        .expect("get");
    let updated = AtributoDefRepo::new(Arc::clone(&db))
        .update(UpdateAtributoDef {
            id: medida_def.id.clone(),
            clave: "medida_mm".into(),
            etiqueta: medida_def.etiqueta.clone(),
            tipo: medida_def.tipo,
            unidad: medida_def.unidad.clone(),
            opciones: medida_def.opciones.clone(),
            facetable: medida_def.facetable,
            buscable: medida_def.buscable,
            orden: medida_def.orden,
        })
        .await
        .expect("repo update without occupancy rule");
    assert_eq!(updated.clave, "medida_mm");
}
