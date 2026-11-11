use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static SERVE_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

const BOOT_WAIT: Duration = Duration::from_secs(15);

fn vortex_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vortex"))
}

fn temp_dir() -> PathBuf {
    let n = SERVE_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("vortex-serve-test-{}-{}", std::process::id(), n));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write_config(dir: &Path, bind: &str) -> PathBuf {
    let datos = dir.join("datos");
    fs::create_dir_all(&datos).expect("datos dir");
    let web = dir.join("web");
    fs::create_dir_all(&web).expect("web dir");
    write_config_with_paths(dir, bind, &datos, &web)
}

fn write_config_with_paths(dir: &Path, bind: &str, datos: &Path, web: &Path) -> PathBuf {
    let path = dir.join("vortex.toml");
    let body = format!(
        r#"
[paths]
datos = "{datos}"
modelos = "/tmp/modelos"
backups = "/tmp/backups"
web = "{web}"

[server]
bind = "{bind}"
hostname = "localhost"

[negocio]
moneda = "ARS"

[meilisearch]
url = "http://127.0.0.1:7700"

[ollama]
url = "http://127.0.0.1:11434"

[backup]
hora = "03:00"
retencion = 14

[logs]
nivel = "info"
retencion = 14
"#,
        datos = datos.display(),
        web = web.display(),
        bind = bind,
    );
    fs::write(&path, body.trim()).expect("write config");
    path
}

fn http_get_headers(addr: SocketAddr, path: &str) -> Option<(u16, String, String, String)> {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(addr).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let host = format!("{}:{}", addr.ip(), addr.port());
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).ok()?;
    let response = String::from_utf8_lossy(&buf[..n]);
    let status_line = response.lines().next()?;
    let status: u16 = status_line.split_whitespace().nth(1)?.parse().ok()?;
    let mut ctype = String::new();
    let mut cache = String::new();
    let mut body_start = 0usize;
    for (i, _) in response.match_indices("\r\n\r\n") {
        body_start = i + 4;
        break;
    }
    for line in response.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim();
            let value = value.trim();
            if name.eq_ignore_ascii_case("content-type") {
                ctype = value.split(';').next().unwrap_or(value).trim().to_string();
            } else if name.eq_ignore_ascii_case("cache-control") {
                cache = value.to_string();
            }
        }
    }
    let body = String::from_utf8_lossy(&buf[body_start..n]).into_owned();
    Some((status, ctype, cache, body))
}

fn ephemeral_bind() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);
    addr.to_string()
}

fn http_get_problem(addr: SocketAddr, path: &str) -> Option<(u16, String, serde_json::Value)> {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(addr).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let host = format!("{}:{}", addr.ip(), addr.port());
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).ok()?;
    let response = String::from_utf8_lossy(&buf[..n]);
    let status_line = response.lines().next()?;
    let status: u16 = status_line.split_whitespace().nth(1)?.parse().ok()?;
    let mut ctype = String::new();
    let mut body_start = 0usize;
    for (i, _) in response.match_indices("\r\n\r\n") {
        body_start = i + 4;
        break;
    }
    for line in response.lines() {
        if line.is_empty() {
            break;
        }
        if let Some(v) = line
            .strip_prefix("Content-Type:")
            .or_else(|| line.strip_prefix("content-type:"))
        {
            ctype = v.trim().split(';').next().unwrap_or(v).trim().to_string();
        }
    }
    let json: serde_json::Value = serde_json::from_slice(&buf[body_start..n]).ok()?;
    Some((status, ctype, json))
}

