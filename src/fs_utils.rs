use anyhow::Result;
use diff::{slice, Result as DiffResult};
use fs_extra::dir::{self, CopyOptions};
use itertools::Itertools;
use regex::Regex;
use std::fs::{self, File, Permissions};
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use tempfile::tempdir; // TODO FIXME
use walkdir::WalkDir;

// TODO might implement a "merging" strategy instead of skipping/overwriting
// but currently not entirely sure about its use case
#[allow(dead_code)]
pub fn merge_files(base: &Path, theirs: &Path, ours: &Path) -> Result<()> {
    let base_content = fs::read_to_string(base)?;
    let their_content = fs::read_to_string(theirs)?;
    let our_content = fs::read_to_string(ours)?;

    let base_lines = base_content.lines().collect::<Vec<_>>();
    let their_lines = their_content.lines().collect::<Vec<_>>();
    let our_lines = our_content.lines().collect::<Vec<_>>();

    let diffs = slice(&base_lines, &their_lines);

    let mut merged_lines = Vec::new();
    let mut our_iter = our_lines.iter();
    for diff in diffs {
        match diff {
            DiffResult::Both(_, line) => {
                // Both versions are the same
                merged_lines.push(*line);
                our_iter.next(); // Move along ours as well
            }
            DiffResult::Left(_) => {
                // Line is only in base, so we take from ours
                if let Some(line) = our_iter.next() {
                    merged_lines.push(*line);
                }
            }
            DiffResult::Right(line) => {
                // Line is only in theirs
                merged_lines.push(*line);
            }
        }
    }
    // Append remaining lines from ours
    for line in our_iter {
        merged_lines.push(*line);
    }

    fs::write(ours, merged_lines.join("\n"))?;
    Ok(())
}

// TODO
#[allow(dead_code)]
pub fn merge_dirs(src: &Path, dst: &Path, options: &CopyOptions) -> Result<()> {
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_entry(|e| e.file_name() != "meta.nix")
    {
        let entry = entry?;
        let target_path = dst.join(entry.path().strip_prefix(src)?);

        if target_path.exists() && false {
            if entry.path().is_file() {
                // Attempt to merge files
                merge_files(&entry.path(), &target_path, &entry.path())?;
            }
        } else {
            if entry.path().is_dir() {
                dir::create_all(&target_path, false)?;
            } else {
                // TODO copy_with_progress?
                dir::copy(&entry.path(), &target_path, &options)?;
            }
        }
    }
    Ok(())
}

pub fn reset_permissions(path: &str) -> std::io::Result<()> {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let metadata = fs::metadata(path)?;

        if metadata.is_dir() {
            fs::set_permissions(path, Permissions::from_mode(0o755))?;
        } else if metadata.is_file() {
            fs::set_permissions(path, Permissions::from_mode(0o644))?;
        }
    }
    Ok(())
}

pub fn regex_in_dir_recursive(dir: &str, pattern: &str, replacement: &str) -> io::Result<()> {
    let re = Regex::new(pattern).unwrap();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let mut contents = String::new();
            {
                let mut file = File::open(path)?;
                file.read_to_string(&mut contents)?;
            }
            let new_contents = re.replace_all(&contents, replacement).to_string();
            if new_contents != contents {
                let mut file = File::create(path)?;
                file.write_all(new_contents.as_bytes())?;
            }
        }
    }
    Ok(())
}
