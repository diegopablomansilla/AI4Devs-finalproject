//! Static SPA and media file serving (T-00-19).

use std::path::{Component, Path, PathBuf};

use axum::{
    extract::{Path as AxumPath, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{ApiError, AppState};

const CACHE_NO_STORE: &str = "no-cache";
const CACHE_IMMUTABLE: &str = "max-age=31536000, immutable";

const PLACEHOLDER_VERSION: &str = "{{VERSION}}";

pub(crate) fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    if rel.is_empty() || rel.contains('\0') {
        return None;
    }
    let rel_path = Path::new(rel);
    for comp in rel_path.components() {
        match comp {
            Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_)
            | Component::CurDir => {
                if !matches!(comp, Component::CurDir) {
                    return None;
                }
            }
            Component::Normal(_) => {}
        }
    }

    let root = root.canonicalize().ok()?;
    let candidate = root.join(rel_path);
    if !candidate.exists() {
        return None;
    }
    let file = candidate.canonicalize().ok()?;
    if !file.starts_with(&root) {
        return None;
    }
    if !file.is_file() {
        return None;
    }
    Some(file)
}

fn content_type_for_path(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "webmanifest" => "application/manifest+json",
        "json" => "application/json",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn file_response(path: &Path, cache_control: &str) -> Response {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return ApiError::route_not_found(path.display().to_string()).into_response(),
    };
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(content_type_for_path(path)),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_str(cache_control).expect("cache header"),
    );
    (StatusCode::OK, headers, bytes).into_response()
}

fn not_found(instance: &str) -> Response {
    ApiError::route_not_found(instance.to_owned()).into_response()
}

pub async fn get_index(State(state): State<AppState>) -> Response {
    let web = state.web_root();
    let index = web.join("index.html");
    if !index.is_file() {
        return not_found("/");
    }
    let html = match std::fs::read_to_string(&index) {
        Ok(s) => s,
        Err(_) => return not_found("/"),
    };
    let body = html.replace(PLACEHOLDER_VERSION, &state.version());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_NO_STORE),
    );
    (StatusCode::OK, headers, body).into_response()
}

pub async fn get_sw_js(State(state): State<AppState>) -> Response {
    let path = state.web_root().join("sw.js");
    if !path.is_file() {
        return not_found("/sw.js");
    }
    file_response(&path, CACHE_NO_STORE)
}

pub async fn get_manifest(State(state): State<AppState>) -> Response {
    let path = state.web_root().join("manifest.webmanifest");
    if !path.is_file() {
        return not_found("/manifest.webmanifest");
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return not_found("/manifest.webmanifest"),
    };
    let body = text.replace(PLACEHOLDER_VERSION, &state.version());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/manifest+json"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_NO_STORE),
    );
    (StatusCode::OK, headers, body).into_response()
}

pub async fn get_versioned_static(
    State(state): State<AppState>,
    AxumPath((version, tail)): AxumPath<(String, String)>,
) -> Response {
    if version != state.version() {
        return not_found(&format!("/static/{version}/{tail}"));
    }
    let rel = tail.trim_start_matches('/');
    let instance = format!("/static/{version}/{rel}");
    let Some(path) = safe_join(&state.web_root(), rel) else {
        return not_found(&instance);
    };
    file_response(&path, CACHE_IMMUTABLE)
}

pub async fn get_media(
    State(state): State<AppState>,
    AxumPath(tail): AxumPath<String>,
) -> Response {
    let rel = tail.trim_start_matches('/');
    let instance = format!("/media/{rel}");
    let Some(path) = safe_join(&state.media_root(), rel) else {
        return not_found(&instance);
    };
    file_response(&path, CACHE_IMMUTABLE)
}
