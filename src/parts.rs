use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct FlakePart {
    flake_uri: String,
    pub short_name: String,
    nix_store_path: PathBuf,
    pub metadata: FlakePartMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlakePartMetadata {
    pub description: String,
    inputs: JsonValue,
    dependencies: Vec<String>,
    conflicts: Vec<String>,

    #[serde(rename = "extraTrustedPublicKeys")]
    extra_trusted_public_keys: Vec<String>,

    #[serde(rename = "extraSubstituters")]
    extra_substituters: Vec<String>,
}

// pub struct (pub FlakePartsStore);

#[derive(Debug)]
pub struct FlakePartsStore {
    flake_uri: String,
    nix_store_path: PathBuf,
    pub parts: Vec<FlakePart>,
}

impl FlakePart {
    pub fn from_path(path: PathBuf, parent_store_flake_uri: &String) -> Result<Self> {
        let name = path.file_name().unwrap().to_str().unwrap();

        let nix_eval = Command::new("nix")
            .args([
                "eval",
                "--expr",
                "let pkgs = import <nixpkgs> {}; in import ./meta.nix { inherit pkgs; inherit (pkgs) lib; }",
                "--impure",
                "--json",
            ])
            .current_dir(&path)
            .output()?;

        let output = String::from_utf8(nix_eval.stdout)?;
        let metadata: FlakePartMetadata = serde_json::from_str(&output)?;

        Ok(Self {
            flake_uri: format!("{}/{}", parent_store_flake_uri, name),
            short_name: name.to_string(),
            nix_store_path: path,
            metadata: metadata,
        })
    }
}

impl FlakePartsStore {
    pub fn from_flake_uri(flake_uri: String) -> Result<Self> {
        let nix_info = Command::new("nix")
            .args(["build", "--no-link", "--print-out-paths", &flake_uri])
            .output()?;

        let output = String::from_utf8(nix_info.stdout)?;

        let parts = fs::read_dir(format!("{}/parts", output.trim()))?
            .map(|entry| {
                let entry = entry?;

                Ok(FlakePart::from_path(entry.path(), &flake_uri)?)
            })
            .collect::<Result<Vec<FlakePart>>>()?;

        Ok(Self {
            nix_store_path: PathBuf::from(&output.trim()),
            flake_uri: flake_uri,
            parts: parts,
        })
    }
}
