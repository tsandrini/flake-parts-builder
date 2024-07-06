use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use thiserror::Error;

use crate::config::META_FILE;

#[derive(Debug, Clone)]
pub struct FlakePart {
    pub name: String,
    pub nix_store_path: PathBuf,
    pub metadata: FlakePartMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakePartMetadata {
    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub inputs: JsonValue,

    #[serde(default)]
    pub dependencies: Vec<String>, // TODO

    #[serde(default)]
    pub conflicts: Vec<String>, // TODg

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

impl FromStr for FlakePart {
    type Err = FlakePartParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FlakePart::from_path(PathBuf::from(s))
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
    fn new(name: &str, nix_store_path: PathBuf, metadata: FlakePartMetadata) -> Self {
        Self {
            name: name.to_string(),
            nix_store_path,
            metadata,
        }
    }

    pub fn from_path(path: PathBuf) -> Result<Self, FlakePartParseError> {
        let name = path
            .file_name()
            .ok_or(FlakePartParseError::InvalidPathError())?
            .to_str()
            .ok_or(FlakePartParseError::InvalidPathError())?;

        let nix_eval = Command::new("nix")
            .args([
                "eval",
                "--expr",
                &format!("let pkgs = import <nixpkgs> {{}}; in import ./{} {{ inherit pkgs; inherit (pkgs) lib; }}", META_FILE),
                "--impure",
                "--json",
            ])
            .current_dir(&path)
            .output()
            .map_err(|e| FlakePartParseError::NixEvalError(e))?;

        let output = String::from_utf8(nix_eval.stdout)
            .map_err(|e| FlakePartParseError::NixEvalUTF8Error(e))?;

        let metadata: FlakePartMetadata = serde_json::from_str(&output)
            .map_err(|e| FlakePartParseError::MetadataConversionError(e))?;

        Ok(Self::new(name, path.clone(), metadata))
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
    fn new(flake_uri: &str, nix_store_path: PathBuf, parts: Vec<FlakePart>) -> Self {
        Self {
            flake_uri: flake_uri.to_string(),
            nix_store_path,
            parts,
        }
    }

    pub fn from_flake_uri(flake_uri: &str) -> Result<Self, FlakePartsStoreParseError> {
        let nix_info = Command::new("nix")
            .args(["build", "--no-link", "--print-out-paths", &flake_uri])
            .output()
            .map_err(|e| FlakePartsStoreParseError::StoreRealizationError(e))?;

        let output = String::from_utf8(nix_info.stdout)
            .map_err(|e| FlakePartsStoreParseError::NixBuildUTF8Error(e))?;

        let parts = fs::read_dir(format!("{}/flake-parts", output.trim()))?
            .map(|entry| {
                let entry = entry?;

                Ok(FlakePart::from_path(entry.path())?)
            })
            .collect::<Result<Vec<FlakePart>, FlakePartParseError>>()?;

        Ok(Self::new(flake_uri, PathBuf::from(output.trim()), parts))
    }
}

impl FromStr for FlakePartsStore {
    type Err = FlakePartsStoreParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FlakePartsStore::from_flake_uri(s)
    }
}
