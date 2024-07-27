use clap::Args;
use color_eyre::eyre::Result;
use fs_extra::dir::{self, CopyOptions};
use tempfile::tempdir;

use crate::cmd::init::{parse_required_parts_tuples, prepare_tmpdir, InitCommand};
use crate::config::{BASE_DERIVATION_NAME, SELF_FLAKE_URI};
use crate::parts::FlakePartsStore;
use crate::templates::FlakeInputsContext;

// TODO for some reason broken formatting
/// Add additional flake-parts to an already initialized project. This is
/// similar to the init command, but differs in two significant ways:
///
/// 1. No `_bootstrap` part is added, as the project is already bootstrapped.
/// 2. `flake.nix` is left untouched as the user may have made manual changes.
///    Additional inputs will be printed to the console and the user is
///    advised to add them manually.
///
///    TODO: I might figure out a way to do this automatically in the future, but
///    it's way too complicated for now.
#[derive(Debug, Args)]
pub struct AddCommand {
    #[clap(flatten)]
    pub init: InitCommand,
}

pub fn add(mut cmd: AddCommand) -> Result<()> {
    if !cmd.init.disable_base_parts {
        cmd.init
            .parts_stores
            .push(format!("{}#{}", SELF_FLAKE_URI, BASE_DERIVATION_NAME));
    }

    // NOTE we init stores here to have sensible ownerships of FlakePartTuples
    let stores = cmd
        .init
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(&store))
        .collect::<Result<Vec<_>>>()?;

    let parts_tuples = parse_required_parts_tuples(&cmd.init, &stores)?;

    let path = cmd
        .init
        .path
        .canonicalize()
        .unwrap_or_else(|_| cmd.init.path.clone());

    // TODO probably yield an error instead
    if !path.exists() {
        dir::create_all(&path, false)?;
    }

    let tmpdir = tempdir()?;
    prepare_tmpdir(
        &tmpdir,
        &parts_tuples,
        path.to_str(),
        &cmd.init.strategy,
        false,
    )?;

    // NOTE the flake.nix file shouldn't be present due to the strucutre of
    // flake-parts, but I am way tooo paranoid.
    if tmpdir.path().join("flake.nix").exists() {
        std::fs::remove_file(tmpdir.path().join("flake.nix"))?;
    }

    let metadata = parts_tuples
        .iter()
        .map(|part_tuple| &part_tuple.part.metadata)
        .collect::<Vec<_>>();

    let flake_context = FlakeInputsContext::from_merged_metadata(&metadata);

    let rendered = flake_context.render()?;
    println!("Please add the following snippet to your `flake.nix` inputs:");
    println!("{}", rendered);

    dir::copy(
        &tmpdir,
        &cmd.init.path,
        &CopyOptions::new()
            .content_only(true)
            .skip_exist(!cmd.init.force)
            .overwrite(cmd.init.force),
    )?;

    Ok(())
}
