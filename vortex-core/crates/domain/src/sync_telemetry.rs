//! Sync worker telemetry port (T-03-09); Prometheus impl lives in `api`.

/// Observes outbox lag, failed events and propagation latency (`SyncTelemetry` in spec).
pub trait SyncTelemetry: Send + Sync {
    fn set_outbox_lag(&self, consumidor: &str, lag: u64);
    fn inc_outbox_failed(&self);
    fn observe_sync_propagation_seconds(&self, seconds: f64);
}

/// No-op for unit tests and composition roots without metrics yet.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSyncTelemetry;

impl SyncTelemetry for NoopSyncTelemetry {
    fn set_outbox_lag(&self, _consumidor: &str, _lag: u64) {}

    fn inc_outbox_failed(&self) {}

    fn observe_sync_propagation_seconds(&self, _seconds: f64) {}
}
