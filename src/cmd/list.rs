use clap::Args;
use color_eyre::eyre::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::config::SELF_FLAKE_URI;
use crate::parts::FlakePartsStore;

#[derive(Debug, Args)]
pub struct ListCommand {
    /// Additional parts templates stores to load
    #[arg(short = 'S', long = "stores")]
    parts_stores: Vec<String>,

    /// Disable base parts provided by this flake
    #[arg(long = "no-base", default_value_t = false)]
    disable_base_parts: bool,
}

pub fn list(mut cmd: ListCommand) -> Result<()> {
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

                write!(&mut stdout, "  - {}: ", part.name)?;

                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;

                writeln!(&mut stdout, "{}", part.metadata.description)?;

                Ok(()) as Result<()>
            })?;

        println!("");
        Ok(())
    })
}
