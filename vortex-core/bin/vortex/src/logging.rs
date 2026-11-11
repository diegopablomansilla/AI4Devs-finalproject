//! Structured logging for Vortex Core (T-00-06); installed by `vortex serve` (T-00-20).

use crate::config::Config;
use std::fs;
use std::io::IsTerminal;
#[cfg(test)]
use std::io::{self, Write};
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(Debug, Error)]
#[error("logging: {key}: {reason}")]
pub struct LoggingError {
    pub key: String,
    pub reason: String,
}

impl LoggingError {
    pub fn new(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            reason: reason.into(),
        }
    }
}

/// Guard that flushes the non-blocking log worker when dropped (production init).
pub struct LoggingGuard {
    _worker_guard: WorkerGuard,
}

enum ConsoleSink {
    None,
    Stderr,
    #[cfg(test)]
    Capture(CapturingWriter),
}

#[cfg(test)]
#[derive(Clone)]
struct CapturingWriter(Arc<Mutex<Vec<u8>>>);

#[cfg(test)]
struct CapturingWriterGuard {
    inner: Arc<Mutex<Vec<u8>>>,
}

#[cfg(test)]
impl Write for CapturingWriterGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}

#[cfg(test)]
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CapturingWriter {
    type Writer = CapturingWriterGuard;

    fn make_writer(&'a self) -> Self::Writer {
        CapturingWriterGuard {
            inner: Arc::clone(&self.0),
        }
    }
}

/// Install the global subscriber for `vortex serve` (T-00-20).
pub fn init_logging(config: &Config) -> Result<LoggingGuard, LoggingError> {
    let vortex_env = std::env::var("VORTEX_ENV").ok();
    let sink = if console_pretty_enabled(vortex_env.as_deref()) {
        ConsoleSink::Stderr
    } else {
        ConsoleSink::None
    };
    let (guard, file_writer, filter) = prepare_file_logging(config)?;
    let init_result = match sink {
        ConsoleSink::None => Registry::default()
            .with(json_layer(file_writer, filter))
            .try_init(),
        ConsoleSink::Stderr => Registry::default()
            .with(json_layer(file_writer, filter.clone()))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_filter(filter),
            )
            .try_init(),
        #[cfg(test)]
        ConsoleSink::Capture(writer) => Registry::default()
            .with(json_layer(file_writer, filter.clone()))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_writer(writer)
                    .with_filter(filter),
            )
            .try_init(),
    };
    init_result.map_err(|e| LoggingError::new("logs", e.to_string()))?;
    Ok(LoggingGuard {
        _worker_guard: guard,
    })
}

/// Run `f` with a scoped subscriber (tests).
#[cfg(test)]
pub fn with_logging<F>(config: &Config, vortex_env: Option<&str>, f: F) -> Result<(), LoggingError>
where
    F: FnOnce(),
{
    with_logging_sink(config, vortex_env, ConsoleSink::None, f)
}

#[cfg(test)]
pub(crate) fn with_logging_capture<F>(
    config: &Config,
    vortex_env: Option<&str>,
    capture: Arc<Mutex<Vec<u8>>>,
    f: F,
) -> Result<(), LoggingError>
where
    F: FnOnce(),
{
    with_logging_sink(
        config,
        vortex_env,
        ConsoleSink::Capture(CapturingWriter(capture)),
        f,
    )
}

#[cfg(test)]
fn with_logging_sink<F>(
    config: &Config,
    vortex_env: Option<&str>,
    sink: ConsoleSink,
    f: F,
) -> Result<(), LoggingError>
where
    F: FnOnce(),
{
    let effective = match (console_pretty_enabled(vortex_env), sink) {
        (true, ConsoleSink::Capture(c)) => ConsoleSink::Capture(c),
        (true, ConsoleSink::Stderr | ConsoleSink::None) => ConsoleSink::Stderr,
        (false, _) => ConsoleSink::None,
    };

    let (guard, file_writer, filter) = prepare_file_logging(config)?;
    run_with_subscriber(effective, file_writer, filter, f)?;
    drop(guard);
    Ok(())
}

fn prepare_file_logging(
    config: &Config,
) -> Result<
    (
        WorkerGuard,
        tracing_appender::non_blocking::NonBlocking,
        EnvFilter,
    ),
    LoggingError,
> {
    let datos = Path::new(&config.paths.datos);
    if datos.exists() && !datos.is_dir() {
        return Err(LoggingError::new("paths.datos", "debe ser un directorio"));
    }

    let logs_dir = datos.join("logs");
    fs::create_dir_all(&logs_dir).map_err(|e| LoggingError::new("logs", e.to_string()))?;

    retain_log_files(&logs_dir, config.logs.retencion)?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("vortex.log")
        .max_log_files(config.logs.retencion as usize)
        .build(&logs_dir)
        .map_err(|e| LoggingError::new("logs", e.to_string()))?;

    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::new(config.logs.nivel.as_str());
    Ok((guard, file_writer, filter))
}

