//! Catalog & Inventory — use cases (EP-02).

pub mod atributo_def;
pub mod categoria;
pub mod marca;
pub mod producto;
pub mod seed;
pub mod stock;

pub use atributo_def::{AttributeDefGateway, AttributeDefService, EmptyAttributeUsage};
pub use categoria::{CategoryGateway, CategoryOccupancy, CategoryService, ProductionOccupancy};
pub use marca::{BrandGateway, BrandService};
pub use producto::{
    PatchProducto, ProductGateway, ProductService, product_id_after,
    product_matches_categoria_branch,
};
pub use seed::{SeedIssue, import_masters, validate_dir};
pub use stock::{StockGateway, StockService};
