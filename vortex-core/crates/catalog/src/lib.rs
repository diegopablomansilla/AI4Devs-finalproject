//! Catalog & Inventory — use cases (EP-02).

pub mod atributo_def;
pub mod categoria;
pub mod marca;
pub mod producto;

pub use atributo_def::{AttributeDefGateway, AttributeDefService};
pub use categoria::{CategoryGateway, CategoryOccupancy, CategoryService, ProductionOccupancy};
pub use marca::{BrandGateway, BrandService};
pub use producto::{PatchProducto, ProductGateway, ProductService};
