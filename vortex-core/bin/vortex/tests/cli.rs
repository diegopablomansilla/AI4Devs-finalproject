use std::path::PathBuf;
use std::process::Command;

const TOP_LEVEL_SUBCOMMANDS: &[&str] = &[
    "serve", "reindex", "backup", "cert", "models", "seed", "verify", "service",
];

fn vortex_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vortex"))
}

fn combined_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{stdout}{stderr}")
}

#[test]
fn vortex_help_lists_all_subcommands() {
    let output = Command::new(vortex_bin())
        .arg("--help")
        .output()
        .expect("spawn vortex");
    assert!(
        output.status.success(),
        "stderr: {}",
        combined_output(&output)
    );
    let text = combined_output(&output);
    for name in TOP_LEVEL_SUBCOMMANDS {
        assert!(text.contains(name), "help must list {name}: {text}");
    }
}

#[test]
fn vortex_short_help_lists_subcommands() {
    let output = Command::new(vortex_bin())
        .arg("-h")
        .output()
        .expect("spawn vortex");
    assert!(
        output.status.success(),
        "stderr: {}",
        combined_output(&output)
    );
    let text = combined_output(&output);
    for name in TOP_LEVEL_SUBCOMMANDS {
        assert!(text.contains(name), "help must list {name}: {text}");
    }
}

#[test]
fn vortex_service_help_lists_install_uninstall() {
    let output = Command::new(vortex_bin())
        .args(["service", "--help"])
        .output()
        .expect("spawn vortex");
    assert!(
        output.status.success(),
        "stderr: {}",
        combined_output(&output)
    );
    let text = combined_output(&output);
    assert!(text.contains("install"), "text: {text}");
    assert!(text.contains("uninstall"), "text: {text}");
}

#[test]
fn vortex_help_mentions_config_flag() {
    let output = Command::new(vortex_bin())
        .arg("--help")
        .output()
        .expect("spawn vortex");
    let text = combined_output(&output);
    assert!(text.contains("--config"), "text: {text}");
}

#[test]
fn vortex_serve_unimplemented_ignores_missing_config() {
    let output = Command::new(vortex_bin())
        .args(["--config", "/nonexistent/vortex-config.toml", "serve"])
        .output()
        .expect("spawn vortex");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no implementado"), "stderr: {stderr}");
}

#[test]
fn vortex_serve_prints_unimplemented() {
    let output = Command::new(vortex_bin())
        .arg("serve")
        .output()
        .expect("spawn vortex");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no implementado"), "stderr: {stderr}");
}

#[test]
fn vortex_top_level_stubs_print_unimplemented() {
    for sub in ["reindex", "backup", "cert", "models", "seed", "verify"] {
        let output = Command::new(vortex_bin())
            .arg(sub)
            .output()
            .expect("spawn vortex");
        assert_eq!(output.status.code(), Some(1), "subcommand {sub}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("no implementado"),
            "subcommand {sub}: stderr: {stderr}"
        );
    }
}

#[test]
fn vortex_service_install_uninstall_print_unimplemented() {
    for args in [["service", "install"], ["service", "uninstall"]] {
        let output = Command::new(vortex_bin())
            .args(args)
            .output()
            .expect("spawn vortex");
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("no implementado"),
            "args {:?}: stderr: {stderr}",
            args
        );
    }
}

#[test]
fn vortex_without_subcommand_is_error() {
    let output = Command::new(vortex_bin()).output().expect("spawn vortex");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
}

#[test]
fn vortex_unknown_subcommand_is_error() {
    let output = Command::new(vortex_bin())
        .arg("definitely-not-a-command")
        .output()
        .expect("spawn vortex");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
}

#[test]
fn vortex_service_without_action_is_error() {
    let output = Command::new(vortex_bin())
        .arg("service")
        .output()
        .expect("spawn vortex");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("no implementado"), "stderr: {stderr}");
}
