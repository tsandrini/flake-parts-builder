use clap::Args;
use color_eyre::eyre::Result;
use fs_extra::dir::{self, CopyOptions};
use tempfile::tempdir;

use crate::cmd::init::{parse_required_parts_tuples, prepare_tmpdir, InitCommand};
use crate::config::{BASE_DERIVATION_NAME, SELF_FLAKE_URI};
use crate::nix::NixCmdInterface;
use crate::parts::FlakePartsStore;
use crate::templates::FlakeInputsContext;

//    TODO: I might figure out a way to do this automatically in the future, but
//    it's way too complicated for now.
// TODO for some reason broken formatting
/// Add additional flake-parts to an already initialized project.
///
/// This is similar to the init command, but differs in two significant ways:
///
/// 1. No `_bootstrap` part is added, as the project is already bootstrapped.
///
/// 2. `flake.nix` is left untouched as the user may have already made manual changes.
///    Additional inputs will be printed to the console and the user is
///    advised to add them manually.
#[derive(Debug, Args)]
pub struct AddCommand {
    #[clap(flatten)]
    pub init: InitCommand,
}

pub fn add(mut cmd: AddCommand, nix_cmd: impl NixCmdInterface) -> Result<()> {
    if !cmd.init.shared_args.disable_base_parts {
        log::info!("Adding base parts store to `cmd.shared_args.parts_stores`");

        cmd.init
            .shared_args
            .parts_stores
            .push(format!("{}#{}", SELF_FLAKE_URI, BASE_DERIVATION_NAME));
    }

    // NOTE we init stores here to have sensible ownerships of FlakePartTuples
    let stores = cmd
        .init
        .shared_args
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(store, &nix_cmd))
        .collect::<Result<Vec<_>>>()?;

    log::debug!(
        "All parts stores: {:?}",
        stores
            .iter()
            .map(|store| store.flake_uri.clone())
            .collect::<Vec<_>>()
    );

    let parts_tuples = parse_required_parts_tuples(&cmd.init, &stores)?;

    let path = cmd
        .init
        .path
        .canonicalize()
        .unwrap_or_else(|_| cmd.init.path.clone());

    log::debug!("Full user provided path: {:?}", path);

    if !path.exists() {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path {:?} does not exist", path),
        ))?
    }

    let tmpdir = tempdir()?;
    log::info!("Preparing new additions in a tmpdir at {:?}", tmpdir.path());
    prepare_tmpdir(
        &nix_cmd,
        &tmpdir,
        &parts_tuples,
        path.file_name().map(|osstr| osstr.to_str().unwrap()),
        &cmd.init.strategy,
        false,
        cmd.init.shared_args.write_meta
    )?;

    // NOTE the flake.nix file shouldn't be present due to the strucutre of
    // flake-parts, but I am way tooo paranoid.
    if tmpdir.path().join("flake.nix").exists() {
        log::warn!("Unexpected flake.nix file found in tmpdir, removing it.");
        std::fs::remove_file(tmpdir.path().join("flake.nix"))?;
    }

    let metadata = parts_tuples
        .iter()
        .map(|part_tuple| &part_tuple.part.metadata)
        .collect::<Vec<_>>();

    if !cmd.init.shared_args.write_meta {
        log::info!("Rendering `flake-inputs.nix.template` inputs");
        let flake_context = FlakeInputsContext::from_merged_metadata(&metadata);

        let rendered = flake_context.render()?;
        println!("Please add the following snippet to your `flake.nix` inputs:");
        println!("{}", rendered);
    }

    log::info!("Addition succesfully prepared in tmpdir, now copying to target directory");
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
