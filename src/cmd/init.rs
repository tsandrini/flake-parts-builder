use clap::{Args, ValueEnum};
use color_eyre::eyre::Result;
use fs_extra::dir::{self, CopyOptions};
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

use crate::config::{
    BASE_DERIVATION_NAME, FLAKE_INPUTS_TEMPLATE, FLAKE_TEMPLATE, META_FILE, NAMEPLACEHOLDER,
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

fn process_flake_string(target: &str, flake: &str, derivation: Option<&str>) -> String {
    if target.contains('#') {
        target.to_string()
    } else if let Some(derivation) = derivation {
        format!("{}#{}/{}", flake, derivation, target)
    } else {
        format!("{}/{}", flake, target)
    }
}

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
fn parse_parts(cmd: &InitCommand) -> Result<Vec<FlakePart>> {
    use std::collections::HashSet;

    let stores = cmd
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(&store))
        .collect::<Result<Vec<_>>>()?;

    let all_parts_with_stores = stores
        .iter()
        .flat_map(|store| store.parts.iter().map(move |part| (store, part)));

    let user_required_parts_uris = cmd
        .parts
        .clone()
        .into_iter()
        .map(|part| process_flake_string(&part, &SELF_FLAKE_URI, Some(&BASE_DERIVATION_NAME)))
        .collect::<Vec<_>>();

    let parts_uniq_dependencies = {
        let mut seen = HashSet::new();

        all_parts_with_stores
            // NOTE we are traversing twice over the proto_out_parts
            // hence we need the iterator clone
            .clone()
            .flat_map(|(&ref store, &ref part)| {
                part.metadata
                    .dependencies
                    .iter()
                    .map(|dep| process_flake_string(&dep, &store.flake_uri, None))
            })
            .filter(|uri| seen.insert(uri.clone()))
            .filter(|uri| !user_required_parts_uris.contains(&uri))
            .collect::<Vec<_>>()
    };

    let all_parts_uris = user_required_parts_uris
        .iter()
        .chain(parts_uniq_dependencies.iter())
        .collect::<Vec<_>>();

    let final_parts_with_stores = all_parts_with_stores
        .filter(|(&ref store, &ref part)| {
            all_parts_uris.contains(&&format!("{}/{}", store.flake_uri, part.name))
        })
        .map(move |(store, part)| (store, part.to_owned()))
        .collect::<Vec<_>>();

    validate_parsed_parts(&user_required_parts_uris, &final_parts_with_stores)?;

    let parts = final_parts_with_stores
        .iter()
        .map(move |(_, part)| part.to_owned())
        .collect::<Vec<_>>();

    // TODO check conflicts
    Ok(parts)
}

// Assuming `parse_to_flake_uri` and similar utility functions exist to handle URI parsing.

fn validate_parsed_parts(
    user_required_parts_uris: &Vec<String>,
    result: &Vec<(&FlakePartsStore, FlakePart)>,
) -> Result<()> {
    Ok(())
}

fn render_flake_nix(flake_context: &FlakeContext) -> Result<String> {
    use minijinja::{context, Environment};

    let mut env = Environment::new();
    env.add_template("flake.nix", &FLAKE_TEMPLATE).unwrap();
    env.add_template("flake-inputs.nix", &FLAKE_INPUTS_TEMPLATE)
        .unwrap();
    let tmpl = env.get_template("flake.nix").unwrap();
    let rendered = tmpl.render(context! ( context => flake_context))?;
    Ok(rendered)
}

fn prepare_tmpdir(
    tmpdir: &TempDir,
    parts: &Vec<FlakePart>,
    target_name: Option<&str>,
) -> Result<()> {
    let tmp_path = tmpdir.path();
    for part in parts {
        dir::copy(
            &part.nix_store_path,
            &tmp_path,
            &CopyOptions::new().content_only(true).skip_exist(true),
        )?;
    }
    // TODO fails if no META_FILE is present
    std::fs::remove_file(tmp_path.join(META_FILE))?;
    reset_permissions(tmp_path.to_str().unwrap())?;

    {
        let flake_context = {
            let metadata = parts.iter().map(|part| &part.metadata).collect::<Vec<_>>();
            FlakeContext::from_merged_metadata(metadata)
        };
        println!("flake_context: {:?}", flake_context);

        let rendered = render_flake_nix(&flake_context)?;
        fs::write(tmp_path.join("flake.nix"), rendered)?;
    }

    // This becomes None when `.`, `../`,etc... is passed
    if let Some(name) = target_name {
        regex_in_dir_recursive(tmp_path.to_str().unwrap(), &NAMEPLACEHOLDER, name)?;
    }

    Ok(())
}

pub fn init(mut cmd: InitCommand) -> Result<()> {
    if !cmd.disable_base_parts {
        cmd.parts_stores
            .push(format!("{}#{}", SELF_FLAKE_URI, BASE_DERIVATION_NAME));
    }

    // NOTE this one is required even if you disable base store parts
    cmd.parts.push(format!(
        "{}#{}/_bootstrap",
        SELF_FLAKE_URI, BASE_DERIVATION_NAME
    ));

    let parts = parse_parts(&cmd)?;

    if !cmd.path.exists() {
        dir::create_all(&cmd.path, false)?;
    }

    let tmpdir = tempdir()?;
    prepare_tmpdir(&tmpdir, &parts, cmd.path.file_name().unwrap().to_str())?;

    dir::copy(
        &tmpdir,
        &cmd.path,
        &CopyOptions::new()
            .content_only(true)
            .skip_exist(cmd.strategy == InitStrategy::Skip)
            .overwrite(cmd.strategy == InitStrategy::Overwrite),
    )?;

    Ok(())
}
