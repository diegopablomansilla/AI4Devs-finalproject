use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use api::{AppState, build_router};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct StaticFixture {
    web: PathBuf,
    media: PathBuf,
    _root: PathBuf,
}

impl StaticFixture {
    fn new() -> Self {
        let n = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("vortex-static-test-{}-{}", std::process::id(), n));
        fs::create_dir_all(&root).expect("fixture root");
        let web = root.join("web");
        let media = root.join("datos").join("media");
        fs::create_dir_all(&web).expect("web dir");
        fs::create_dir_all(&media).expect("media dir");
        Self {
            web,
            media,
            _root: root,
        }
    }

    fn with_app_shell(self) -> Self {
        fs::write(self.web.join("index.html"), "{{VERSION}}").expect("index.html");
        fs::write(self.web.join("sw.js"), "// sw").expect("sw.js");
        fs::write(
            self.web.join("manifest.webmanifest"),
            r#"{"name":"Vectra"}"#,
        )
        .expect("manifest");
        self
    }

    fn with_app_js(self, body: &str) -> Self {
        fs::write(self.web.join("app.js"), body).expect("app.js");
        self
    }

    fn with_media_jpg(self) -> Self {
        let dir = self.media.join("productos").join("01TEST");
        fs::create_dir_all(&dir).expect("media dir");
        fs::write(dir.join("3f2a9c.jpg"), b"fake-jpeg-bytes").expect("jpg");
        self
    }

    fn secret_outside_web(&self, contents: &str) -> PathBuf {
        let secret = self._root.join("secret.txt");
        fs::write(&secret, contents).expect("secret");
        secret
    }

    fn secret_outside_media(&self, contents: &str) -> PathBuf {
        let secret = self._root.join("secret-media.txt");
        fs::write(&secret, contents).expect("secret");
        secret
    }

    fn state(&self) -> AppState {
        AppState::new().with_static_roots(self.web.clone(), self.media.clone())
    }
}

fn api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

struct HttpResponse {
    status: u16,
    content_type: String,
    cache_control: String,
    body: String,
    problem: serde_json::Value,
}

fn parse_response(buf: &[u8]) -> HttpResponse {
    let response = String::from_utf8_lossy(buf);
    let status_line = response.lines().next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    let mut content_type = String::new();
    let mut cache_control = String::new();
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
                content_type = value.split(';').next().unwrap_or(value).trim().to_string();
            } else if name.eq_ignore_ascii_case("cache-control") {
                cache_control = value.to_string();
            }
        }
    }
    let body = String::from_utf8_lossy(&buf[body_start..]).into_owned();
    let problem = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
    HttpResponse {
        status,
        content_type,
        cache_control,
        body,
        problem,
    }
}

async fn http_request(addr: SocketAddr, method: &str, path: &str) -> HttpResponse {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let request = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut buf = vec![0u8; 64 * 1024];
    let n = stream.read(&mut buf).await.expect("read");
    parse_response(&buf[..n])
}

async fn serve(state: AppState) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    (addr, handle)
}

