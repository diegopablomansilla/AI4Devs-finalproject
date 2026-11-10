use std::path::PathBuf;
use std::process::Command;

fn vortex_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vortex"))
}

fn example_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config.example.toml")
}

fn write_invalid_config() -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("vortex-invalid-config-{}.toml", std::process::id()));
    std::fs::write(
        &path,
        r#"
[paths]
datos = "/tmp/datos"
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
    )
    .expect("write temp config");
    path
}

#[test]
fn vortex_exits_nonzero_without_path() {
    let output = Command::new(vortex_bin()).output().expect("spawn vortex");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--config") || stderr.contains("VORTEX_CONFIG"),
        "stderr: {stderr}"
    );
}

#[test]
fn vortex_exits_nonzero_missing_file() {
    let output = Command::new(vortex_bin())
        .args(["--config", "/nonexistent/vortex-config.toml"])
        .output()
        .expect("spawn vortex");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("vortex-config.toml") || stderr.contains("nonexistent"),
        "stderr: {stderr}"
    );
}

#[test]
fn vortex_exits_nonzero_invalid_config() {
    let path = write_invalid_config();
    let output = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap()])
        .output()
        .expect("spawn vortex");
    let _ = std::fs::remove_file(&path);
    assert!(!output.status.success(), "expected failure");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("negocio.moneda"),
        "stderr must name key: {stderr}"
    );
    assert!(!stderr.is_empty(), "stderr must include reason");
}

#[test]
fn vortex_exits_zero_with_config_flag() {
    let path = example_config();
    let output = Command::new(vortex_bin())
        .args(["--config", path.to_str().unwrap()])
        .output()
        .expect("spawn vortex");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn vortex_exits_zero_with_env_path() {
    let path = example_config();
    let output = Command::new(vortex_bin())
        .env("VORTEX_CONFIG", &path)
        .output()
        .expect("spawn vortex");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn vortex_config_flag_overrides_env() {
    let valid = example_config();
    let invalid = write_invalid_config();
    let output = Command::new(vortex_bin())
        .env("VORTEX_CONFIG", &invalid)
        .args(["--config", valid.to_str().unwrap()])
        .output()
        .expect("spawn vortex");
    let _ = std::fs::remove_file(&invalid);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