fn wait_for_http_404(bind: &str, timeout: Duration) -> Option<(String, serde_json::Value)> {
    let addr: SocketAddr = bind.parse().expect("bind addr");
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some((status, ctype, json)) = http_get_problem(addr, "/no-such-route") {
            if status == 404 && ctype == "application/problem+json" {
                return Some((ctype, json));
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    None
}

fn wait_for_health_ok_storage_up(bind: &str, timeout: Duration) -> Option<serde_json::Value> {
    let addr: SocketAddr = bind.parse().expect("bind addr");
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some((status, ctype, json)) = http_get_problem(addr, "/health") {
            if status == 200
                && ctype == "application/json"
                && json.get("status").and_then(|v| v.as_str()) == Some("ok")
                && json.get("code").is_none()
                && json["components"]["storage"]
                    .get("status")
                    .and_then(|v| v.as_str())
                    == Some("up")
            {
                return Some(json);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    None
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status();
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

fn spawn_serve_production(config_path: &Path) -> Child {
    Command::new(vortex_bin())
        .args(["--config", config_path.to_str().unwrap(), "serve"])
        .env("VORTEX_ENV", "production")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn serve")
}

fn read_log_json_lines(logs_dir: &Path) -> Vec<serde_json::Value> {
    let mut lines = Vec::new();
    let Ok(entries) = fs::read_dir(logs_dir) else {
        return lines;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("vortex.log") {
            continue;
        }
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                lines.push(value);
            }
        }
    }
    lines
}

fn json_mentions_boot_phase(value: &serde_json::Value, phase: &str) -> bool {
    if value
        .get("span")
        .and_then(|s| s.get("name"))
        .and_then(|n| n.as_str())
        == Some(phase)
    {
        return true;
    }
    if let Some(spans) = value.get("spans").and_then(|s| s.as_array()) {
        if spans.iter().any(|span| {
            span.get("name")
                .and_then(|n| n.as_str())
                .is_some_and(|n| n == phase)
        }) {
            return true;
        }
    }
    if value.get("target").and_then(|t| t.as_str()) == Some(phase) {
        return true;
    }
    if let Some(fields) = value.get("fields") {
        if fields.get("phase").and_then(|p| p.as_str()) == Some(phase) {
            return true;
        }
        if fields
            .get("message")
            .and_then(|m| m.as_str())
            .is_some_and(|m| m.contains(phase))
        {
            return true;
        }
    }
    value
        .get("message")
        .and_then(|m| m.as_str())
        .is_some_and(|m| m.contains(phase))
}

fn json_has_duration(value: &serde_json::Value) -> bool {
    let check_obj = |obj: &serde_json::Value| {
        for key in [
            "duration_ms",
            "duration",
            "elapsed",
            "busy",
            "busy_ns",
            "time.busy",
        ] {
            if let Some(v) = obj.get(key) {
                if v.is_number() || v.as_str().is_some() {
                    return true;
                }
            }
        }
        false
    };
    if check_obj(value) {
        return true;
    }
    if let Some(fields) = value.get("fields") {
        if check_obj(fields) {
            return true;
        }
    }
    if let Some(span) = value.get("span") {
        if check_obj(span) {
            return true;
        }
    }
    false
}

fn port_is_free(bind: &str) -> bool {
    let addr: SocketAddr = bind.parse().expect("bind addr");
    // Brief settle after process exit.
    std::thread::sleep(Duration::from_millis(100));
    TcpListener::bind(addr).is_ok()
}

#[test]
fn vortex_serve_without_config_fails() {
    let output = Command::new(vortex_bin())
        .arg("serve")
        .env_remove("VORTEX_CONFIG")
        .output()
        .expect("spawn vortex");
    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
    assert!(
        stderr.contains("--config") || stderr.contains("VORTEX_CONFIG"),
        "stderr: {stderr}"
    );
}

#[test]
fn vortex_serve_missing_config_file_fails() {
    let output = Command::new(vortex_bin())
        .args(["--config", "/nonexistent/vortex-config.toml", "serve"])
        .output()
        .expect("spawn vortex");
    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
}

#[test]
fn vortex_serve_invalid_config_fails() {
    let dir = temp_dir();
    let datos = dir.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let path = dir.join("bad.toml");
    let body = format!(
        r#"
[paths]
datos = "{datos}"
modelos = "/tmp/modelos"
backups = "/tmp/backups"
web = "/tmp/web"

[server]
bind = "127.0.0.1:8080"
hostname = "localhost"

[negocio]
moneda = "peso"

[meilisearch]
url = "http://127.0.0.1:7700"

[ollama]
url = "http://127.0.0.1:11434"

[backup]
hora = "03:00"
retencion = 14

[logs]
nivel = "info"
retencion = 14
"#,
        datos = datos.display(),
    );
    fs::write(&path, body.trim()).expect("write bad config");
    let output = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap(), "serve"])
        .output()
        .expect("spawn vortex");
    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
    assert!(
        stderr.contains("negocio.moneda") || stderr.contains("moneda"),
        "stderr: {stderr}"
    );
}

#[test]
fn vortex_serve_bind_in_use_fails() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let bind = listener.local_addr().expect("addr").to_string();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let output = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap(), "serve"])
        .env("VORTEX_ENV", "production")
        .output()
        .expect("spawn vortex");
    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
}