async fn wait_ready(addr: SocketAddr) {
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("server not ready");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn index_html_replaces_version_and_sends_no_cache() {
    let fixture = StaticFixture::new().with_app_shell();
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let root = http_request(addr, "GET", "/").await;
    let explicit = http_request(addr, "GET", "/index.html").await;
    handle.abort();

    let version = api_version();
    assert_eq!(root.status, 200);
    assert!(
        root.content_type.starts_with("text/html"),
        "ctype: {}",
        root.content_type
    );
    assert_eq!(root.cache_control, "no-cache");
    assert!(!root.cache_control.contains("no-store"));
    assert!(root.body.contains(version));
    assert!(!root.body.contains("{{VERSION}}"));

    assert_eq!(explicit.status, 200);
    assert!(!explicit.body.contains("{{VERSION}}"));
    assert_eq!(explicit.cache_control, "no-cache");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn index_html_missing_returns_404() {
    let fixture = StaticFixture::new();
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/").await;
    handle.abort();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.content_type, "application/problem+json");
    assert_eq!(resp.problem["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_index_is_not_200_html() {
    let fixture = StaticFixture::new().with_app_shell();
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "POST", "/").await;
    handle.abort();

    assert!(
        resp.status != 200
            || !resp.content_type.starts_with("text/html")
            || resp.body.contains("{{VERSION}}"),
        "POST / must not be 200 HTML with substituted index"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sw_js_sends_no_cache() {
    let fixture = StaticFixture::new().with_app_shell();
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/sw.js").await;
    handle.abort();

    assert_eq!(resp.status, 200);
    assert_eq!(resp.cache_control, "no-cache");
    assert!(!resp.cache_control.contains("no-store"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manifest_sends_no_cache() {
    let fixture = StaticFixture::new().with_app_shell();
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/manifest.webmanifest").await;
    handle.abort();

    assert_eq!(resp.status, 200);
    assert_eq!(resp.cache_control, "no-cache");
    assert!(!resp.cache_control.contains("no-store"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manifest_replaces_version() {
    let fixture = StaticFixture::new().with_app_shell();
    fs::write(
        fixture.web.join("manifest.webmanifest"),
        r#"{"name":"Vectra","icons":[{"src":"/static/{{VERSION}}/icons/icon-192.png"}]}"#,
    )
    .expect("manifest with placeholder");
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/manifest.webmanifest").await;
    handle.abort();

    assert_eq!(resp.status, 200);
    assert!(
        resp.content_type.starts_with("application/manifest+json"),
        "content-type: {}",
        resp.content_type
    );
    assert_eq!(resp.cache_control, "no-cache");
    assert!(!resp.body.contains("{{VERSION}}"));
    assert!(resp.body.contains(api_version()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn static_asset_sends_immutable_cache() {
    let fixture = StaticFixture::new().with_app_js("console.log('v');");
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let path = format!("/static/{}/app.js", api_version());
    let resp = http_request(addr, "GET", &path).await;
    handle.abort();

    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, "console.log('v');");
    assert!(resp.cache_control.contains("max-age=31536000"));
    assert!(resp.cache_control.contains("immutable"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn static_asset_wrong_version_returns_404() {
    let fixture = StaticFixture::new().with_app_js("leak-me");
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/static/not-the-version/app.js").await;
    handle.abort();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.problem["code"], "not_found");
    assert!(!resp.body.contains("leak-me") || resp.content_type == "application/problem+json");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn static_path_traversal_returns_404() {
    let fixture = StaticFixture::new().with_app_js("ok");
    fixture.secret_outside_web("top-secret");
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let ver = api_version();
    let paths = [
        format!("/static/{ver}/../secret.txt"),
        format!("/static/{ver}/..%2fsecret.txt"),
    ];
    for path in paths {
        let resp = http_request(addr, "GET", &path).await;
        assert_eq!(resp.status, 404, "path {path}");
        assert_eq!(resp.problem["code"], "not_found");
        assert!(
            !resp.body.contains("top-secret"),
            "must not leak secret for {path}"
        );
    }
    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn media_file_sends_long_cache() {
    let fixture = StaticFixture::new().with_media_jpg();
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/media/productos/01TEST/3f2a9c.jpg").await;
    handle.abort();

    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, "fake-jpeg-bytes");
    assert!(resp.cache_control.contains("max-age=31536000"));
    assert!(resp.cache_control.contains("immutable"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn media_missing_returns_404() {
    let fixture = StaticFixture::new();
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/media/no-such-object.bin").await;
    handle.abort();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.problem["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn media_path_traversal_returns_404() {
    let fixture = StaticFixture::new();
    fixture.secret_outside_media("media-secret");
    let (addr, handle) = serve(fixture.state()).await;
    wait_ready(addr).await;

    let resp = http_request(addr, "GET", "/media/../secret-media.txt").await;
    handle.abort();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.problem["code"], "not_found");
    assert!(!resp.body.contains("media-secret"));
}
