//! Provides a way to parse and store flake parts metadata
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::config::META_FILE;
use crate::nix::NixCmdInterface;

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
        target.to_string() // OK
    } else if flake.contains('#') {
        format!("{}/{}", flake, target)
    } else if let Some(derivation) = derivation {
        format!("{}#{}/{}", flake, derivation, target)
    } else {
        format!("{}#default/{}", flake, target)
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
        parts_tuples_pool: &[FlakePartTuple],
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
        parts_tuples: &'a [FlakePartTuple],
    ) -> Vec<&'a FlakePartTuple<'a>> {
        let conflicting_parts_uris = parts_tuples
            .iter()
            .flat_map(|part_tuple| {
                part_tuple.part.metadata.conflicts.iter().map(|conflict| {
                    normalize_flake_string(conflict, &part_tuple.store.flake_uri, None)
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
        parts_tuples: &[FlakePartTuple],
        required_parts: &'b [String],
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
    pub extra_trusted_public_keys: Vec<String>,

    #[serde(rename = "extraSubstituters", default)]
    pub extra_substituters: Vec<String>,
}

#[derive(Debug)]
pub struct FlakePartsStore {
    pub flake_uri: String,
    pub nix_store_path: PathBuf,
    pub parts: Vec<FlakePart>,
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

    pub fn from_path(nix_store_path: PathBuf, nix_cmd: &impl NixCmdInterface) -> Result<Self> {
        let name = nix_store_path
            .file_name()
            .ok_or(FlakePartParseError::InvalidPathError())?
            .to_str()
            .ok_or(FlakePartParseError::InvalidPathError())?;

        let eval_output = nix_cmd.eval_nix_file(&nix_store_path.join(META_FILE), true)?;

        let metadata: FlakePartMetadata = serde_json::from_str(&eval_output)
            .map_err(FlakePartParseError::MetadataConversionError)?;

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
    pub fn from_flake_uri(flake_uri: &str, nix_cmd: &impl NixCmdInterface) -> Result<Self> {
        let nix_store_path = nix_cmd.store_path_of_flake(flake_uri)?;

        let parts = fs::read_dir(nix_store_path.join("flake-parts"))?
            .map(|entry| {
                let entry = entry?;

                FlakePart::from_path(entry.path(), nix_cmd)
            })
            .collect::<Result<_>>()?;

        Ok(Self::new(flake_uri.to_string(), nix_store_path, parts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_flake_string_with_hash_in_target() {
        let result = normalize_flake_string("github:user/repo#output", "unused", None);
        assert_eq!(result, "github:user/repo#output");
    }

    #[test]
    fn test_normalize_flake_string_with_hash_in_flake() {
        let result = normalize_flake_string("output", "github:user/repo#derivation", None);
        assert_eq!(result, "github:user/repo#derivation/output");
    }

    #[test]
    fn test_normalize_flake_string_with_derivation() {
        let result = normalize_flake_string("target", "github:user/repo", Some("derivation"));
        assert_eq!(result, "github:user/repo#derivation/target");
    }

    #[test]
    fn test_normalize_flake_string_without_derivation() {
        let result = normalize_flake_string("target", "github:user/repo", None);
        assert_eq!(result, "github:user/repo#default/target");
    }

    #[test]
    fn test_normalize_flake_string_with_empty_target() {
        let result = normalize_flake_string("", "github:user/repo", Some("derivation"));
        assert_eq!(result, "github:user/repo#derivation/");
    }

    #[test]
    fn test_normalize_flake_string_with_empty_target_and_no_derivation() {
        let result = normalize_flake_string("", "github:user/repo", None);
        assert_eq!(result, "github:user/repo#default/");
    }

    #[test]
    fn test_normalize_flake_string_with_complex_target() {
        let result =
            normalize_flake_string("path/to/target", "github:user/repo", Some("derivation"));
        assert_eq!(result, "github:user/repo#derivation/path/to/target");
    }

    #[test]
    fn test_normalize_flake_string_with_hash_in_flake_and_derivation() {
        let result =
            normalize_flake_string("output", "github:user/repo#derivation", Some("unused"));
        assert_eq!(result, "github:user/repo#derivation/output");
    }

    #[test]
    fn test_normalize_flake_string_with_local_path() {
        let result = normalize_flake_string("target", "./local/path", Some("derivation"));
        assert_eq!(result, "./local/path#derivation/target");
    }

    #[test]
    fn test_normalize_flake_string_with_hash_in_flake_and_target() {
        let result = normalize_flake_string("output#extra", "github:user/repo#branch", None);
        assert_eq!(result, "output#extra");
    }
}
