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
        .args(["eval", "--json", "--file", META_FILE])
        .current_dir(&path)
        .output()?;

    let output = String::from_utf8(nix_eval.stdout)?;
    Ok(output)
}

pub fn nixfmt_file(path: &PathBuf) -> Result<()> {
    let path = path.to_str().unwrap(); // TODO

    Command::new("nixfmt").args([&path]).output()?;

    Ok(())
}
