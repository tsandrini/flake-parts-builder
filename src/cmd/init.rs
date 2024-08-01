use clap::{Args, ValueEnum};
use color_eyre::eyre::Result;
use fs_extra::dir::{self, CopyOptions};
use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use thiserror::Error;

use crate::cmd::SharedArgs;
use crate::config::{
    BASE_DERIVATION_NAME, BOOTSTRAP_DERIVATION_NAME, META_FILE, NAMEPLACEHOLDER, SELF_FLAKE_URI,
};
use crate::fs_utils::{regex_in_dir_recursive, reset_permissions};
use crate::nix::NixCmdInterface;
use crate::parts::{FlakePartTuple, FlakePartsStore};
use crate::templates::FlakeContext;

/// Initialize a new flake-parts projects using the builder.
#[derive(Debug, Args)]
pub struct InitCommand {
    #[clap(flatten)]
    pub shared_args: SharedArgs,

    /// Path (relative or absolute) for the desired flake-parts project.
    /// You can either pass an empty or non-existing directory, in which case
    /// all content will be new or you can pass an existing directory already
    /// populated with files. In such case the directories will be merged
    /// according to the strategy specified in `--strategy`.
    #[clap(verbatim_doc_comment)]
    pub path: PathBuf,

    /// Which parts to include in the project separated by commas. To see
    /// which ones are available use the `list` subcommand.
    #[arg(
        short = 'p',
        long = "parts",
        required = true,
        value_delimiter = ',',
        verbatim_doc_comment
    )]
    pub parts: Vec<String>,

    /// Strategy to use when encountering already existing files
    #[arg(value_enum, short, long, default_value = "skip", verbatim_doc_comment)]
    pub strategy: InitStrategy,

    /// Force initialization in case of conflicting parts. Note that in such
    /// cases you should probably also pass a merging strategy that fits your
    /// specific needs.
    #[arg(
        long = "ignore-conflicts",
        default_value_t = false,
        verbatim_doc_comment
    )]
    pub ignore_conflicts: bool,

    /// Force initialization in case of unresolved dependencies. This can happen
    /// if you request parts that have 3rd party dependencies on parts stores
    /// that aren't included via the `--include` or `-I` flag.
    #[arg(
        long = "ignore-unresolved-deps",
        default_value_t = false,
        verbatim_doc_comment
    )]
    pub ignore_unresolved_deps: bool,

    /// Force overwriting of local files in case of initialization in
    /// a non-empty directory
    #[arg(long = "force", default_value_t = false, verbatim_doc_comment)]
    pub force: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, ValueEnum)]
pub enum InitStrategy {
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

    #[error("You have requested parts that conflict with each other: {0:?} If you wish to ignore these conflicts use the `--ignore-conflicts` flag")]
    ConflictingPartsError(Vec<String>),

    #[error("The following dependencies were required but couldn't be resolved: {0:?} Please include the necessary flake-parts stores using the `-I` flag or pass the `--ignore-unresolved-deps` flag to ignore this error and force initialization.")]
    UnresolvedDependenciesError(Vec<String>),
}

pub fn parse_required_parts_tuples<'a>(
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

    log::debug!("User requested parts: {:?}", user_req_flake_strings);

    let (resolved_deps, unresolved_deps) = {
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

    if !unresolved_deps.is_empty() {
        return Err(PartsTuplesParsingError::UnresolvedDependenciesError(
            unresolved_deps,
        ));
    }

    let all_req_flake_strings = user_req_flake_strings
        .iter()
        .chain(resolved_deps.iter())
        .collect::<Vec<_>>();

    log::debug!("Resolved dependencies: {:?}", resolved_deps);
    log::debug!("Unresolved dependencies: {:?}", unresolved_deps);
    log::debug!("All required parts: {:?}", all_req_flake_strings);

    let final_parts_tuples = all_parts_tuples
        .into_iter()
        .filter(|part_tuple| {
            let flake_uri = part_tuple.to_flake_uri(None);
            all_req_flake_strings
                .iter()
                .any(|&req| req == &flake_uri || req == &part_tuple.part.name)
        })
        .collect::<Vec<_>>();

    let missing_parts =
        FlakePartTuple::find_missing_parts_in(&final_parts_tuples, &user_req_flake_strings);

    if missing_parts.len() > 0 {
        log::error!("Missing parts: {:?}", missing_parts);
        return Err(PartsTuplesParsingError::MissingPartsError(
            missing_parts.into_iter().cloned().collect::<Vec<_>>(),
        ));
    }

    if !cmd.ignore_conflicts {
        // check_for_conflicts(&final_parts_tuples)?;
        let conflicts = FlakePartTuple::find_conflicting_parts_in(&final_parts_tuples);

        if conflicts.len() > 0 {
            log::error!("Conflicting parts: {:?}", conflicts);
            return Err(PartsTuplesParsingError::ConflictingPartsError(
                conflicts
                    .into_iter()
                    .map(|flake_part| flake_part.to_flake_uri(None))
                    .collect::<Vec<_>>(),
            ));
        }
    } else {
        log::warn!("Ignoring conflicts");
    }

    Ok(final_parts_tuples)
}