#[test]
fn vortex_serve_starts_and_returns_404() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = spawn_serve_production(&path);
    let problem =
        wait_for_http_404(&bind, BOOT_WAIT).expect("serve must listen and return 404 problem+json");
    assert_eq!(problem.1["code"], "not_found");
    terminate_child(&mut child);
}

#[test]
fn vortex_serve_serves_index_from_paths_web() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let web = dir.join("web");
    fs::create_dir_all(&web).expect("web dir");
    fs::write(web.join("index.html"), "{{VERSION}}").expect("index.html");
    let datos = dir.join("datos");
    fs::create_dir_all(&datos).expect("datos dir");
    let path = write_config_with_paths(&dir, &bind, &datos, &web);
    let mut child = spawn_serve_production(&path);
    let addr: SocketAddr = bind.parse().expect("bind addr");
    let start = Instant::now();
    let mut ok = false;
    while start.elapsed() < BOOT_WAIT {
        if let Some((status, ctype, cache, body)) = http_get_headers(addr, "/") {
            if status == 200
                && ctype.starts_with("text/html")
                && cache == "no-cache"
                && !body.contains("{{VERSION}}")
                && !body.is_empty()
            {
                ok = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    terminate_child(&mut child);
    assert!(
        ok,
        "GET / must serve index.html from paths.web with no-cache"
    );
}

#[test]
fn vortex_serve_get_health_returns_json() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = spawn_serve_production(&path);
    let json = wait_for_health_ok_storage_up(&bind, BOOT_WAIT)
        .expect("GET /health must return 200 Health JSON with storage up");
    assert!(json.get("uptimeSeconds").is_some());
    assert!(json.get("version").is_some());
    assert!(json["components"]["meilisearch"].is_object());
    assert!(json["components"]["llm"].is_object());
    assert!(json["components"]["speech"].is_object());
    assert!(json["components"]["textEmbeddings"].is_object());
    assert!(json["components"]["sync"].is_object());
    terminate_child(&mut child);
}

#[test]
fn vortex_serve_health_ok_with_storage_up() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = spawn_serve_production(&path);
    let json = wait_for_health_ok_storage_up(&bind, BOOT_WAIT)
        .expect("serve must reach /health ok with storage up");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["components"]["storage"]["status"], "up");
    assert!(json.get("code").is_none());
    terminate_child(&mut child);
}

#[test]
fn vortex_serve_boot_logs_phase_durations() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let datos = dir.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    let web = dir.join("web");
    fs::create_dir_all(&web).expect("web");
    let path = write_config_with_paths(&dir, &bind, &datos, &web);
    let mut child = spawn_serve_production(&path);
    assert!(
        wait_for_health_ok_storage_up(&bind, BOOT_WAIT).is_some(),
        "serve must start before reading boot logs"
    );
    // Give the non-blocking appender a moment to flush.
    std::thread::sleep(Duration::from_millis(200));
    terminate_child(&mut child);

    let logs_dir = datos.join("logs");
    let lines = read_log_json_lines(&logs_dir);
    assert!(
        !lines.is_empty(),
        "expected JSON log lines under {}",
        logs_dir.display()
    );

    for phase in [
        "boot.config",
        "boot.logging",
        "boot.storage",
        "boot.health",
        "boot.api",
    ] {
        let matching: Vec<_> = lines
            .iter()
            .filter(|v| json_mentions_boot_phase(v, phase))
            .collect();
        assert!(
            !matching.is_empty(),
            "missing boot phase {phase} in logs: {lines:?}"
        );
        assert!(
            matching.iter().any(|v| json_has_duration(v)),
            "boot phase {phase} must record a duration: {matching:?}"
        );
    }
}

fn wait_child_exit(mut child: Child, timeout: Duration) -> (std::process::ExitStatus, String) {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stderr = String::new();
                if let Some(mut err) = child.stderr.take() {
                    use std::io::Read;
                    let _ = err.read_to_string(&mut stderr);
                }
                return (status, stderr);
            }
            Ok(None) if start.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("vortex serve should have exited within {timeout:?}");
            }
            Err(e) => panic!("try_wait: {e}"),
        }
    }
}

