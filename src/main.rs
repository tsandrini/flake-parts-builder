use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use minijinja::{context, Environment};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod parts;
use parts::*;

static FLAKE_TEMPLATE: &'static str = include_str!("assets/flake.nix.template");
// static SELF_FLAKE_URI: &'static str = "github:tsandrini/flake-parts-builder";
static SELF_FLAKE_URI: &'static str = "."; // TODO only for dev

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
    #[arg(short = 'S', long = "stores")]
    parts_stores: Vec<String>,

    /// Disable base parts provided by this flake
    #[arg(long = "disable-base-parts", default_value_t = false)]
    disable_base_parts: bool,
}

#[derive(Debug, Args)]
struct ListCommand {
    /// Additional parts templates stores to load
    #[arg(short = 'S', long = "stores")]
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
    // TODO add a command to add a new part
}

fn init(mut cmd: InitCommand) -> Result<()> {
    // let target = tempdir()?;

    if !cmd.disable_base_parts {
        cmd.parts_stores
            .push(format!("{}#flake-parts", SELF_FLAKE_URI));
    }

    // NOTE this one is required even if you disable this parts store
    cmd.parts
        .push(format!("{}#flake-parts/_base", SELF_FLAKE_URI));

    let parts_flake_uri = cmd
        .parts
        .into_iter()
        .map(|part| {
            if part.contains('#') {
                part
            } else {
                format!("{}#flake-parts/{}", SELF_FLAKE_URI, part)
            }
        })
        .collect::<Vec<_>>();

    println!("parts flake uri: {:?}", parts_flake_uri);

    let out_parts = cmd
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(&store))
        .collect::<Result<Vec<FlakePartsStore>, FlakePartsStoreParseError>>()?
        .into_iter()
        .map(|store| store.parts)
        .flatten()
        .filter(|part| parts_flake_uri.contains(&part.flake_uri))
        .collect::<Vec<FlakePart>>();

    println!("out parts: {:?}", out_parts);

    // NOTE
    // 1. convert cmd.parts to actual parts
    // 2. start iterating over actual parts and building
    //    a. create a tmpdir
    //    b.
    println!("{:?}", cmd.path.file_name());

    let mut env = Environment::new();
    env.add_template("flake.nix", &FLAKE_TEMPLATE).unwrap();
    let tmpl = env.get_template("flake.nix").unwrap();
    println!(
        "{}",
        tmpl.render(context!(
            context => FlakeContext {
                inputs: Default::default(),
                extra_trusted_public_keys: vec![String::from("haha"), String::from("hehe")],
                extra_substituters: Default::default(),
            }
        ))
        .unwrap()
    );

    // let rendered = tt.render("flake.nix", &context)?;
    // println!("{}", rendered);

    Ok(())
}

fn list(mut cmd: ListCommand) -> Result<()> {
    if !cmd.disable_base_parts {
        cmd.parts_stores
            .push(format!("{}#flake-parts", SELF_FLAKE_URI));
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    cmd.parts_stores.iter().try_for_each(|flake_uri| {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        writeln!(&mut stdout, " # {}", flake_uri)?;

        FlakePartsStore::from_flake_uri(&flake_uri)
            .unwrap()
            .parts
            .iter()
            .try_for_each(|part| {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;

                write!(&mut stdout, "  - {}: ", part.short_name)?;

                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;

                writeln!(&mut stdout, "{}", part.metadata.description)?;

                Ok(()) as Result<()>
            })?;

        println!("");
        Ok(())
    })
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
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
