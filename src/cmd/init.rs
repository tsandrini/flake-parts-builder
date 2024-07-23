use clap::{Args, ValueEnum};
use color_eyre::eyre::Result;
use fs_extra::dir::{self, CopyOptions};
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use thiserror::Error;

use crate::config::{
    BASE_DERIVATION_NAME, BOOTSTRAP_DERIVATION_NAME, FLAKE_INPUTS_TEMPLATE, FLAKE_TEMPLATE,
    META_FILE, NAMEPLACEHOLDER, SELF_FLAKE_URI,
};
use crate::fs_utils::{regex_in_dir_recursive, reset_permissions};
use crate::nix::nixfmt_file;
use crate::parts::{FlakeContext, FlakePartTuple, FlakePartsStore};

/// Initialize a new flake-parts projects using the builder.
#[derive(Debug, Args)]
pub struct InitCommand {
    /// Path for the new desired flake-parts project. This can be either an
    /// already existing path or a new one. Can be relative or absolute.
    #[clap(verbatim_doc_comment)]
    path: PathBuf,

    /// Which parts to include in the project separated by commas. To see
    /// which ones are available use the `list` subcommand.
    #[arg(
        short = 'p',
        long = "parts",
        required = true,
        value_delimiter = ',',
        verbatim_doc_comment
    )]
    parts: Vec<String>,

    /// Additional parts templates stores to load. This currently accepts any
    /// valid flakes derivation URI. For example:
    ///
    /// - `github:tsandrini/flake-parts-builder#flake-parts`
    /// - `../myDir#flake-parts`
    /// - `.#different-flake-parts`
    ///
    /// NOTE: that the derivation needs to have the parts stored at
    /// `$out/flake-parts`
    #[arg(
        short = 'I',
        long = "include",
        value_delimiter = ',',
        verbatim_doc_comment
    )]
    parts_stores: Vec<String>,

    /// Strategy to use when encountering already existing files
    #[arg(value_enum, short, long, default_value = "skip", verbatim_doc_comment)]
    strategy: InitStrategy,

    /// Disable base parts provided by this flake, that is,
    /// `github:tsandrini/flake-parts-builder#flake-parts`. Useful in case
    /// you'd like to override certain parts or simply not use the one provided
    /// by this repo.
    ///
    /// NOTE: _bootstrap part is always included for the project to
    /// properly function (if you really need to you can override the files
    /// with your own versions)
    #[arg(long = "disable-base", default_value_t = false, verbatim_doc_comment)]
    disable_base_parts: bool,

    // TODO
    /// Force initialization in case of conflicting parts. Note that in such
    /// cases you should probably also pass a merging strategy that fits your
    /// specific needs.
    #[arg(
        long = "ignore-conflicts",
        default_value_t = false,
        verbatim_doc_comment
    )]
    ignore_conflicts: bool,

    /// Force overwriting of local files in case of initialization in
    /// a non-empty directory
    #[arg(long = "force", default_value_t = false, verbatim_doc_comment)]
    force: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InitStrategy {
    /// Skip file if already present in the filesystem
    #[clap(verbatim_doc_comment)]
    Skip,

    /// Overwrite file if already present in the filesystem
    #[clap(verbatim_doc_comment)]
    Overwrite,

    /// Try to merge file if already present in the filesystem.
    /// This will use a diff like patching algorithm and may fail
    /// in case of conflicts. (TODO not public yet)
    #[clap(verbatim_doc_comment)]
    Merge,
}

#[derive(Error, Debug)]
pub enum PartsTuplesParsingError {
    #[error("The following user required parts couldn't be resolved: {0:?}")]
    MissingPartsError(Vec<String>),

