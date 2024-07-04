use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;
mod config;
mod parts;

use crate::cmd::init::{init, InitCommand};
use crate::cmd::list::{list, ListCommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Init a new flake project
    Init(InitCommand),

    /// List all available flake-parts
    List(ListCommand),

    // TODO
    Add,
}

// TODO add logging
// TODO add tests
// TODO better docs
// TODO constructors?
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(cmd) => init(cmd),
        Commands::List(cmd) => list(cmd),
        Commands::Add => todo!("Add command not implemented yet"), // TODO
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
