use color_eyre::eyre::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::config::META_FILE;

pub fn get_flake_store_path(flake_uri: &str) -> Result<PathBuf> {
    let nix_info = Command::new("nix")
        .args(["build", "--no-link", "--print-out-paths", &flake_uri])
        .output()?;

    let output = String::from_utf8(nix_info.stdout)?;
    Ok(PathBuf::from(output.trim()))
}

pub fn eval_meta_file(path: &PathBuf) -> Result<String> {
    let nix_eval = Command::new("nix")
            .args([
                "eval",
                "--expr",
                &format!("let pkgs = import <nixpkgs> {{}}; in import ./{} {{ inherit pkgs; inherit (pkgs) lib; }}", META_FILE),
                "--impure",
                "--json",
            ])
            .current_dir(&path)
            .output()?;

    let output = String::from_utf8(nix_eval.stdout)?;
    Ok(output)
}
