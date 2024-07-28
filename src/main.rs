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
// TODO add tests
// TODO better docs
fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(cmd) => init(cmd),
        Commands::List(cmd) => list(cmd),
        Commands::Add(cmd) => add(cmd),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
