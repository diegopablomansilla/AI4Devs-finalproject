//! Health ports and aggregation (T-00-13).

/// Fixed platform component identifiers for `/health`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentId {
    Storage,
    Meilisearch,
    Llm,
    Speech,
    TextEmbeddings,
    Sync,
}

/// Per-component status (`ComponentStatus` in OpenAPI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Up,
    Starting,
    Degraded,
    Down,
    Disabled,
}

/// Aggregate `/health` status (`Health.status` in OpenAPI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateStatus {
    Ok,
    Degraded,
    Starting,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentHealth {
    pub status: ComponentStatus,
}

impl ComponentHealth {
    #[must_use]
    pub const fn new(status: ComponentStatus) -> Self {
        Self { status }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthComponents {
    pub storage: ComponentHealth,
    pub meilisearch: ComponentHealth,
    pub llm: ComponentHealth,
    pub speech: ComponentHealth,
    pub text_embeddings: ComponentHealth,
    pub sync: ComponentHealth,
}

impl HealthComponents {
    #[must_use]
    pub fn with_uniform(status: ComponentStatus) -> Self {
        let ch = ComponentHealth::new(status);
        Self {
            storage: ch,
            meilisearch: ch,
            llm: ch,
            speech: ch,
            text_embeddings: ch,
            sync: ch,
        }
    }

    #[must_use]
    pub fn all_up() -> Self {
        Self::with_uniform(ComponentStatus::Up)
    }

    #[must_use]
    pub fn all_disabled() -> Self {
        Self::with_uniform(ComponentStatus::Disabled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub status: AggregateStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: HealthComponents,
}

impl HealthSnapshot {
    #[must_use]
    pub fn from_components(
        version: impl Into<String>,
        uptime_seconds: u64,
        components: HealthComponents,
    ) -> Self {
        let status = aggregate(&components);
        Self {
            status,
            version: version.into(),
            uptime_seconds,
            components,
        }
    }

    #[must_use]
    pub fn all_disabled(version: impl Into<String>) -> Self {
        Self::from_components(version, 0, HealthComponents::all_disabled())
    }
}

/// Computes aggregate status from component states (no HTTP).
#[must_use]
pub fn aggregate(components: &HealthComponents) -> AggregateStatus {
    if components.storage.status == ComponentStatus::Down {
        return AggregateStatus::Down;
    }

    let statuses = [
        components.storage.status,
        components.meilisearch.status,
        components.llm.status,
        components.speech.status,
        components.text_embeddings.status,
        components.sync.status,
    ];

    if statuses.contains(&ComponentStatus::Starting) {
        return AggregateStatus::Starting;
    }

    if statuses
        .iter()
        .any(|s| *s == ComponentStatus::Down || *s == ComponentStatus::Degraded)
    {
        return AggregateStatus::Degraded;
    }

    AggregateStatus::Ok
}

/// Port for publishing and reading health snapshots (no sidecar probes).
pub trait HealthReport: Send + Sync {
    fn snapshot(&self) -> HealthSnapshot;
}

impl HealthReport for HealthSnapshot {
    fn snapshot(&self) -> HealthSnapshot {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn components_with(
        mut components: HealthComponents,
        id: ComponentId,
        status: ComponentStatus,
    ) -> HealthComponents {
        let ch = ComponentHealth::new(status);
        match id {
            ComponentId::Storage => components.storage = ch,
            ComponentId::Meilisearch => components.meilisearch = ch,
            ComponentId::Llm => components.llm = ch,
            ComponentId::Speech => components.speech = ch,
            ComponentId::TextEmbeddings => components.text_embeddings = ch,
            ComponentId::Sync => components.sync = ch,
        }
        components
    }

    #[test]
    fn aggregate_all_up_is_ok() {
        assert_eq!(aggregate(&HealthComponents::all_up()), AggregateStatus::Ok);
    }

    #[test]
    fn aggregate_storage_down_is_down() {
        let c = components_with(
            HealthComponents::all_up(),
            ComponentId::Storage,
            ComponentStatus::Down,
        );
        assert_eq!(aggregate(&c), AggregateStatus::Down);
    }

    #[test]
    fn aggregate_any_starting_is_starting() {
        let c = components_with(
            HealthComponents::all_up(),
            ComponentId::Meilisearch,
            ComponentStatus::Starting,
        );
        assert_eq!(aggregate(&c), AggregateStatus::Starting);
    }

    #[test]
    fn aggregate_any_down_or_degraded_is_degraded() {
        let c = components_with(
            HealthComponents::all_up(),
            ComponentId::Meilisearch,
            ComponentStatus::Down,
        );
        assert_eq!(aggregate(&c), AggregateStatus::Degraded);
    }

    #[test]
    fn aggregate_disabled_does_not_degrade() {
        let mut c = HealthComponents::all_disabled();
        c.storage = ComponentHealth::new(ComponentStatus::Up);
        assert_eq!(aggregate(&c), AggregateStatus::Ok);
    }

    #[test]
    fn aggregate_storage_down_wins_over_starting() {
        let mut c = HealthComponents::all_up();
        c.storage = ComponentHealth::new(ComponentStatus::Down);
        c.meilisearch = ComponentHealth::new(ComponentStatus::Starting);
        assert_eq!(aggregate(&c), AggregateStatus::Down);
    }
}
