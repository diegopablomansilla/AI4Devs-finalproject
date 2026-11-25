//! Meilisearch integration probe (T-09-05).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn meilisearch_base_url() -> String {
    std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".to_string())
}

fn parse_host_port(url: &str) -> (&str, u16) {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or_else(|| panic!("MEILISEARCH_URL debe ser http(s)://host:puerto: {url}"));
    let (host, port) = rest
        .split_once(':')
        .unwrap_or_else(|| panic!("MEILISEARCH_URL debe incluir puerto: {url}"));
    (
        host,
        port.parse()
            .unwrap_or_else(|_| panic!("puerto inválido en MEILISEARCH_URL: {url}")),
    )
}

fn get_health_body(base: &str) -> Result<(u16, String), String> {
    let (host, port) = parse_host_port(base);
    let addr = format!("{host}:{port}");
    let mut stream =
        TcpStream::connect(&addr).map_err(|e| format!("conexión a /health en {base}: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let request = format!("GET /health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("/health write: {e}"))?;
    let mut buf = vec![0u8; 8192];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("/health read: {e}"))?;
    let response = String::from_utf8_lossy(&buf[..n]);
    let status_line = response
        .lines()
        .next()
        .ok_or_else(|| "GET /health sin línea de estado".to_string())?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| format!("status HTTP inválido: {status_line}"))?
        .parse()
        .map_err(|_| format!("status HTTP no numérico: {status_line}"))?;
    let body_start = response.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = response[body_start..].to_string();
    Ok((status, body))
}

#[test]
#[ignore = "meilisearch"]
fn meilisearch_health_is_available() {
    let base = meilisearch_base_url();
    let (status, body) =
        get_health_body(&base).unwrap_or_else(|e| panic!("GET /health contra Meilisearch: {e}"));
    assert_eq!(status, 200, "GET /health status");
    assert!(
        body.contains("\"status\":\"available\"") || body.contains("\"status\": \"available\""),
        "GET /health body debe tener status available: {body}"
    );
}
