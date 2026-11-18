//! Catalog & Inventory — use cases (EP-02).

pub mod categoria;
pub mod marca;

pub use categoria::{CategoryGateway, CategoryOccupancy, CategoryService, ProductionOccupancy};
pub use marca::{BrandGateway, BrandService};
