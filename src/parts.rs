//! Provides a way to parse and store flake parts metadata
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::nix::{eval_meta_file, get_flake_store_path};

#[derive(Debug, Clone)]
pub struct FlakePart {
    pub name: String,
    pub nix_store_path: PathBuf,
    pub metadata: FlakePartMetadata,
}

// TODO helper struct to evade overcomplicated lifetimes
// beware that the store references will live only as long
// as the original function call
#[derive(Debug)]
pub struct FlakePartTuple<'a> {
    pub store: &'a FlakePartsStore,
    pub part: FlakePart,
}

pub fn normalize_flake_string(target: &str, flake: &str, derivation: Option<&str>) -> String {
    if target.contains('#') {
        target.to_string()
    } else if let Some(derivation) = derivation {
        format!("{}#{}/{}", flake, derivation, target)
    } else {
        format!("{}/{}", flake, target)
    }
}

impl<'a> FlakePartTuple<'a> {
    pub fn new(store: &'a FlakePartsStore, part: FlakePart) -> Self {
        Self { store, part }
    }

    pub fn to_flake_uri(&self, derivation: Option<&str>) -> String {
        normalize_flake_string(&self.part.name, &self.store.flake_uri, derivation)
    }

    pub fn resolve_dependencies_of(
        parts_tuples_pool: &Vec<FlakePartTuple>,
        start_indices: Vec<usize>,
    ) -> (Vec<String>, Vec<String>) {
        use std::collections::{HashSet, VecDeque};
        let mut resolved_dependencies = HashSet::new();
        let mut unresolved_dependencies = Vec::new();
        let mut to_process = VecDeque::from(start_indices);

        while let Some(index) = to_process.pop_front() {
            let part_tuple = &parts_tuples_pool[index];
            for dep in &part_tuple.part.metadata.dependencies {
                let normalized_dep = normalize_flake_string(dep, &part_tuple.store.flake_uri, None);
                if resolved_dependencies.insert(normalized_dep.clone()) {
                    // If this is a new dependency, try to find the corresponding PartTuple
                    if let Some(dep_index) = parts_tuples_pool
                        .iter()
                        .position(|p| p.to_flake_uri(None) == normalized_dep)
                    {
                        to_process.push_back(dep_index);
                    } else {
                        // This dependency couldn't be resolved
                        unresolved_dependencies.push(normalized_dep);
                    }
                }
            }
        }

        (
            resolved_dependencies.into_iter().collect(),
            unresolved_dependencies,
        )
    }

    pub fn find_conflicting_parts_in(
        parts_tuples: &'a Vec<FlakePartTuple>,
    ) -> Vec<&'a FlakePartTuple<'a>> {
        let conflicting_parts_uris = parts_tuples
            .iter()
            .flat_map(|part_tuple| {
                part_tuple.part.metadata.conflicts.iter().map(|conflict| {
                    normalize_flake_string(&conflict, &part_tuple.store.flake_uri, None)
                })
            })
            .collect::<Vec<_>>();

        let conflicting_parts: Vec<&'a FlakePartTuple> = parts_tuples
            .iter()
            .filter(|&uri| conflicting_parts_uris.contains(&uri.to_flake_uri(None)))
            .collect::<Vec<_>>();

        conflicting_parts
    }

