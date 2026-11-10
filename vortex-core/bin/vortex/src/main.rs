//! Vortex Core binary.

mod cli;
mod config;
mod logging;

fn main() {
    cli::Cli::parse_or_exit().run_stub();
}
