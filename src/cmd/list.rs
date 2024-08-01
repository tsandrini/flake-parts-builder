use clap::Args;
use color_eyre::eyre::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::cmd::SharedArgs;
use crate::config::{BASE_DERIVATION_NAME, BOOTSTRAP_DERIVATION_NAME, SELF_FLAKE_URI};
use crate::nix::NixCmdInterface;
use crate::parts::FlakePartsStore;

/// List all available flake-parts in all parts stores provided by the user.
#[derive(Debug, Args)]
pub struct ListCommand {
    #[clap(flatten)]
    pub shared_args: SharedArgs,
}

pub fn list(mut cmd: ListCommand, nix_cmd: impl NixCmdInterface) -> Result<()> {
    if !cmd.shared_args.disable_base_parts {
        cmd.shared_args
            .parts_stores
            .push(format!("{}#{}", SELF_FLAKE_URI, BASE_DERIVATION_NAME));
    }

    // NOTE this one is required even if you disable base store parts
    cmd.shared_args
        .parts_stores
        .push(format!("{}#{}", SELF_FLAKE_URI, BOOTSTRAP_DERIVATION_NAME));

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    cmd.shared_args
        .parts_stores
        .iter()
        .try_for_each(|flake_uri| {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            writeln!(&mut stdout, " # {}", flake_uri)?;

            // TODO maybe some error message instead of unwrap?
            FlakePartsStore::from_flake_uri(&flake_uri, &nix_cmd)
                .unwrap()
                .parts
                .iter()
                .try_for_each(|part| {
                    // Visually distinguish collections
                    let color = if part.name.contains('+') {
                        Color::Cyan
                    } else {
                        Color::Red
                    };

                    stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;

                    write!(&mut stdout, "  - {}: ", part.name)?;

                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;

                    writeln!(&mut stdout, "{}", part.metadata.description)?;

                    Ok(()) as Result<()>
                })?;

            println!("");
            Ok(())
        })
}