fn json_layer(
    file_writer: tracing_appender::non_blocking::NonBlocking,
    filter: EnvFilter,
) -> impl Layer<Registry> + Send + Sync {
    tracing_subscriber::fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_filter(filter)
}

#[cfg(test)]
fn run_with_subscriber<F>(
    console: ConsoleSink,
    file_writer: tracing_appender::non_blocking::NonBlocking,
    filter: EnvFilter,
    f: F,
) -> Result<(), LoggingError>
where
    F: FnOnce(),
{
    match console {
        ConsoleSink::None => {
            let subscriber = Registry::default().with(json_layer(file_writer, filter));
            tracing::subscriber::with_default(subscriber, f);
        }
        ConsoleSink::Stderr => {
            let subscriber = Registry::default()
                .with(json_layer(file_writer, filter.clone()))
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(std::io::stderr)
                        .with_filter(filter),
                );
            tracing::subscriber::with_default(subscriber, f);
        }
        ConsoleSink::Capture(writer) => {
            let subscriber = Registry::default()
                .with(json_layer(file_writer, filter.clone()))
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_ansi(false)
                        .with_writer(writer)
                        .with_filter(filter),
                );
            tracing::subscriber::with_default(subscriber, f);
        }
    }
    Ok(())
}

fn console_pretty_enabled(vortex_env: Option<&str>) -> bool {
    match vortex_env.map(str::trim) {
        Some("production") | Some("prod") => false,
        Some("development") | Some("dev") => true,
        Some(_) => false,
        None => std::io::stderr().is_terminal(),
    }
}