    #[error("You have requested parts that conflict with each other: {0:?} If you wish to ignore these conflicts use the --ignore-conflicts flag")]
    ConflictingPartsError(Vec<String>),
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
fn parse_required_parts_tuples<'a>(
    cmd: &InitCommand,
    stores: &'a Vec<FlakePartsStore>,
) -> Result<Vec<FlakePartTuple<'a>>, PartsTuplesParsingError> {
    let all_parts_tuples = stores
        .iter()
        .flat_map(|store| {
            store
                .parts
                .iter()
                .map(move |part| FlakePartTuple::new(store, part.to_owned()))
        })
        .collect::<Vec<_>>();

    let user_req_flake_strings = cmd.parts.clone();

    println!("User required parts: {:?}", user_req_flake_strings);

    let parts_uniq_dependencies = {
        let start_indices: Vec<usize> = all_parts_tuples
            .iter()
            .enumerate()
            .filter(|&(_, part_tuple)| {
                let flake_uri = part_tuple.to_flake_uri(None);
                user_req_flake_strings
                    .iter()
                    .any(|req| req == &flake_uri || req == &part_tuple.part.name)
            })
            .map(|(index, _)| index)
            .collect();

        FlakePartTuple::resolve_dependencies_of(&all_parts_tuples, start_indices)
    };

    let all_req_flake_strings = user_req_flake_strings
        .iter()
        .chain(parts_uniq_dependencies.iter())
        .collect::<Vec<_>>();

    println!("All required parts: {:?}", all_req_flake_strings);

    let final_parts_tuples = all_parts_tuples
        .into_iter()
        .filter(|part_tuple| {
            let flake_uri = part_tuple.to_flake_uri(None);
            all_req_flake_strings
                .iter()
                .any(|&req| req == &flake_uri || req == &part_tuple.part.name)
        })
        .collect::<Vec<_>>();

    let final_parts_uris = final_parts_tuples
        .iter()
        .map(|flake_part| flake_part.to_flake_uri(None))
        .collect::<Vec<_>>();

    println!("Final parts: {:?}", final_parts_uris);

    let missing_parts =
        FlakePartTuple::find_missing_parts_in(&final_parts_tuples, &user_req_flake_strings);

    if missing_parts.len() > 0 {
        return Err(PartsTuplesParsingError::MissingPartsError(
            missing_parts.into_iter().cloned().collect::<Vec<_>>(),
        ));
    }

    // TODO probably print that we are ignoring conflicts
    if !cmd.ignore_conflicts {
        // check_for_conflicts(&final_parts_tuples)?;
        let conflicts = FlakePartTuple::find_conflicting_parts_in(&final_parts_tuples);

        if conflicts.len() > 0 {
            return Err(PartsTuplesParsingError::ConflictingPartsError(
                conflicts
                    .into_iter()
                    .map(|flake_part| flake_part.to_flake_uri(None))
                    .collect::<Vec<_>>(),
            ));
        }
    }

    Ok(final_parts_tuples)
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
    parts_tuples: &Vec<FlakePartTuple>,
    target_name: Option<&str>,
    init_strategy: &InitStrategy,
) -> Result<()> {
    // TODO MERGE STRATEGY
    let tmp_path = tmpdir.path();
    for part_tuple in parts_tuples {
        dir::copy(
            &part_tuple.part.nix_store_path,
            &tmp_path,
            &CopyOptions::new()
                .content_only(true)
                .skip_exist(init_strategy == &InitStrategy::Skip)
                .overwrite(init_strategy == &InitStrategy::Overwrite),
        )?;
    }

    // TODO fails if no META_FILE is present
    // check if meta exists and delete it if yes

    std::fs::remove_file(tmp_path.join(META_FILE))?;

    reset_permissions(tmp_path.to_str().unwrap())?;

    {
        let flake_context = {
            let metadata = parts_tuples
                .iter()
                .map(|part_tuple| &part_tuple.part.metadata)
                .collect::<Vec<_>>();
            FlakeContext::from_merged_metadata(metadata)
        };

        let rendered = render_flake_nix(&flake_context)?;
        fs::write(tmp_path.join("flake.nix"), rendered)?;
        nixfmt_file(&tmp_path.join("flake.nix"))?;
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
    cmd.parts_stores
        .push(format!("{}#{}", SELF_FLAKE_URI, BOOTSTRAP_DERIVATION_NAME));
    cmd.parts.push(format!(
        "{}#{}/_bootstrap",
        SELF_FLAKE_URI, BOOTSTRAP_DERIVATION_NAME
    ));

    // NOTE we init stores here to have sensible ownerships of FlakePartTuples
    let stores = cmd
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(&store))
        .collect::<Result<Vec<_>>>()?;

    let parts_tuples = parse_required_parts_tuples(&cmd, &stores)?;

    if !cmd.path.exists() {
        dir::create_all(&cmd.path, false)?;
    }

    let tmpdir = tempdir()?;
    prepare_tmpdir(
        &tmpdir,
        &parts_tuples,
        cmd.path.file_name().unwrap().to_str(),
        &cmd.strategy,
    )?;

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