pub fn prepare_tmpdir(
    nix_cmd: &impl NixCmdInterface,
    tmpdir: &TempDir,
    parts_tuples: &Vec<FlakePartTuple>,
    target_name: Option<&str>,
    init_strategy: &InitStrategy,
    render_flake_nix: bool,
) -> Result<()> {
    // TODO MERGE STRATEGY
    let tmp_path = tmpdir.path();
    for part_tuple in parts_tuples {
        log::debug!(
            "Copying the following part into tmpdir: {:?}",
            part_tuple.part.name
        );
        dir::copy(
            &part_tuple.part.nix_store_path,
            &tmp_path,
            &CopyOptions::new()
                .content_only(true)
                .skip_exist(init_strategy == &InitStrategy::Skip)
                .overwrite(init_strategy == &InitStrategy::Overwrite),
        )?;
    }

    log::debug!("Removing meta file from tmpdir");
    std::fs::remove_file(tmp_path.join(META_FILE))?;

    log::info!("Resetting permissions in tmpdir");
    reset_permissions(tmp_path.to_str().unwrap())?;

    if render_flake_nix {
        log::info!("Rendering `flake.nix.template` in tmpdir");

        let metadata = parts_tuples
            .iter()
            .map(|part_tuple| &part_tuple.part.metadata)
            .collect::<Vec<_>>();

        let flake_context = FlakeContext::from_merged_metadata(&metadata);

        let rendered = flake_context.render()?;
        fs::write(tmp_path.join("flake.nix"), rendered)?;
        log::info!("Running nixfmt on flake.nix in tmpdir");
        nix_cmd.nixfmt_file(&tmp_path.join("flake.nix"))?;
        // nixfmt_file(&tmp_path.join("flake.nix"))?;
    } else {
        log::info!("Skipping rendering of `flake.nix.template`");
    }

    // This becomes None when `.`, `../`,etc... is passed
    if let Some(name) = target_name {
        log::info!(
            "Globally replacing NAMEPLACEHOLDER in tmpdir to name: {}",
            name
        );
        regex_in_dir_recursive(tmp_path.to_str().unwrap(), &NAMEPLACEHOLDER, name)?;
    }

    Ok(())
}

pub fn init(mut cmd: InitCommand, nix_cmd: impl NixCmdInterface) -> Result<()> {
    if !cmd.shared_args.disable_base_parts {
        log::info!("Adding base parts store to `cmd.shared_args.parts_stores`");

        cmd.shared_args
            .parts_stores
            .push(format!("{}#{}", SELF_FLAKE_URI, BASE_DERIVATION_NAME));
    }

    log::info!("Adding bootstrap parts store to `cmd.shared_args.parts_stores`");
    cmd.shared_args
        .parts_stores
        .push(format!("{}#{}", SELF_FLAKE_URI, BOOTSTRAP_DERIVATION_NAME));

    log::info!("Adding _bootstrap to required `cmd.parts`");
    cmd.parts.push(format!(
        "{}#{}/_bootstrap",
        SELF_FLAKE_URI, BOOTSTRAP_DERIVATION_NAME
    ));

    // NOTE we init stores here to have sensible ownerships of FlakePartTuples
    let stores = cmd
        .shared_args
        .parts_stores
        .iter()
        .map(|store| FlakePartsStore::from_flake_uri(&store, &nix_cmd))
        .collect::<Result<Vec<_>>>()?;

    log::debug!(
        "All parts stores: {:?}",
        stores
            .iter()
            .map(|store| store.flake_uri.clone())
            .collect::<Vec<_>>()
    );

    let parts_tuples = parse_required_parts_tuples(&cmd, &stores)?;

    let path = cmd.path.canonicalize().unwrap_or_else(|_| cmd.path.clone());
    log::debug!("Full user provided path: {:?}", path);

    if !path.exists() {
        log::info!("Provided path doesn't exist, creating it");
        dir::create_all(&path, false)?;
    }

    let tmpdir = tempdir()?;
    log::info!("Preparing new project in a tmpdir at {:?}", tmpdir.path());
    prepare_tmpdir(
        &nix_cmd,
        &tmpdir,
        &parts_tuples,
        path.file_name().map(|osstr| osstr.to_str().unwrap()),
        &cmd.strategy,
        true,
    )?;

    log::info!("Project successfully prepared in tmpdir, now copying to target directory");
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
