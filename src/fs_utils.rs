use color_eyre::eyre::Result;
use diff::{slice, Result as DiffResult};
use fs_extra::dir::{self, CopyOptions};
use regex::Regex;
use std::fs::{self, File, Permissions};
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::META_FILE;

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
        .filter_entry(|e| e.file_name() != META_FILE)
    {
        let entry = entry?;
        let target_path = dst.join(entry.path().strip_prefix(src)?);

        if target_path.exists() {
            if entry.path().is_file() {
                // Attempt to merge files
                merge_files(entry.path(), &target_path, entry.path())?;
            }
        } else if entry.path().is_dir() {
            dir::create_all(&target_path, false)?;
        } else {
            // TODO copy_with_progress?
            dir::copy(entry.path(), &target_path, options)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, File};
    use tempfile::TempDir;

    #[test]
    fn test_reset_permissions() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().join("test_dir");
        create_dir_all(&test_dir)?;

        // Create a test file with different permissions
        let test_file = test_dir.join("test_file.txt");
        File::create(&test_file)?;
        fs::set_permissions(&test_file, Permissions::from_mode(0o777))?;

        reset_permissions(test_dir.to_str().unwrap())?;

        let metadata = fs::metadata(&test_file)?;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o644);

        let dir_metadata = fs::metadata(&test_dir)?;
        assert_eq!(dir_metadata.permissions().mode() & 0o777, 0o755);

        Ok(())
    }

    #[test]
    fn test_reset_permissions_recursive() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let root_dir = temp_dir.path().join("root");
        let nested_dir = root_dir.join("level1").join("level2");
        create_dir_all(&nested_dir)?;

        // Create files at different levels with different permissions
        let root_file = root_dir.join("root_file.txt");
        let nested_file = nested_dir.join("nested_file.txt");
        File::create(&root_file)?;
        File::create(&nested_file)?;

        fs::set_permissions(&root_file, Permissions::from_mode(0o777))?;
        fs::set_permissions(&nested_file, Permissions::from_mode(0o600))?;
        fs::set_permissions(&nested_dir, Permissions::from_mode(0o700))?;

        reset_permissions(root_dir.to_str().unwrap())?;

        // Check permissions
        assert_eq!(
            fs::metadata(&root_file)?.permissions().mode() & 0o777,
            0o644
        );
        assert_eq!(
            fs::metadata(&nested_file)?.permissions().mode() & 0o777,
            0o644
        );
        assert_eq!(
            fs::metadata(&nested_dir)?.permissions().mode() & 0o777,
            0o755
        );
        assert_eq!(fs::metadata(&root_dir)?.permissions().mode() & 0o777, 0o755);

        Ok(())
    }

    #[test]
    fn test_regex_in_dir_recursive() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test_file.txt");

        let initial_content = "Hello, world! This is a test.";
        fs::write(&test_file, initial_content)?;

        regex_in_dir_recursive(temp_dir.path().to_str().unwrap(), r"world", "universe")?;

        let new_content = fs::read_to_string(&test_file)?;
        assert_eq!(new_content, "Hello, universe! This is a test.");

        Ok(())
    }

    #[test]
    fn test_regex_in_dir_recursive_nested() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let root_dir = temp_dir.path().join("root");
        let nested_dir = root_dir.join("level1").join("level2");
        create_dir_all(&nested_dir)?;

        // Create files at different levels with test content
        let root_file = root_dir.join("root_file.txt");
        let nested_file = nested_dir.join("nested_file.txt");

        fs::write(&root_file, "Hello, world! This is the root file.")?;
        fs::write(&nested_file, "Goodbye, world! This is the nested file.")?;

        regex_in_dir_recursive(root_dir.to_str().unwrap(), r"world", "universe")?;

        // Check content of both files
        let root_content = fs::read_to_string(&root_file)?;
        let nested_content = fs::read_to_string(&nested_file)?;

        assert_eq!(root_content, "Hello, universe! This is the root file.");
        assert_eq!(
            nested_content,
            "Goodbye, universe! This is the nested file."
        );

        Ok(())
    }
}
