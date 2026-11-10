//! `GET /health` handler and OpenAPI-shaped JSON (T-00-13).

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::{AggregateStatus, ComponentStatus, HealthSnapshot};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
    version: String,
    uptime_seconds: u64,
    components: ComponentsResponse,
}

#[derive(Serialize)]
struct ComponentsResponse {
    storage: ComponentResponse,
    meilisearch: ComponentResponse,
    llm: ComponentResponse,
    speech: ComponentResponse,
    #[serde(rename = "textEmbeddings")]
    text_embeddings: ComponentResponse,
    sync: ComponentResponse,
}

#[derive(Serialize)]
struct ComponentResponse {
    status: &'static str,
}

fn aggregate_status_str(s: AggregateStatus) -> &'static str {
    match s {
        AggregateStatus::Ok => "ok",
        AggregateStatus::Degraded => "degraded",
        AggregateStatus::Starting => "starting",
        AggregateStatus::Down => "down",
    }
}

fn component_status_str(s: ComponentStatus) -> &'static str {
    match s {
        ComponentStatus::Up => "up",
        ComponentStatus::Starting => "starting",
        ComponentStatus::Degraded => "degraded",
        ComponentStatus::Down => "down",
        ComponentStatus::Disabled => "disabled",
    }
}

fn to_response(snap: HealthSnapshot) -> HealthResponse {
    let c = &snap.components;
    HealthResponse {
        status: aggregate_status_str(snap.status),
        version: snap.version,
        uptime_seconds: snap.uptime_seconds,
        components: ComponentsResponse {
            storage: ComponentResponse {
                status: component_status_str(c.storage.status),
            },
            meilisearch: ComponentResponse {
                status: component_status_str(c.meilisearch.status),
            },
            llm: ComponentResponse {
                status: component_status_str(c.llm.status),
            },
            speech: ComponentResponse {
                status: component_status_str(c.speech.status),
            },
            text_embeddings: ComponentResponse {
                status: component_status_str(c.text_embeddings.status),
            },
            sync: ComponentResponse {
                status: component_status_str(c.sync.status),
            },
        },
    }
}

fn http_status_for_aggregate(s: AggregateStatus) -> StatusCode {
    match s {
        AggregateStatus::Ok | AggregateStatus::Degraded => StatusCode::OK,
        AggregateStatus::Starting | AggregateStatus::Down => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub async fn get_health(State(state): State<AppState>) -> Response {
    let snap = state.health().snapshot();
    let code = http_status_for_aggregate(snap.status);
    let body = to_response(snap);
    (code, Json(body)).into_response()
}
