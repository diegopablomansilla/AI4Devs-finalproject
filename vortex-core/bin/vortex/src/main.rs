//! Vortex Core binary.

mod boot;
mod cli;
mod config;
mod logging;
mod seed;
mod serve;

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse_or_exit();
    match cli.command {
        Commands::Serve => serve::run(cli.config.as_ref()),
        Commands::Seed { dir } => seed::run(cli.config.as_ref(), dir),
        _ => cli.run_stub(),
    }
}
