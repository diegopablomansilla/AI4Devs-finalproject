//! Meilisearch boot probe (T-00-27).

use std::future::Future;
use std::time::{Duration, Instant};

use domain::health::{ComponentHealth, ComponentStatus};
use domain::{DomainError, TextSearch};

const DEFAULT_MEILI_BOOT_TIMEOUT: Duration = Duration::from_secs(20);
const MEILI_PROBE_INTERVAL: Duration = Duration::from_millis(200);

/// Boot wait for Meilisearch; overridable via `VORTEX_BOOT_MEILI_TIMEOUT_MS` (harness only).
#[must_use]
pub fn meili_boot_timeout() -> Duration {
    std::env::var("VORTEX_BOOT_MEILI_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_MEILI_BOOT_TIMEOUT)
}

/// Polls `probe` until it returns `up`, `timeout` elapses, or `probe` errors persist.
pub async fn wait_meilisearch_up<F, Fut>(
    mut probe: F,
    timeout: Duration,
    interval: Duration,
) -> bool
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ComponentHealth, DomainError>>,
{
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(health) = probe().await
            && health.status == ComponentStatus::Up
        {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(interval).await;
    }
}

/// Uses [`meili_boot_timeout`] and [`MEILI_PROBE_INTERVAL`].
pub async fn wait_meilisearch_text_search(search: &impl TextSearch) -> bool {
    let timeout = meili_boot_timeout();
    wait_meilisearch_up(
        || async { search.health().await },
        timeout,
        MEILI_PROBE_INTERVAL,
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use domain::DomainError;
    use domain::error::UnavailableCode;

    use super::*;

    struct ProbeState {
        calls: AtomicUsize,
        up_after: usize,
    }

    impl ProbeState {
        fn call(&self) -> Result<ComponentHealth, DomainError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n >= self.up_after {
                Ok(ComponentHealth::new(ComponentStatus::Up))
            } else {
                Err(DomainError::unavailable(UnavailableCode::Unavailable))
            }
        }
    }

    #[tokio::test]
    async fn wait_meilisearch_returns_true_when_probe_is_up() {
        let probe = Arc::new(ProbeState {
            calls: AtomicUsize::new(0),
            up_after: 1,
        });
        let p = Arc::clone(&probe);
        let ok = wait_meilisearch_up(
            || {
                let p = Arc::clone(&p);
                async move { p.call() }
            },
            Duration::from_secs(1),
            Duration::from_millis(10),
        )
        .await;
        assert!(ok);
        assert!(probe.calls.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn wait_meilisearch_returns_false_when_probe_stays_down() {
        let probe = Arc::new(ProbeState {
            calls: AtomicUsize::new(0),
            up_after: usize::MAX,
        });
        let p = Arc::clone(&probe);
        let ok = wait_meilisearch_up(
            || {
                let p = Arc::clone(&p);
                async move { p.call() }
            },
            Duration::from_millis(50),
            Duration::from_millis(10),
        )
        .await;
        assert!(!ok);
        assert!(probe.calls.load(Ordering::SeqCst) >= 1);
    }
}
