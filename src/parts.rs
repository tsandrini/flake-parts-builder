use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug)]
pub struct FlakePart {
    pub flake_uri: String,
    pub short_name: String,
    nix_store_path: PathBuf,
    pub metadata: FlakePartMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlakePartMetadata {
    #[serde(default)]
    pub description: String,

    #[serde(default)]
    inputs: JsonValue,

    #[serde(default)]
    dependencies: Vec<String>,

    #[serde(default)]
    conflicts: Vec<String>,

    #[serde(rename = "extraTrustedPublicKeys", default)]
    extra_trusted_public_keys: Vec<String>,

    #[serde(rename = "extraSubstituters", default)]
    extra_substituters: Vec<String>,

    #[serde(default)]
    gitignore: Vec<String>,
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
    nix_store_path: PathBuf,
    pub parts: Vec<FlakePart>,
}

impl FlakeContext {
    pub fn from_metadata(metadata: &FlakePartMetadata) -> Self {
        Self {
            inputs: metadata.inputs.clone(),
            extra_trusted_public_keys: metadata.extra_trusted_public_keys.clone(),
            extra_substituters: metadata.extra_substituters.clone(),
        }
    }

    pub fn from_metadata_merge(metadata: &[FlakePartMetadata]) -> Self {
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

        Self {
            inputs,
            extra_trusted_public_keys,
            extra_substituters,
        }
    }
}

impl FromStr for FlakePart {
    type Err = FlakePartParseError;

    fn from_str(s: &str) -> Result<Self, FlakePartParseError> {
        FlakePart::from_path(PathBuf::from(s), "".to_string())
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
    pub fn from_path(
        path: PathBuf,
        parent_store_flake_uri: String,
    ) -> Result<Self, FlakePartParseError> {
        let name = path
            .file_name()
            .ok_or(FlakePartParseError::InvalidPathError())?
            .to_str()
            .ok_or(FlakePartParseError::InvalidPathError())?;

        let nix_eval = Command::new("nix")
            .args([
                "eval",
                "--expr",
                "let pkgs = import <nixpkgs> {}; in import ./meta.nix { inherit pkgs; inherit (pkgs) lib; }",
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

        let flake_uri = format!("{}/{}", parent_store_flake_uri, name);
        Ok(Self {
            flake_uri: flake_uri,
            short_name: name.to_string(),
            nix_store_path: path,
            metadata: metadata,
        })
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

                Ok(FlakePart::from_path(entry.path(), flake_uri.to_string())?)
            })
            .collect::<Result<Vec<FlakePart>, FlakePartParseError>>()?;

        Ok(Self {
            nix_store_path: PathBuf::from(&output.trim()),
            flake_uri: flake_uri.to_string(),
            parts: parts,
        })
    }
}

impl FromStr for FlakePartsStore {
    type Err = FlakePartsStoreParseError;

    fn from_str(s: &str) -> Result<Self, FlakePartsStoreParseError> {
        FlakePartsStore::from_flake_uri(s)
    }
}
