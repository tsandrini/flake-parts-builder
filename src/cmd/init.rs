use clap::{Args, ValueEnum};
use color_eyre::eyre::Result;
use fs_extra::dir::{self, CopyOptions};
use itertools::Itertools;
use minijinja::{context, Environment};
use std::fs::{self};
use std::path::PathBuf;
use tempfile::tempdir;

use crate::config::{
    BASE_PARTS_DERIVATION, FLAKE_INPUTS_TEMPLATE, FLAKE_TEMPLATE, META_FILE, NAMEPLACEHOLDER,
    SELF_FLAKE_URI,
};
use crate::parts::{FlakeContext, FlakePart, FlakePartsStore};

use crate::fs_utils::{regex_in_dir_recursive, reset_permissions};

#[derive(Debug, Args)]
pub struct InitCommand {
    /// Path for the desired flake project
    path: PathBuf,

    /// Desired parts to be used
    #[arg(short = 'P', long, required = true, value_delimiter = ',')]
    parts: Vec<String>,

    /// Additional parts templates stores to load
    #[arg(short = 'I', long = "include")]
    parts_stores: Vec<String>,

    /// Strategy to use when encountering already existing files
    #[arg(value_enum, short, long, default_value = "skip")]
    strategy: InitStrategy,

    /// Disable base parts provided by this flake
    /// NOTE: _bootstrap part is always included
    /// for the project to properly function
    #[arg(long = "disable-base", default_value_t = false)]
    disable_base_parts: bool,

    /// Force the initialization even in case of conflicts
    #[arg(long = "force", default_value_t = false)]
    force: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InitStrategy {
    /// Skip file if already present in the filesystem
    Skip,

    /// Overwrite file if already present in the filesystem
    Overwrite,

    /// Try to merge file if already present in the filesystem.
    /// This will use a diff like patching algorithm and may fail
    /// in case of conflicts. (TODO not public yet)
    Merge,
}

pub fn parse_final_parts(cmd: &InitCommand) -> Result<Vec<FlakePart>> {
    // NOTE
    // 1. Load all FlakePartsStores
    // 2. Create an iterator over all parts (don't collect them yet)
    // 3. Construct a final vec of all parts that should be used
    //    a. First we parse the CLI parts
    //    b. Then we iterate over those to add potential dependencies
    //    c. Unique filter
    //    d. Combine these two
    // 4. We finally can create a vec of all parts that should be used
    // 5. Collect! (profit)
    let stores = cmd
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(&store))
        .collect::<Result<Vec<FlakePartsStore>>>()?;

    let proto_out_parts = stores
        .iter()
        .flat_map(|store| store.parts.iter().map(move |part| (store, part)));

    let final_parts_uris = {
        let mut proto_out_parts_uris = cmd
            .parts
            .clone()
            .into_iter()
            .map(|part| {
                if part.contains('#') {
                    part
                } else {
                    format!("{}#{}/{}", SELF_FLAKE_URI, BASE_PARTS_DERIVATION, part)
                }
            })
            .collect::<Vec<_>>();

        let mut dependencies = proto_out_parts
            // NOTE we are traversing twice over the proto_out_parts
            // hence we need the iterator clone
            .clone()
            .flat_map(|(&ref store, &ref part)| {
                part.metadata.dependencies.iter().map(|dep| {
                    if dep.contains('#') {
                        dep.to_string()
                    } else {
                        format!("{}/{}", store.flake_uri, dep)
                    }
                })
            })
            .unique()
            .filter(|uri| !proto_out_parts_uris.contains(&uri))
            .collect::<Vec<_>>();

        proto_out_parts_uris.append(&mut dependencies);
        proto_out_parts_uris
    };

    let final_out_parts = proto_out_parts
        .filter(|(&ref store, &ref part)| {
            final_parts_uris.contains(&&format!("{}/{}", store.flake_uri, part.name))
        })
        // TODO One day, I will figure out how to not clone this
        // fricker .... :copium:
        .map(|(_, part)| part.clone())
        .collect::<Vec<_>>();

    // in case an invalid part is passed
    // let invalid_parts_uris = final_parts_uris
    // in case an invalid part is passed
    // let invalid_parts_uris = final_parts_uris
    //     .iter()
    //     .filter(|uri| {
    //         !final_out_parts
    //             .iter()
    //             .any(|part| part.name == uri.split('/').last().unwrap())
    // })
    // .collect::<Vec<_>>();
    //

    Ok(final_out_parts)
}

pub fn init(mut cmd: InitCommand) -> Result<()> {
    if !cmd.disable_base_parts {
        cmd.parts_stores
            .push(format!("{}#{}", SELF_FLAKE_URI, BASE_PARTS_DERIVATION));
    }

    // NOTE this one is required even if you disable base store parts
    cmd.parts.push(format!(
        "{}#{}/_bootstrap",
        SELF_FLAKE_URI, BASE_PARTS_DERIVATION
    ));

    let final_parts = parse_final_parts(&cmd)?;

    if !cmd.path.exists() {
        dir::create_all(&cmd.path, false)?;
    }

    let target = tempdir()?;
    let target_path = target.path();
    for part in &final_parts {
        dir::copy(
            &part.nix_store_path,
            &target_path,
            &CopyOptions::new().content_only(true).skip_exist(true),
        )?;
    }
    std::fs::remove_file(target.path().join(META_FILE))?;
    reset_permissions(target.path().to_str().unwrap())?;

    {
        let metadata = final_parts
            .iter()
            .map(|part| &part.metadata)
            .collect::<Vec<_>>();

        let flake_context = FlakeContext::from_merged_metadata(metadata);

        let mut env = Environment::new();
        env.add_template("flake.nix", &FLAKE_TEMPLATE).unwrap();
        env.add_template("flake-inputs.nix", &FLAKE_INPUTS_TEMPLATE)
            .unwrap();
        let tmpl = env.get_template("flake.nix").unwrap();
        let rendered = tmpl.render(context! { context => flake_context})?;

        fs::write(target_path.join("flake.nix"), rendered)?;
    }

    // This becomes None when `.`, `../`,etc... is passed
    if let Some(name) = cmd.path.file_name() {
        let name = name.to_str().unwrap(); // TODO unwrap?
        regex_in_dir_recursive(target_path.to_str().unwrap(), &NAMEPLACEHOLDER, name)?;
    }

    dir::copy(
        &target_path,
        &cmd.path,
        &CopyOptions::new()
            .content_only(true)
            .skip_exist(cmd.strategy == InitStrategy::Skip)
            .overwrite(cmd.strategy == InitStrategy::Overwrite),
    )?;

    Ok(())
}
