use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::Value;

static OPENAPI: OnceLock<Value> = OnceLock::new();

fn openapi_document() -> &'static Value {
    OPENAPI.get_or_init(|| {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = manifest.join("../../../docs/openapi.yaml");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("parse openapi.yaml: {e}"));
        serde_json::to_value(yaml).expect("openapi yaml to json value")
    })
}

fn decode_pointer_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

fn json_pointer_get<'a>(doc: &'a Value, pointer: &str) -> Result<&'a Value, String> {
    if !pointer.starts_with("#/") {
        return Err(format!("unsupported $ref: {pointer}"));
    }
    let path = pointer.strip_prefix("#/").unwrap();
    let mut current = doc;
    for segment in path.split('/').filter(|s| !s.is_empty()) {
        let key = decode_pointer_segment(segment);
        current = current
            .get(&key)
            .ok_or_else(|| format!("$ref segment not found: {key} in {pointer}"))?;
    }
    Ok(current)
}

fn inline_refs(doc: &Value, node: &Value) -> Result<Value, String> {
    if let Some(obj) = node.as_object() {
        if let Some(Value::String(r)) = obj.get("$ref") {
            let target = json_pointer_get(doc, r)?;
            return inline_refs(doc, target);
        }
        let mut out = serde_json::Map::new();
        for (k, v) in obj {
            out.insert(k.clone(), inline_refs(doc, v)?);
        }
        return Ok(Value::Object(out));
    }
    if let Some(arr) = node.as_array() {
        let mut out = Vec::with_capacity(arr.len());
        for v in arr {
            out.push(inline_refs(doc, v)?);
        }
        return Ok(Value::Array(out));
    }
    Ok(node.clone())
}

fn normalize_mime(mime: &str) -> String {
    mime.split(';')
        .next()
        .unwrap_or(mime)
        .trim()
        .to_ascii_lowercase()
}

fn path_segments(path: &str) -> Vec<&str> {
    path.trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect()
}

fn path_matches(template: &str, path: &str) -> bool {
    let t = path_segments(template);
    let p = path_segments(path);
    if t.len() != p.len() {
        return false;
    }
    t.iter()
        .zip(p.iter())
        .all(|(ti, pi)| (ti.starts_with('{') && ti.ends_with('}')) || ti == pi)
}

fn find_path_item<'a>(doc: &'a Value, path: &str) -> Result<&'a Value, String> {
    let paths = doc
        .get("paths")
        .ok_or_else(|| "openapi: missing paths".to_string())?;
    if let Some(item) = paths.get(path) {
        return Ok(item);
    }
    let Some(obj) = paths.as_object() else {
        return Err(format!("no path item for {path}"));
    };
    for (template, item) in obj {
        if path_matches(template, path) {
            return Ok(item);
        }
    }
    Err(format!("no path item for {path}"))
}

fn schema_for_response_media(doc: &Value, media: &Value) -> Result<Value, String> {
    let schema = media
        .get("schema")
        .ok_or_else(|| "response media missing schema".to_string())?;
    inline_refs(doc, schema)
}

fn schema_for(
    doc: &Value,
    method: &str,
    path: &str,
    status: u16,
    content_type: &str,
) -> Result<Value, String> {
    let path_item = find_path_item(doc, path)?;
    let method = method.to_ascii_lowercase();
    let operation = path_item
        .get(&method)
        .ok_or_else(|| format!("no operation {method} {path}"))?;
    let responses = operation
        .get("responses")
        .ok_or_else(|| format!("no responses for {method} {path}"))?;
    let status_key = status.to_string();
    let response = responses
        .get(&status_key)
        .ok_or_else(|| format!("no response {status} for {method} {path}"))?;
    let response = inline_refs(doc, response)?;
    let content = response
        .get("content")
        .ok_or_else(|| format!("no content for {method} {path} {status}"))?;
    let mime = normalize_mime(content_type);
    let Some(content_obj) = content.as_object() else {
        return Err(format!(
            "no content type {content_type} for {method} {path} {status}"
        ));
    };
    let media = content_obj
        .iter()
        .find(|(k, _)| normalize_mime(k) == mime)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("no content type {content_type} for {method} {path} {status}"))?;
    schema_for_response_media(doc, media)
}

pub fn conforms(
    method: &str,
    path: &str,
    status: u16,
    content_type: &str,
    body: &Value,
) -> Result<(), String> {
    let doc = openapi_document();
    let schema = schema_for(doc, method, path, status, content_type)?;
    let validator = jsonschema::draft202012::new(&schema).map_err(|e| e.to_string())?;
    let errors: Vec<String> = validator.iter_errors(body).map(|e| e.to_string()).collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[allow(dead_code)]
pub fn assert_conforms(method: &str, path: &str, status: u16, content_type: &str, body: &Value) {
    conforms(method, path, status, content_type, body).expect("response conforms to openapi.yaml");
}