    pub fn find_missing_parts_in<'b>(
        parts_tuples: &Vec<FlakePartTuple>,
        required_parts: &'b Vec<String>,
    ) -> Vec<&'b String> {
        required_parts
            .iter()
            .filter(|&uri| {
                !parts_tuples.iter().any(|part_tuple| {
                    uri == &part_tuple.to_flake_uri(None) || uri == &part_tuple.part.name
                })
            })
            .collect::<Vec<_>>()
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakePartMetadata {
    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub inputs: JsonValue,

    #[serde(default)]
    pub dependencies: Vec<String>,

    #[serde(default)]
    pub conflicts: Vec<String>,

    #[serde(rename = "extraTrustedPublicKeys", default)]
    extra_trusted_public_keys: Vec<String>,

    #[serde(rename = "extraSubstituters", default)]
    extra_substituters: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FlakeContext {
    pub inputs: JsonValue,
    pub extra_trusted_public_keys: Vec<String>,
    pub extra_substituters: Vec<String>,
}

#[derive(Debug)]
pub struct FlakePartsStore {
    pub flake_uri: String,
    pub nix_store_path: PathBuf,
    pub parts: Vec<FlakePart>,
}

impl FlakeContext {
    fn new(
        inputs: JsonValue,
        extra_trusted_public_keys: Vec<String>,
        extra_substituters: Vec<String>,
    ) -> Self {
        Self {
            inputs,
            extra_trusted_public_keys,
            extra_substituters,
        }
    }

    pub fn from_merged_metadata(metadata: Vec<&FlakePartMetadata>) -> Self {
        let inputs = metadata
            .iter()
            .fold(JsonValue::Object(Default::default()), |mut acc, m| {
                if let (JsonValue::Object(acc_obj), JsonValue::Object(inputs_obj)) =
                    (&mut acc, &m.inputs)
                {
                    for (k, v) in inputs_obj.iter() {
                        acc_obj.insert(k.clone(), v.clone());
                    }
                }
                acc
            });

        let extra_trusted_public_keys = metadata
            .iter()
            .flat_map(|m| m.extra_trusted_public_keys.iter().cloned())
            .collect::<Vec<String>>();

        let extra_substituters = metadata
            .iter()
            .flat_map(|m| m.extra_substituters.iter().cloned())
            .collect::<Vec<String>>();

        Self::new(
            inputs.clone(),
            extra_trusted_public_keys,
            extra_substituters,
        )
    }
}

#[derive(Error, Debug)]
pub enum FlakePartParseError {
    #[error("provided flake part path is invalid")]
    InvalidPathError(),

    #[error("failed to evaluate flake part metadata")]
    NixEvalError(#[from] std::io::Error),

    #[error("failed to parse the UTF-8 output of nix eval")]
    NixEvalUTF8Error(#[from] std::string::FromUtf8Error),

    #[error("failed to convert flake parts metadata to JSON")]
    MetadataConversionError(#[from] serde_json::Error),
}

impl FlakePart {
    fn new(name: String, nix_store_path: PathBuf, metadata: FlakePartMetadata) -> Self {
        Self {
            name,
            nix_store_path,
            metadata,
        }
    }

    pub fn from_path(nix_store_path: PathBuf) -> Result<Self> {
        let name = nix_store_path
            .file_name()
            .ok_or(FlakePartParseError::InvalidPathError())?
            .to_str()
            .ok_or(FlakePartParseError::InvalidPathError())?;

        let eval_output = eval_meta_file(&nix_store_path)?;

        let metadata: FlakePartMetadata = serde_json::from_str(&eval_output)
            .map_err(|e| FlakePartParseError::MetadataConversionError(e))?;

        Ok(Self::new(name.to_string(), nix_store_path, metadata))
    }
}

#[derive(Error, Debug)]
pub enum FlakePartsStoreParseError {
    #[error("failed to realize flake uri to a valid store path")]
    StoreRealizationError(#[from] std::io::Error),

    #[error("failed to parse the UTF-8 output of nix build command")]
    NixBuildUTF8Error(#[from] std::string::FromUtf8Error),

    #[error("failed to parse flake part")]
    FlakePartParseError(#[from] FlakePartParseError),
}

impl FlakePartsStore {
    fn new(flake_uri: String, nix_store_path: PathBuf, parts: Vec<FlakePart>) -> Self {
        Self {
            flake_uri,
            nix_store_path,
            parts,
        }
    }

    // TODO handle errors
    pub fn from_flake_uri(flake_uri: &str) -> Result<Self> {
        let nix_store_path = get_flake_store_path(flake_uri)?;

        let parts = fs::read_dir(nix_store_path.join("flake-parts"))?
            .map(|entry| {
                let entry = entry?;

                Ok(FlakePart::from_path(entry.path())?)
            })
            .collect::<Result<_>>()?;

        Ok(Self::new(flake_uri.to_string(), nix_store_path, parts))
    }
}
