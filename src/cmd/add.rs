use clap::Args;
use color_eyre::eyre::Result;
use fs_extra::dir::{self, CopyOptions};
use tempfile::tempdir;

use crate::cmd::init::{parse_required_parts_tuples, prepare_tmpdir, InitCommand};
use crate::config::{BASE_DERIVATION_NAME, FLAKE_INPUTS_TEMPLATE, SELF_FLAKE_URI};
use crate::parts::{FlakeContext, FlakePartsStore};

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

fn render_flake_inputs(flake_context: &FlakeContext) -> Result<String> {
    use minijinja::{context, Environment};

    let mut env = Environment::new();
    env.add_template("flake-inputs.nix", &FLAKE_INPUTS_TEMPLATE)
        .unwrap();
    let tmpl = env.get_template("flake-inputs.nix").unwrap();
    let rendered = tmpl.render(context! ( context => flake_context))?;
    Ok(rendered)
}

pub fn add(add_cmd: AddCommand) -> Result<()> {
    let mut cmd = add_cmd.init;

    if !cmd.disable_base_parts {
        cmd.parts_stores
            .push(format!("{}#{}", SELF_FLAKE_URI, BASE_DERIVATION_NAME));
    }

    // TODO we probably don't need to bootstrap again?
    // cmd.parts_stores
    //     .push(format!("{}#{}", SELF_FLAKE_URI, BOOTSTRAP_DERIVATION_NAME));
    // cmd.parts.push(format!(
    //     "{}#{}/_bootstrap",
    //     SELF_FLAKE_URI, BOOTSTRAP_DERIVATION_NAME
    // ));

    // NOTE we init stores here to have sensible ownerships of FlakePartTuples
    let stores = cmd
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(&store))
        .collect::<Result<Vec<_>>>()?;

    let parts_tuples = parse_required_parts_tuples(&cmd, &stores)?;

    let path = cmd.path.canonicalize().unwrap_or_else(|_| cmd.path.clone());

    if !path.exists() {
        dir::create_all(&path, false)?;
    }

    let tmpdir = tempdir()?;
    prepare_tmpdir(&tmpdir, &parts_tuples, path.to_str(), &cmd.strategy, false)?;

    // NOTE the flake.nix flake shouldn't be present due to the strucutre of
    // flake-parts, but I am way tooo paranoid.
    if tmpdir.path().join("flake.nix").exists() {
        std::fs::remove_file(tmpdir.path().join("flake.nix"))?;
    }

    let flake_context = {
        let metadata = parts_tuples
            .iter()
            .map(|part_tuple| &part_tuple.part.metadata)
            .collect::<Vec<_>>();

        FlakeContext::from_merged_metadata(metadata)
    };

    let rendered = render_flake_inputs(&flake_context)?;
    println!("Please add the following inputs to your flake.nix:");
    println!("{}", rendered);

    dir::copy(
        &tmpdir,
        &cmd.path,
        &CopyOptions::new()
            .content_only(true)
            .skip_exist(!cmd.force)
            .overwrite(cmd.force),
    )?;

    Ok(())
}