#[test]
fn vortex_serve_inaccessible_storage_exits() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let datos = dir.join("datos");
    fs::create_dir_all(&datos).expect("datos");
    // File where RocksDB directory must be → OpenError.
    fs::write(datos.join("db"), b"not-a-rocksdb-dir").expect("db as file");
    let web = dir.join("web");
    fs::create_dir_all(&web).expect("web");
    let path = write_config_with_paths(&dir, &bind, &datos, &web);

    let child = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap(), "serve"])
        .env("VORTEX_ENV", "production")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vortex");
    let (status, stderr) = wait_child_exit(child, Duration::from_secs(10));
    assert_ne!(status.code(), Some(0));
    assert!(port_is_free(&bind), "API must not be listening");
    let stderr_l = stderr.to_lowercase();
    let logs = read_log_json_lines(&datos.join("logs"));
    let log_blob = format!("{logs:?}").to_lowercase();
    assert!(
        stderr_l.contains("storage")
            || stderr_l.contains("abrir")
            || stderr_l.contains("base")
            || stderr_l.contains("db")
            || stderr_l.contains("archivo")
            || log_blob.contains("storage")
            || log_blob.contains("abrir")
            || log_blob.contains("base"),
        "stderr or logs must mention storage failure; stderr={stderr}; logs={log_blob}"
    );
}

#[test]
fn vortex_serve_unwritable_datos_exits() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let datos_file = dir.join("datos-as-file");
    fs::write(&datos_file, b"not-a-dir").expect("datos as file");
    let web = dir.join("web");
    fs::create_dir_all(&web).expect("web");
    let path = write_config_with_paths(&dir, &bind, &datos_file, &web);

    let child = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap(), "serve"])
        .env("VORTEX_ENV", "production")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vortex");
    let (status, stderr) = wait_child_exit(child, Duration::from_secs(10));
    assert_ne!(status.code(), Some(0));
    assert!(port_is_free(&bind), "API must not be listening");
    assert!(
        stderr.contains("paths.datos") || stderr.contains("logs"),
        "stderr must name paths.datos or logs: {stderr}"
    );
}

fn http_get_text(addr: SocketAddr, path: &str) -> Option<(u16, String, String)> {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(addr).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let host = format!("{}:{}", addr.ip(), addr.port());
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut buf = vec![0u8; 65536];
    let n = stream.read(&mut buf).ok()?;
    let response = String::from_utf8_lossy(&buf[..n]);
    let status_line = response.lines().next()?;
    let status: u16 = status_line.split_whitespace().nth(1)?.parse().ok()?;
    let mut ctype = String::new();
    let mut body_start = 0usize;
    for (i, _) in response.match_indices("\r\n\r\n") {
        body_start = i + 4;
        break;
    }
    for line in response.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-type") {
                ctype = value
                    .trim()
                    .split(';')
                    .next()
                    .unwrap_or(value.trim())
                    .to_string();
            }
        }
    }
    let body = String::from_utf8_lossy(&buf[body_start..n]).into_owned();
    Some((status, ctype, body))
}

#[test]
fn vortex_serve_get_metrics_returns_prometheus() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = spawn_serve_production(&path);
    let addr: SocketAddr = bind.parse().expect("bind addr");
    let start = Instant::now();
    let mut ok = false;
    while start.elapsed() < BOOT_WAIT {
        let _ = http_get_text(addr, "/health");
        if let Some((status, ctype, body)) = http_get_text(addr, "/metrics") {
            if status == 200
                && ctype.starts_with("text/plain")
                && body.contains("http_request_duration_seconds")
            {
                ok = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    terminate_child(&mut child);
    assert!(
        ok,
        "GET /metrics must return 200 Prometheus text with HTTP histogram"
    );
}

#[cfg(unix)]
fn spawn_serve(_bind: &str, config_path: &Path) -> Child {
    spawn_serve_production(config_path)
}

#[cfg(unix)]
#[test]
fn vortex_serve_exits_clean_on_sigterm() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = spawn_serve(&bind, &path);
    assert!(
        wait_for_http_404(&bind, BOOT_WAIT).is_some(),
        "server did not start"
    );
    Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .expect("kill -TERM");
    let status = child.wait().expect("wait after SIGTERM");
    assert_eq!(status.code(), Some(0));
    assert!(
        TcpListener::bind(bind.parse::<SocketAddr>().unwrap()).is_ok(),
        "port must be free after graceful shutdown"
    );
}

#[cfg(unix)]
#[test]
fn vortex_serve_exits_clean_on_sigint() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = spawn_serve(&bind, &path);
    assert!(
        wait_for_http_404(&bind, BOOT_WAIT).is_some(),
        "server did not start"
    );
    Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("kill -INT");
    let status = child.wait().expect("wait after SIGINT");
    assert_eq!(status.code(), Some(0));
    assert!(
        TcpListener::bind(bind.parse::<SocketAddr>().unwrap()).is_ok(),
        "port must be free after graceful shutdown"
    );
}
