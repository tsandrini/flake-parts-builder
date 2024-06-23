use serde::{Deserialize, Serialize};

use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use std::{error::Error, io};

use std::process::Command;
use tempfile::tempdir;
use tinytemplate::TinyTemplate;

use clap::{Args, Parser, Subcommand};

static FLAKE_TEMPLATE: &'static str = include_str!("assets/flake.nix.template");
static SELF_FLAKE_URI: &'static str = "github:tsandrini/flake-parts-builder";

#[derive(Serialize)]
struct Context {
    inputs: String,
    extraTrustedPublicKeys: String,
    extraSubstituters: String,
}

#[derive(Debug)]
struct FlakePartsStore {
    flake_uri: String,
    nix_store_path: PathBuf,
    parts: Vec<FlakePart>,
}

#[derive(Debug)]
struct FlakePart {
    flake_uri: String,
    short_name: String,
    nix_store_path: PathBuf,
    metadata: FlakePartMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct FlakePartMetadata {
    description: String,
    inputs: JsonValue,
    dependencies: Vec<String>,
    conflicts: Vec<String>,
    extraTrustedPublicKeys: Vec<String>,
    extraSubstituters: Vec<String>,
}

impl FlakePart {
    fn from_path(path: PathBuf, parent_store_flake_uri: &String) -> Result<Self, Box<dyn Error>> {
        let name = path.file_name().unwrap().to_str().unwrap();

        let nix_eval = Command::new("nix")
            .args([
                "eval",
                "--expr",
                "let pkgs = import <nixpkgs> {}; in import ./meta.nix { inherit pkgs; }",
                "--impure",
                "--json",
            ])
            .current_dir(&path)
            .output()?;
        // use std::io::{self, Write};
        // io::stdout().write_all(&nix_eval.stdout).unwrap();
        // io::stderr().write_all(&nix_eval.stderr).unwrap();

        let output = String::from_utf8(nix_eval.stdout)?;
        let metadata: FlakePartMetadata = serde_json::from_str(&output)?;

        Ok(Self {
            flake_uri: format!("{}/{}", parent_store_flake_uri, name),
            short_name: name.to_string(),
            nix_store_path: path,
            metadata: metadata,
        })
    }
}

impl FlakePartsStore {
    fn from_flake_uri(flake_uri: String) -> Result<Self, Box<dyn Error>> {
        let nix_info = Command::new("nix")
            .args(["build", "--no-link", "--print-out-paths", &flake_uri])
            .output()?;

        let output = String::from_utf8(nix_info.stdout)?;

        let parts = fs::read_dir(format!("{}/parts", output.trim()))?
            .map(|entry| {
                let entry = entry?;
                let path = entry.path();
                let part_info = FlakePart::from_path(path, &flake_uri)?;
                println!("Part info: {:?}", part_info);
                Ok(part_info)
            })
            .collect::<Result<Vec<FlakePart>, Box<dyn Error>>>()?;

        Ok(Self {
            nix_store_path: PathBuf::from(&output.trim()),
            flake_uri: flake_uri,
            parts: parts,
        })
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}
#[derive(Debug, Args)]
struct InitCommand {
    /// Path for the desired flake project
    path: PathBuf,

    /// Desired parts to be used
    #[arg(short = 'P', long, required = true)]
    parts: Vec<String>,

    /// Additional parts templates stores to load
    #[arg(short = 'S', long)]
    parts_stores: Vec<String>,

    /// Disable base parts provided by this flake
    #[arg(long = "no-base", default_value_t = false)]
    disable_base_parts: bool,
}

#[derive(Debug, Args)]
struct ListCommand {
    /// Additional parts templates stores to load
    #[arg(short = 'S', long)]
    parts_stores: Vec<String>,

    /// Disable base parts provided by this flake
    #[arg(long = "no-base", default_value_t = false)]
    disable_base_parts: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Init a new flake project
    Init(InitCommand),

    /// List all available flake-parts
    List(ListCommand),
}

fn parse_parts_stores(cmd: &InitCommand) -> Result<Vec<FlakePartsStore>, Box<dyn Error>> {
    cmd.parts_stores
        .iter()
        .try_fold(Vec::new(), |mut stores, flake_uri| {
            // let store = FlakePartsStore::from_flake_uri(flake_uri.clone())
            //     .map_err(|_| Box::new(std::io::Error::new(
            //         std::io::ErrorKind::InvalidData,
            //         format!("Provided parts store flake_uri {} doesn't\
            //          correspond to any valid flakes enabled derivation. Please make\
            //          sure you're using a valid flake URI with a derivation that has parts stored at $out/parts.", flake_uri)
            //     )) as Box<dyn Error>)?;
            let store = FlakePartsStore::from_flake_uri(flake_uri.clone())?;

            println!("Parsed store: {:?}", store);
            println!(
                "Parsed store path: {:?}",
                store.nix_store_path.canonicalize()
            );
            stores.push(store);
            Ok(stores)
        })
}

fn init(mut cmd: InitCommand) -> Result<(), Box<dyn Error>> {
    let target = tempdir()?;

    if !cmd.disable_base_parts {
        println!("Enabling default parts");
        cmd.parts_stores.push(format!("{}#parts", SELF_FLAKE_URI));
    }

    let stores = parse_parts_stores(&cmd)?;

    println!("{:?}", cmd.path.file_name());

    let mut tt = TinyTemplate::new();
    tt.add_template("flake.nix", FLAKE_TEMPLATE)?;

    let context = Context {
        inputs: "".to_string(),
        extraTrustedPublicKeys: "".to_string(),
        extraSubstituters: "".to_string(),
    };
    // let rendered = tt.render("flake.nix", &context)?;
    // println!("{}", rendered);

    Ok(())
}

fn list(cmd: ListCommand) -> Result<(), Box<dyn Error>> {
    Ok(())
}

// TODO add logging
// TODO add tests
// TODO better docs
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(cmd) => init(cmd),
        Commands::List(cmd) => list(cmd),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
