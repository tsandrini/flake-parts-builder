use anyhow::Result;
use clap::Args;
use itertools::Itertools;
use minijinja::{context, Environment};
use std::path::PathBuf;
use tempfile::tempdir; // TODO FIXME

use crate::config::{FLAKE_TEMPLATE, SELF_FLAKE_URI};
use crate::parts::{FlakeContext, FlakePart, FlakePartsStore, FlakePartsStoreParseError};

#[derive(Debug, Args)]
pub struct InitCommand {
    /// Path for the desired flake project
    path: PathBuf,

    /// Desired parts to be used
    #[arg(short = 'P', long, required = true, value_delimiter = ',')]
    parts: Vec<String>,

    /// Additional parts templates stores to load
    #[arg(short = 'S', long = "stores")]
    parts_stores: Vec<String>,

    /// Disable base parts provided by this flake
    #[arg(long = "disable-base-parts", default_value_t = false)]
    disable_base_parts: bool,

    /// Force the initialization even in case of conflicts
    #[arg(long = "force", default_value_t = false)]
    force: bool,
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
        .collect::<Result<Vec<FlakePartsStore>, FlakePartsStoreParseError>>()?;

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
                    format!("{}#flake-parts/{}", SELF_FLAKE_URI, part)
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
            .filter(|uri| !proto_out_parts_uris.contains(uri))
            .collect::<Vec<_>>();

        proto_out_parts_uris.append(&mut dependencies);
        proto_out_parts_uris
    };

    let final_out_parts = proto_out_parts
        .filter(|(&ref store, &ref part)| {
            final_parts_uris.contains(&format!("{}/{}", store.flake_uri, part.name))
        })
        .map(|(_, part)| part.clone())
        .collect::<Vec<FlakePart>>();

    Ok(final_out_parts)
}

pub fn init(mut cmd: InitCommand) -> Result<()> {
    let target = tempdir()?;

    if !cmd.disable_base_parts {
        cmd.parts_stores
            .push(format!("{}#flake-parts", SELF_FLAKE_URI));
    }

    // NOTE this one is required even if you disable this parts store
    cmd.parts
        .push(format!("{}#flake-parts/_base", SELF_FLAKE_URI));

    let final_parts = parse_final_parts(&cmd)?;

    // TODO handle conflicts
    println!("{:?}", cmd.path.file_name());

    if cmd.path.exists() {
        println!("Path already exists");
    }

    // println!("{:?}", cmd.path.);

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

    Ok(())
}
