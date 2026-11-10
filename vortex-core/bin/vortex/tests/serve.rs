use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static SERVE_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn vortex_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vortex"))
}

fn minimal_valid_toml(bind: &str) -> String {
    format!(
        r#"
[paths]
datos = "/tmp/datos"
modelos = "/tmp/modelos"
backups = "/tmp/backups"
web = "/tmp/web"

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
"#
    )
    .trim()
    .to_string()
}

fn write_config(dir: &Path, bind: &str) -> PathBuf {
    let path = dir.join("vortex.toml");
    fs::write(&path, minimal_valid_toml(bind)).expect("write config");
    path
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

fn temp_dir() -> PathBuf {
    let n = SERVE_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("vortex-serve-test-{}-{}", std::process::id(), n));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
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
    let path = dir.join("bad.toml");
    fs::write(
        &path,
        minimal_valid_toml("127.0.0.1:8080").replace("moneda = \"ARS\"", "moneda = \"peso\""),
    )
    .expect("write bad config");
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
    let mut child = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap(), "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn serve");
    let problem = wait_for_http_404(&bind, Duration::from_secs(5))
        .expect("serve must listen and return 404 problem+json");
    assert_eq!(problem.1["code"], "not_found");
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
    let status = child.wait().expect("wait");
    assert_eq!(status.code(), Some(0));
}

#[test]
fn vortex_serve_get_health_returns_json() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap(), "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn serve");
    let addr: SocketAddr = bind.parse().expect("bind addr");
    let start = Instant::now();
    let mut ok = false;
    while start.elapsed() < Duration::from_secs(5) {
        if let Some((status, ctype, json)) = http_get_problem(addr, "/health") {
            if status == 200
                && ctype == "application/json"
                && json.get("status").and_then(|v| v.as_str()) == Some("ok")
                && json.get("code").is_none()
                && json.get("uptimeSeconds").is_some()
                && json.get("version").is_some()
                && json["components"]["storage"].is_object()
                && json["components"]["meilisearch"].is_object()
                && json["components"]["llm"].is_object()
                && json["components"]["speech"].is_object()
                && json["components"]["textEmbeddings"].is_object()
                && json["components"]["sync"].is_object()
            {
                ok = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
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
    assert!(ok, "GET /health must return 200 Health JSON");
}

#[cfg(unix)]
fn spawn_serve(_bind: &str, config_path: &Path) -> Child {
    Command::new(vortex_bin())
        .args(["--config", config_path.to_str().unwrap(), "serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn serve")
}

#[cfg(unix)]
#[test]
fn vortex_serve_exits_clean_on_sigterm() {
    let bind = ephemeral_bind();
    let dir = temp_dir();
    let path = write_config(&dir, &bind);
    let mut child = spawn_serve(&bind, &path);
    assert!(
        wait_for_http_404(&bind, Duration::from_secs(5)).is_some(),
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
        wait_for_http_404(&bind, Duration::from_secs(5)).is_some(),
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
