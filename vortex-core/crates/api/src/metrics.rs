//! Prometheus exposition and HTTP latency histogram (T-00-14).

use std::sync::OnceLock;
use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::{Unit, describe_histogram, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

/// Upper bounds for `http_request_duration_seconds` (classic Prometheus histogram).
const HTTP_DURATION_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

/// Reserved name for outbox lag gauge (T-03-09).
pub const OUTBOX_LAG: &str = "outbox_lag";

/// Reserved name for failed outbox events counter (T-03-09).
pub const OUTBOX_FAILED_TOTAL: &str = "outbox_failed_total";

/// Reserved name for backup failure counter (T-00-29).
pub const BACKUP_FAILURES_TOTAL: &str = "backup_failures_total";

/// Builds the Prometheus metric name `{operation}_duration_seconds`.
#[must_use]
pub fn operation_duration_seconds(operation: &str) -> String {
    format!("{operation}_duration_seconds")
}

static PROMETHEUS: OnceLock<PrometheusHandle> = OnceLock::new();

/// Installs the global Prometheus recorder once and returns the scrape handle.
pub fn prometheus_handle() -> &'static PrometheusHandle {
    PROMETHEUS.get_or_init(|| {
        describe_histogram!(
            "http_request_duration_seconds",
            Unit::Seconds,
            "HTTP request latency"
        );
        PrometheusBuilder::new()
            .set_buckets(HTTP_DURATION_BUCKETS)
            .expect("valid HTTP duration histogram buckets")
            .install_recorder()
            .expect("install prometheus metrics recorder")
    })
}

pub async fn get_metrics() -> impl IntoResponse {
    let body = prometheus_handle().render();
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}

pub async fn record_http_duration(
    matched_path: Option<MatchedPath>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path();
    if method == axum::http::Method::GET && path == "/metrics" {
        return next.run(request).await;
    }

    let start = Instant::now();
    let response = next.run(request).await;

    let route = if response.status() == StatusCode::NOT_FOUND {
        "unmatched".to_owned()
    } else {
        matched_path
            .as_ref()
            .map(axum::extract::MatchedPath::as_str)
            .unwrap_or("unmatched")
            .to_owned()
    };
    let method_label = method.as_str().to_owned();
    let status = response.status().as_u16().to_string();

    histogram!(
        "http_request_duration_seconds",
        "route" => route,
        "method" => method_label,
        "status" => status,
    )
    .record(start.elapsed().as_secs_f64());

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_metric_names_match_convention() {
        assert_eq!(OUTBOX_LAG, "outbox_lag");
        assert_eq!(OUTBOX_FAILED_TOTAL, "outbox_failed_total");
        assert_eq!(BACKUP_FAILURES_TOTAL, "backup_failures_total");
        assert_eq!(
            operation_duration_seconds("search"),
            "search_duration_seconds"
        );
    }
}
