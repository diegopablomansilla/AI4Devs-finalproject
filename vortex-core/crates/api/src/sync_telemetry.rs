//! Prometheus `SyncTelemetry` implementation (T-03-09).

use domain::SyncTelemetry;
use metrics::{
    Unit, counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram,
};

use crate::metrics::{OUTBOX_FAILED_TOTAL, OUTBOX_LAG};

/// Histogram name for outbox commit → Meilisearch confirmation latency.
pub const SYNC_PROPAGATION_SECONDS: &str = "sync_propagation_seconds";

/// Registers domain sync series descriptions (idempotent with global recorder).
pub fn describe_sync_metrics() {
    describe_gauge!(
        OUTBOX_LAG,
        Unit::Count,
        "Pending outbox events for a sync consumer"
    );
    describe_counter!(
        OUTBOX_FAILED_TOTAL,
        Unit::Count,
        "Outbox events marked fallido for Meilisearch sync"
    );
    describe_histogram!(
        SYNC_PROPAGATION_SECONDS,
        Unit::Seconds,
        "Seconds between outbox creado_en and successful Meilisearch sync"
    );
}

/// Records sync worker stats into the process-wide Prometheus recorder.
#[derive(Debug, Default, Clone, Copy)]
pub struct PrometheusSyncTelemetry;

impl SyncTelemetry for PrometheusSyncTelemetry {
    fn set_outbox_lag(&self, consumidor: &str, lag: u64) {
        gauge!(OUTBOX_LAG, "consumidor" => consumidor.to_string()).set(lag as f64);
    }

    fn inc_outbox_failed(&self) {
        counter!(OUTBOX_FAILED_TOTAL).increment(1);
    }

    fn observe_sync_propagation_seconds(&self, seconds: f64) {
        histogram!(SYNC_PROPAGATION_SECONDS).record(seconds);
    }
}