pub(crate) fn retain_log_files(logs_dir: &Path, keep: u32) -> Result<(), LoggingError> {
    if keep == 0 {
        return Err(LoggingError::new("logs.retencion", "debe ser al menos 1"));
    }

    let mut names: Vec<String> = Vec::new();
    let read_dir = fs::read_dir(logs_dir).map_err(|e| LoggingError::new("logs", e.to_string()))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| LoggingError::new("logs", e.to_string()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("vortex.log") {
            names.push(name);
        }
    }

    if names.len() <= keep as usize {
        return Ok(());
    }

    names.sort();
    let to_remove = names.len() - keep as usize;
    for name in names.into_iter().take(to_remove) {
        let path = logs_dir.join(name);
        fs::remove_file(&path).map_err(|e| LoggingError::new("logs", e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
fn log_files_matching(logs_dir: &Path) -> Result<Vec<PathBuf>, LoggingError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(logs_dir).map_err(|e| LoggingError::new("logs", e.to_string()))? {
        let entry = entry.map_err(|e| LoggingError::new("logs", e.to_string()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("vortex.log") {
            paths.push(entry.path());
        }
    }
    Ok(paths)
}

#[cfg(test)]
fn test_config(datos: &Path, nivel: &str, retencion: u32) -> Config {
    let toml = format!(
        r#"
[paths]
datos = "{}"
modelos = "/tmp/modelos"
backups = "/tmp/backups"
web = "/tmp/web"

[server]
bind = "127.0.0.1:8080"
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
nivel = "{nivel}"
retencion = {retencion}
"#,
        datos.display()
    );
    Config::from_str(toml.trim()).expect("config de test válida")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DIR_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_datos_dir() -> PathBuf {
        let id = DIR_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("vortex-t00-06-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).expect("temp datos");
        path
    }

    fn logs_dir_for(datos: &Path) -> PathBuf {
        datos.join("logs")
    }

    fn read_log_json_lines(logs_dir: &Path) -> Result<Vec<Value>, LoggingError> {
        let files = log_files_matching(logs_dir)?;
        let mut lines = Vec::new();
        for path in files {
            let content =
                fs::read_to_string(&path).map_err(|e| LoggingError::new("logs", e.to_string()))?;
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let value: Value = serde_json::from_str(line)
                    .map_err(|e| LoggingError::new("logs", format!("línea JSON inválida: {e}")))?;
                lines.push(value);
            }
        }
        Ok(lines)
    }

    fn json_message(value: &Value) -> Option<&str> {
        value
            .get("fields")
            .and_then(|f| f.get("message"))
            .and_then(|m| m.as_str())
            .or_else(|| value.get("message").and_then(|m| m.as_str()))
    }

    fn json_level(value: &Value) -> Option<&str> {
        value.get("level").and_then(|l| l.as_str())
    }

    fn json_has_span(value: &Value, name: &str) -> bool {
        if let Some(spans) = value.get("spans").and_then(|s| s.as_array()) {
            return spans.iter().any(|span| {
                span.get("name")
                    .and_then(|n| n.as_str())
                    .is_some_and(|n| n == name)
            });
        }
        value
            .pointer("/span/name")
            .and_then(|n| n.as_str())
            .is_some_and(|n| n == name)
    }

    #[test]
    fn logging_writes_json_line_in_temp_dir() {
        let datos = unique_datos_dir();
        let config = test_config(&datos, "info", 14);
        with_logging(&config, Some("production"), || {
            tracing::info!("mensaje json conocido");
        })
        .expect("logging init");

        let lines = read_log_json_lines(&logs_dir_for(&datos)).expect("leer logs");
        assert!(
            lines.iter().any(|v| {
                json_message(v) == Some("mensaje json conocido")
                    && json_level(v).is_some_and(|l| l.eq_ignore_ascii_case("info"))
            }),
            "expected JSON info line: {lines:?}"
        );
    }

    #[test]
    fn logging_filters_debug_when_level_is_info() {
        let datos = unique_datos_dir();
        let config = test_config(&datos, "info", 14);
        with_logging(&config, Some("production"), || {
            tracing::debug!("debug secreto");
        })
        .expect("logging init");

        let lines = read_log_json_lines(&logs_dir_for(&datos)).expect("leer logs");
        assert!(
            !lines
                .iter()
                .any(|v| json_message(v) == Some("debug secreto")),
            "debug must be filtered: {lines:?}"
        );
    }

    #[test]
    fn logging_json_includes_span_name() {
        let datos = unique_datos_dir();
        let config = test_config(&datos, "info", 14);
        with_logging(&config, Some("production"), || {
            let _span = tracing::info_span!("known_span");
            let _enter = _span.enter();
            tracing::info!("dentro del span");
        })
        .expect("logging init");

        let lines = read_log_json_lines(&logs_dir_for(&datos)).expect("leer logs");
        assert!(
            lines.iter().any(|v| {
                json_message(v) == Some("dentro del span") && json_has_span(v, "known_span")
            }),
            "expected span in JSON: {lines:?}"
        );
    }

    #[test]
    fn logging_retention_keeps_at_most_n_files() {
        let datos = unique_datos_dir();
        let logs = logs_dir_for(&datos);
        fs::create_dir_all(&logs).expect("logs dir");
        for day in ["2020-01-01", "2020-01-02", "2020-01-03", "2020-01-04"] {
            fs::write(logs.join(format!("vortex.log.{day}")), "x").expect("seed file");
        }

        let config = test_config(&datos, "info", 2);
        retain_log_files(&logs, config.logs.retencion).expect("retain");
        let count = fs::read_dir(&logs)
            .expect("read")
            .filter(|e| {
                e.as_ref()
                    .ok()
                    .map(|ent| ent.file_name().to_string_lossy().starts_with("vortex.log"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(count, 2, "retencion debe dejar 2 archivos");
    }

    #[test]
    fn logging_pretty_console_in_development() {
        let datos = unique_datos_dir();
        let config = test_config(&datos, "info", 14);
        let buf = Arc::new(Mutex::new(Vec::new()));
        with_logging_capture(&config, Some("development"), Arc::clone(&buf), || {
            tracing::info!("pretty conocido");
        })
        .expect("logging init");

        let pretty = String::from_utf8(buf.lock().unwrap().clone()).expect("utf8");
        assert!(
            pretty.contains("pretty conocido"),
            "stderr pretty debe contener el mensaje: {pretty:?}"
        );
        assert!(
            !pretty.trim_start().starts_with('{'),
            "pretty no debe ser JSON: {pretty:?}"
        );

        let lines = read_log_json_lines(&logs_dir_for(&datos)).expect("json file");
        assert!(
            lines
                .iter()
                .any(|v| json_message(v) == Some("pretty conocido"))
        );
    }

    #[test]
    fn logging_no_pretty_console_in_production() {
        let datos = unique_datos_dir();
        let config = test_config(&datos, "info", 14);
        let buf = Arc::new(Mutex::new(Vec::new()));
        with_logging_capture(&config, Some("production"), Arc::clone(&buf), || {
            tracing::info!("solo archivo");
        })
        .expect("logging init");

        let pretty = String::from_utf8(buf.lock().unwrap().clone()).expect("utf8");
        assert!(
            pretty.is_empty(),
            "production no debe escribir capa pretty: {pretty:?}"
        );

        let lines = read_log_json_lines(&logs_dir_for(&datos)).expect("json");
        assert!(
            lines
                .iter()
                .any(|v| json_message(v) == Some("solo archivo"))
        );
    }

    #[test]
    fn logging_fails_when_datos_is_a_file() {
        let parent = unique_datos_dir();
        let file_path = parent.join("not_a_dir");
        fs::write(&file_path, "x").expect("file");
        let config = test_config(&file_path, "info", 14);
        let err = with_logging(&config, Some("production"), || {}).unwrap_err();
        assert!(
            err.key == "paths.datos" || err.key == "logs",
            "expected paths.datos or logs, got {}",
            err.key
        );
        assert!(!err.reason.is_empty());
    }

    #[test]
    fn logging_fails_when_logs_dir_not_writable() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let datos = unique_datos_dir();
            let logs = logs_dir_for(&datos);
            fs::create_dir_all(&logs).expect("logs");
            let mut perms = fs::metadata(&logs).expect("meta").permissions();
            perms.set_mode(0o500);
            fs::set_permissions(&logs, perms).expect("chmod");

            let config = test_config(&datos, "info", 14);
            let err = with_logging(&config, Some("production"), || {}).unwrap_err();
            assert_eq!(err.key, "logs");
            assert!(!err.reason.is_empty());
        }
    }
}
