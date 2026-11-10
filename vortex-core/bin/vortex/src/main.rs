//! Vortex Core binary.

mod config;

use config::ConfigError;
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run() -> Result<(), ConfigError> {
    let path = resolve_config_path()?;
    config::Config::from_path(&path)?;
    Ok(())
}

fn resolve_config_path() -> Result<PathBuf, ConfigError> {
    let from_flag = parse_config_flag(env::args());
    if let Some(path) = from_flag {
        return Ok(path);
    }
    if let Ok(path) = env::var("VORTEX_CONFIG") {
        if path.is_empty() {
            return Err(ConfigError::new(
                "VORTEX_CONFIG",
                "no se especificó ruta de configuración",
            ));
        }
        return Ok(PathBuf::from(path));
    }
    Err(ConfigError::new(
        "--config",
        "no se especificó ruta de configuración (use --config o VORTEX_CONFIG)",
    ))
}

fn parse_config_flag<I>(args: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = String>,
{
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--config" {
            let path = iter.next().filter(|p| !p.is_empty())?;
            return Some(PathBuf::from(path));
        }
        if let Some(rest) = arg.strip_prefix("--config=") {
            if rest.is_empty() {
                continue;
            }
            return Some(PathBuf::from(rest));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_flag_accepts_equals_form() {
        let path = parse_config_flag([
            "vortex".to_string(),
            "--config=/etc/vortex.toml".to_string(),
        ]);
        assert_eq!(path, Some(PathBuf::from("/etc/vortex.toml")));
    }
}
