//! Sidecar supervisor — health registry (T-00-13).

use std::sync::RwLock;
use std::time::Instant;

use domain::{
    ComponentHealth, ComponentId, ComponentStatus, HealthComponents, HealthReport, HealthSnapshot,
    aggregate,
};

pub struct HealthRegistry {
    version: String,
    started_at: Instant,
    components: RwLock<HealthComponents>,
}

impl HealthRegistry {
    #[must_use]
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            started_at: Instant::now(),
            components: RwLock::new(HealthComponents::all_disabled()),
        }
    }

    pub fn set(&self, id: ComponentId, status: ComponentStatus) {
        let mut guard = self.components.write().expect("health registry lock");
        let ch = ComponentHealth::new(status);
        match id {
            ComponentId::Storage => guard.storage = ch,
            ComponentId::Meilisearch => guard.meilisearch = ch,
            ComponentId::Llm => guard.llm = ch,
            ComponentId::Speech => guard.speech = ch,
            ComponentId::TextEmbeddings => guard.text_embeddings = ch,
            ComponentId::Sync => guard.sync = ch,
        }
    }
}

impl HealthReport for HealthRegistry {
    fn snapshot(&self) -> HealthSnapshot {
        let components = self
            .components
            .read()
            .expect("health registry lock")
            .clone();
        let status = aggregate(&components);
        let uptime_seconds = self.started_at.elapsed().as_secs();
        HealthSnapshot {
            status,
            version: self.version.clone(),
            uptime_seconds,
            components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::AggregateStatus;

    #[test]
    fn registry_new_is_ok_with_disabled_components() {
        let reg = HealthRegistry::new("0.1.0");
        let snap = reg.snapshot();
        assert_eq!(snap.version, "0.1.0");
        assert_eq!(snap.status, AggregateStatus::Ok);
        let _ = snap.uptime_seconds;
        assert_eq!(snap.components.storage.status, ComponentStatus::Disabled);
        assert_eq!(
            snap.components.meilisearch.status,
            ComponentStatus::Disabled
        );
        assert_eq!(snap.components.llm.status, ComponentStatus::Disabled);
        assert_eq!(snap.components.speech.status, ComponentStatus::Disabled);
        assert_eq!(
            snap.components.text_embeddings.status,
            ComponentStatus::Disabled
        );
        assert_eq!(snap.components.sync.status, ComponentStatus::Disabled);
    }

    #[test]
    fn registry_set_meilisearch_down_is_degraded() {
        let reg = HealthRegistry::new("0.1.0");
        reg.set(ComponentId::Meilisearch, ComponentStatus::Down);
        let snap = reg.snapshot();
        assert_eq!(snap.components.meilisearch.status, ComponentStatus::Down);
        assert_eq!(snap.status, AggregateStatus::Degraded);
    }

    #[test]
    fn registry_set_storage_starting_is_starting() {
        let reg = HealthRegistry::new("0.1.0");
        reg.set(ComponentId::Storage, ComponentStatus::Starting);
        let snap = reg.snapshot();
        assert_eq!(snap.components.storage.status, ComponentStatus::Starting);
        assert_eq!(snap.status, AggregateStatus::Starting);
    }
}
