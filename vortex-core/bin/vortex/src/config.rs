//! Vortex Core configuration (T-00-05).
#![allow(dead_code)] // `from_path` se engancha en T-00-10; el módulo se ejerce desde tests unitarios.

use serde::Deserialize;
use std::net::SocketAddr;
use std::path::Path;
use thiserror::Error;

const SECTIONS: &[&str] = &[
    "paths",
    "server",
    "negocio",
    "meilisearch",
    "ollama",
    "backup",
    "logs",
];

#[derive(Debug, Error)]
#[error("config: {key}: {reason}")]
pub struct ConfigError {
    pub key: String,
    pub reason: String,
}

impl ConfigError {
    pub fn new(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub paths: Paths,
    pub server: Server,
    pub negocio: Negocio,
    pub meilisearch: Meilisearch,
    pub ollama: Ollama,
    pub backup: Backup,
    pub logs: Logs,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paths {
    pub datos: String,
    pub modelos: String,
    pub backups: String,
    pub web: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub bind: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Negocio {
    pub moneda: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meilisearch {
    pub url: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ollama {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backup {
    pub hora: String,
    pub retencion: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Logs {
    pub nivel: String,
    pub retencion: u32,
}

impl Config {
    pub fn from_str(contents: &str) -> Result<Self, ConfigError> {
        reject_unknown_sections(contents)?;
        let config: Config = toml::from_str(contents).map_err(map_toml_deserialize_error)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_path(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::new(path.display().to_string(), e.to_string()))?;
        Self::from_str(&contents)
    }
}

fn map_toml_deserialize_error(e: toml::de::Error) -> ConfigError {
    let msg = e.to_string();
    if let Some(field) = extract_backtick_field(&msg) {
        if msg.contains("unknown field") {
            return ConfigError::new(field, "clave desconocida");
        }
        if msg.contains("missing field") {
            return ConfigError::new(field, "clave requerida");
        }
    }
    ConfigError::new("toml", msg)
}

fn extract_backtick_field(message: &str) -> Option<String> {
    let start = message.find('`')? + 1;
    let rest = &message[start..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

fn reject_unknown_sections(contents: &str) -> Result<(), ConfigError> {
    let table: toml::Table =
        toml::from_str(contents).map_err(|e| ConfigError::new("toml", e.to_string()))?;
    for key in table.keys() {
        if !SECTIONS.contains(&key.as_str()) {
            return Err(ConfigError::new(key.clone(), "sección desconocida"));
        }
    }
    Ok(())
}

impl Config {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_non_empty("paths.datos", &self.paths.datos)?;
        validate_non_empty("paths.modelos", &self.paths.modelos)?;
        validate_non_empty("paths.backups", &self.paths.backups)?;
        validate_non_empty("paths.web", &self.paths.web)?;

        self.server.bind.parse::<SocketAddr>().map_err(|_| {
            ConfigError::new("server.bind", "debe ser una dirección host:puerto válida")
        })?;
        validate_non_empty("server.hostname", &self.server.hostname)?;

        validate_moneda("negocio.moneda", &self.negocio.moneda)?;
        validate_http_url("meilisearch.url", &self.meilisearch.url)?;
        validate_http_url("ollama.url", &self.ollama.url)?;
        validate_hora("backup.hora", &self.backup.hora)?;
        validate_retencion("backup.retencion", self.backup.retencion)?;
        validate_log_level("logs.nivel", &self.logs.nivel)?;
        validate_retencion("logs.retencion", self.logs.retencion)?;
        Ok(())
    }
}

fn validate_non_empty(key: &str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        return Err(ConfigError::new(key, "no puede estar vacío"));
    }
    Ok(())
}

fn validate_moneda(key: &str, value: &str) -> Result<(), ConfigError> {
    let ok = value.len() == 3
        && value
            .chars()
            .all(|c| c.is_ascii_uppercase() && c.is_ascii_alphabetic());
    if !ok {
        return Err(ConfigError::new(
            key,
            "debe ser un código ISO 4217 de tres letras mayúsculas",
        ));
    }
    Ok(())
}

fn validate_http_url(key: &str, value: &str) -> Result<(), ConfigError> {
    if !(value.starts_with("http://") || value.starts_with("https://")) {
        return Err(ConfigError::new(
            key,
            "debe comenzar con http:// o https://",
        ));
    }
    Ok(())
}

fn validate_hora(key: &str, value: &str) -> Result<(), ConfigError> {
    let Some((h, m)) = value.split_once(':') else {
        return Err(ConfigError::new(key, "debe tener el formato HH:MM"));
    };
    if h.len() != 2 || m.len() != 2 {
        return Err(ConfigError::new(key, "debe tener el formato HH:MM"));
    }
    let hour: u32 = h
        .parse()
        .map_err(|_| ConfigError::new(key, "hora inválida"))?;
    let minute: u32 = m
        .parse()
        .map_err(|_| ConfigError::new(key, "minutos inválidos"))?;
    if hour > 23 || minute > 59 {
        return Err(ConfigError::new(key, "debe estar entre 00:00 y 23:59"));
    }
    Ok(())
}

fn validate_retencion(key: &str, value: u32) -> Result<(), ConfigError> {
    if value < 1 {
        return Err(ConfigError::new(key, "debe ser al menos 1"));
    }
    Ok(())
}

fn validate_log_level(key: &str, value: &str) -> Result<(), ConfigError> {
    if matches!(value, "trace" | "debug" | "info" | "warn" | "error") {
        Ok(())
    } else {
        Err(ConfigError::new(
            key,
            "debe ser trace, debug, info, warn o error",
        ))
    }
}

#[cfg(test)]
fn example_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config.example.toml")
}

#[cfg(test)]
fn minimal_valid_toml() -> String {
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
    .trim()
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_parses_example_toml() {
        let path = example_config_path();
        Config::from_path(&path).expect("config.example.toml debe ser válido");
    }

    #[test]
    fn config_allows_omitted_meilisearch_api_key() {
        let toml = minimal_valid_toml();
        Config::from_str(&toml).expect("sin api_key debe ser válido");
    }

    #[test]
    fn config_rejects_missing_required_key() {
        let toml = minimal_valid_toml().replace("datos = \"/tmp/datos\"\n", "");
        let err = Config::from_str(&toml).unwrap_err();
        assert!(
            err.key.contains("datos") || err.reason.contains("datos"),
            "expected datos in error: {err}"
        );

        let toml = minimal_valid_toml().replace("[negocio]\nmoneda = \"ARS\"\n", "");
        let err = Config::from_str(&toml).unwrap_err();
        assert!(
            err.key.contains("negocio") || err.reason.contains("negocio"),
            "expected negocio in error: {err}"
        );
    }

    #[test]
    fn config_rejects_invalid_values() {
        let cases: Vec<(&str, &str, &str)> = vec![
            ("bind", "bind = \"127.0.0.1\"", "server.bind"),
            ("moneda", "moneda = \"peso\"", "negocio.moneda"),
            ("hora", "hora = \"25:00\"", "backup.hora"),
            ("backup retencion", "retencion = 0", "backup.retencion"),
            ("nivel", "nivel = \"verbose\"", "logs.nivel"),
            ("meili url", "url = \"127.0.0.1:7700\"", "meilisearch.url"),
        ];

        for (label, replacement_line, expected_key) in cases {
            let mut toml = minimal_valid_toml();
            if label == "bind" {
                toml = toml.replace("bind = \"127.0.0.1:8080\"", replacement_line);
            } else if label == "moneda" {
                toml = toml.replace("moneda = \"ARS\"", replacement_line);
            } else if label == "hora" {
                toml = toml.replace("hora = \"03:00\"", replacement_line);
            } else if label == "backup retencion" {
                toml = toml.replace(
                    "[backup]\nhora = \"03:00\"\nretencion = 14",
                    &format!("[backup]\nhora = \"03:00\"\n{replacement_line}"),
                );
            } else if label == "nivel" {
                toml = toml.replace("nivel = \"info\"", replacement_line);
            } else if label == "meili url" {
                toml = toml.replace("url = \"http://127.0.0.1:7700\"", replacement_line);
            }
            let err = Config::from_str(&toml).unwrap_err();
            assert!(
                err.key.contains(expected_key)
                    || err
                        .key
                        .contains(expected_key.split('.').next_back().unwrap_or(expected_key)),
                "{label}: expected key {expected_key}, got {err}"
            );
        }

        let mut toml = minimal_valid_toml();
        toml = toml.replace(
            "[logs]\nnivel = \"info\"\nretencion = 14",
            "[logs]\nnivel = \"info\"\nretencion = 0",
        );
        let err = Config::from_str(&toml).unwrap_err();
        assert!(err.key.contains("logs.retencion"));
    }

    #[test]
    fn config_rejects_unknown_key() {
        let toml = minimal_valid_toml().replace(
            "hostname = \"localhost\"",
            "hostname = \"localhost\"\nport = 8080",
        );
        let err = Config::from_str(&toml).unwrap_err();
        assert_eq!(err.key, "port");

        let toml = format!("{}\n[foo]\nbar = 1\n", minimal_valid_toml());
        let err = Config::from_str(&toml).unwrap_err();
        assert_eq!(err.key, "foo");
    }

    #[test]
    fn config_rejects_malformed_toml() {
        let err = Config::from_str("[paths").unwrap_err();
        assert_eq!(err.key, "toml");
    }
}
