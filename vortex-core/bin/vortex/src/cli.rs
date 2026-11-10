//! CLI entrypoint for Vortex Core (T-00-07).

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "vortex", about = "Vortex Core")]
pub struct Cli {
    /// Ruta del archivo de configuración TOML
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Arranca el servicio HTTP
    Serve,
    /// Reindexa índices de búsqueda
    Reindex,
    /// Ejecuta un backup
    Backup,
    /// Gestiona certificados TLS locales
    Cert,
    /// Descarga modelos y binarios de sidecars
    Models,
    /// Carga datos semilla en el catálogo
    Seed,
    /// Verifica la consistencia del sistema
    Verify,
    /// Instalación del servicio del sistema
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Registra el servicio en launchd
    Install,
    /// Quita el servicio del sistema
    Uninstall,
}

impl Cli {
    pub fn parse_or_exit() -> Self {
        match Self::try_parse() {
            Ok(cli) => cli,
            Err(err) => err.exit(),
        }
    }

    pub fn run_stub(self) -> ! {
        let _ = self.config;
        let _ = self.command;
        eprintln!("no implementado");
        process::exit(1);
    }
}
