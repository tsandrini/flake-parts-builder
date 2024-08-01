//! Nix flakes interactive template builder based on flake-parts written
//! in Rust.
//!
//! TODO lorem ipsum
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

pub mod cmd;
pub mod config;
pub mod fs_utils;
pub mod nix;
pub mod parts;
pub mod templates;

use crate::cmd::add::{add, AddCommand};
use crate::cmd::init::{init, InitCommand};
use crate::cmd::list::{list, ListCommand};
use crate::nix::NixExecutor;

/// Nix flakes interactive template builder based on flake-parts written
/// in Rust.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None, verbatim_doc_comment)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init(InitCommand),
    List(ListCommand),
    Add(AddCommand),
}

// TODO add logging
// TODO better docs
fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();
    log::debug!("color-eyre installed and logger initialized");

    let cli = Cli::parse();
    let nix_cmd = NixExecutor::from_env()?;

    match cli.command {
        Commands::List(cmd) => {
            log::info!("Executing list command");
            list(cmd, nix_cmd)
        }
        Commands::Init(cmd) => {
            log::info!("Executing init command");
            init(cmd, nix_cmd)
        }
        Commands::Add(cmd) => {
            log::info!("Executing add command");
            add(cmd, nix_cmd)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
