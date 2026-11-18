//! API Gateway HTTP shell (T-00-10 … T-00-14, T-00-19).

mod app_json;
mod attribute_definitions;
mod brands;
mod categories;
mod error;
mod gateway;
mod health;
mod metrics;
mod products;
mod static_files;
mod stock;

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Router,
    http::{StatusCode, Uri},
    middleware,
    response::IntoResponse,
    routing::{get, patch, post},
};
use catalog::{AttributeDefGateway, BrandGateway, CategoryGateway, ProductGateway, StockGateway};
use domain::HealthReport;

pub use app_json::AppJson;
pub use error::ApiError;
pub use gateway::{DEFAULT_BODY_LIMIT, DEFAULT_REQUEST_TIMEOUT, apply_gateway_layers};
pub use metrics::{
    BACKUP_FAILURES_TOTAL, OUTBOX_FAILED_TOTAL, OUTBOX_LAG, operation_duration_seconds,
};

/// Shared application state for HTTP handlers (`Arc<dyn …>` added in later tickets).
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Clone)]
struct AppStateInner {
    health: Arc<dyn HealthReport>,
    web_root: PathBuf,
    media_root: PathBuf,
    version: String,
    categories: Option<Arc<dyn CategoryGateway>>,
    brands: Option<Arc<dyn BrandGateway>>,
    attribute_defs: Option<Arc<dyn AttributeDefGateway>>,
    products: Option<Arc<dyn ProductGateway>>,
    stock: Option<Arc<dyn StockGateway>>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self::with_health(Arc::new(domain::HealthSnapshot::all_disabled(env!(
            "CARGO_PKG_VERSION"
        ))))
    }

    #[must_use]
    pub fn with_health(health: Arc<dyn HealthReport>) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                health,
                web_root: PathBuf::new(),
                media_root: PathBuf::new(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                categories: None,
                brands: None,
                attribute_defs: None,
                products: None,
                stock: None,
            }),
        }
    }

    #[must_use]
    pub fn with_static_roots(mut self, web_root: PathBuf, media_root: PathBuf) -> Self {
        Arc::make_mut(&mut self.inner).web_root = web_root;
        Arc::make_mut(&mut self.inner).media_root = media_root;
        self
    }

    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.inner).version = version.into();
        self
    }

    #[must_use]
    pub fn health(&self) -> Arc<dyn HealthReport> {
        Arc::clone(&self.inner.health)
    }

    #[must_use]
    pub fn web_root(&self) -> PathBuf {
        self.inner.web_root.clone()
    }

    #[must_use]
    pub fn media_root(&self) -> PathBuf {
        self.inner.media_root.clone()
    }

    #[must_use]
    pub fn version(&self) -> String {
        self.inner.version.clone()
    }

    #[must_use]
    pub fn with_categories(mut self, categories: Arc<dyn CategoryGateway>) -> Self {
        Arc::make_mut(&mut self.inner).categories = Some(categories);
        self
    }

    pub(crate) fn categories(&self) -> Option<Arc<dyn CategoryGateway>> {
        self.inner.categories.clone()
    }

    #[must_use]
    pub fn with_brands(mut self, brands: Arc<dyn BrandGateway>) -> Self {
        Arc::make_mut(&mut self.inner).brands = Some(brands);
        self
    }

    pub(crate) fn brands(&self) -> Option<Arc<dyn BrandGateway>> {
        self.inner.brands.clone()
    }

    #[must_use]
    pub fn with_attribute_defs(mut self, attribute_defs: Arc<dyn AttributeDefGateway>) -> Self {
        Arc::make_mut(&mut self.inner).attribute_defs = Some(attribute_defs);
        self
    }

    pub(crate) fn attribute_defs(&self) -> Option<Arc<dyn AttributeDefGateway>> {
        self.inner.attribute_defs.clone()
    }

    #[must_use]
    pub fn with_products(mut self, products: Arc<dyn ProductGateway>) -> Self {
        Arc::make_mut(&mut self.inner).products = Some(products);
        self
    }

    pub(crate) fn products(&self) -> Option<Arc<dyn ProductGateway>> {
        self.inner.products.clone()
    }

    #[must_use]
    pub fn with_stock(mut self, stock: Arc<dyn StockGateway>) -> Self {
        Arc::make_mut(&mut self.inner).stock = Some(stock);
        self
    }

    pub(crate) fn stock(&self) -> Option<Arc<dyn StockGateway>> {
        self.inner.stock.clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

async fn fallback_not_found(uri: Uri) -> impl IntoResponse {
    ApiError::route_not_found(uri.path().to_owned())
}

async fn fallback_method_not_allowed() -> impl IntoResponse {
    StatusCode::METHOD_NOT_ALLOWED
}

/// Router shell (fallback 404) before gateway layers; used to attach test-only routes.
pub fn router_shell(state: AppState) -> Router {
    // Eager init so the first scrape and concurrent tests share one recorder.
    let _ = metrics::prometheus_handle();
    Router::new()
        .route(
            "/",
            get(static_files::get_index).post(fallback_method_not_allowed),
        )
        .route(
            "/index.html",
            get(static_files::get_index).post(fallback_method_not_allowed),
        )
        .route("/sw.js", get(static_files::get_sw_js))
        .route("/manifest.webmanifest", get(static_files::get_manifest))
        .route(
            "/static/{version}/{*path}",
            get(static_files::get_versioned_static),
        )
        .route("/media/{*path}", get(static_files::get_media))
        .route("/health", get(health::get_health))
        .route(
            "/metrics",
            get(metrics::get_metrics).post(|| async { StatusCode::METHOD_NOT_ALLOWED }),
        )
        .route(
            "/api/v1/categories",
            get(categories::list_categories).post(categories::post_category),
        )
        .route(
            "/api/v1/categories/{category_id}",
            patch(categories::patch_category).delete(categories::delete_category),
        )
        .route(
            "/api/v1/brands",
            get(brands::list_brands).post(brands::post_brand),
        )
        .route(
            "/api/v1/brands/{brand_id}",
            patch(brands::patch_brand).delete(brands::delete_brand),
        )
        .route(
            "/api/v1/attribute-definitions",
            get(attribute_definitions::list_attribute_definitions)
                .post(attribute_definitions::post_attribute_definition),
        )
        .route(
            "/api/v1/attribute-definitions/{attribute_id}",
            patch(attribute_definitions::patch_attribute_definition)
                .delete(attribute_definitions::delete_attribute_definition),
        )
        .route("/api/v1/products", post(products::post_product))
        .route(
            "/api/v1/products/{product_id}",
            get(products::get_product)
                .patch(products::patch_product)
                .delete(products::delete_product),
        )
        .route(
            "/api/v1/products/{product_id}/stock-movements",
            post(stock::post_stock_movement),
        )
        .fallback(fallback_not_found)
        .layer(middleware::from_fn(metrics::record_http_duration))
        .with_state(state)
}

/// Builds the HTTP router without business routes (unknown paths → 404 `problem+json`).
pub fn build_router(state: AppState) -> Router {
    apply_gateway_layers(router_shell(state))
}
