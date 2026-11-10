//! Vortex Core binary.

mod cli;
mod config;
mod logging;
mod serve;

use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse_or_exit();
    if matches!(cli.command, Commands::Serve) {
        serve::run(cli.config.as_ref());
    } else {
        cli.run_stub();
    }
}
