use std::path::PathBuf;
use std::process::Command;

fn vortex_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vortex"))
}

fn example_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config.example.toml")
}

#[test]
fn vortex_without_subcommand_does_not_load_config() {
    let output = Command::new(vortex_bin()).output().expect("spawn vortex");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("no implementado"),
        "missing subcommand is not a stub: {stderr}"
    );
}

#[test]
fn vortex_backup_with_missing_config_still_stub() {
    let output = Command::new(vortex_bin())
        .args(["--config", "/nonexistent/vortex-config.toml", "backup"])
        .output()
        .expect("spawn vortex");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no implementado"),
        "stub must not validate config: {stderr}"
    );
}
