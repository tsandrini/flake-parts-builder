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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakePartMetadata {
    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub inputs: JsonValue,

    #[serde(default)]
    pub dependencies: Vec<String>,

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
