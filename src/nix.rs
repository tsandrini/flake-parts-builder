use color_eyre::eyre::Result;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NixError {
    #[error("provided path is invalid: {0}")]
    InvalidPathError(PathBuf),
    #[error("failed to run nix command: {0}")]
    NixCommandError(String),
    #[error("failed to convert output to utf8: {0}")]
    UTF8ConversionError(#[from] std::string::FromUtf8Error),
    #[error("nix command not found. Please ensure 'nix' is installed and in your PATH.")]
    NixNotFound,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub fn get_nix_binary() -> Option<PathBuf> {
    std::env::var_os("NIX_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| which::which("nix").ok())
}

pub fn nix_command() -> Command {
    let mut cmd = Command::new(get_nix_binary().expect("Nix executable not found"));
    cmd.args(&[
        "--extra-experimental-features",
        "nix-command",
        "--extra-experimental-features",
        "flakes",
    ]);
    cmd
}

pub fn eval_nix_file(path: &PathBuf, to_json: bool) -> Result<String, NixError> {
    let path = path
        .to_str()
        .ok_or_else(|| NixError::InvalidPathError(path.clone()))?;

    let mut command = nix_command();
    command.arg("eval");
    command.arg("--file").arg(path);
    if to_json {
        command.arg("--json");
    }

    let output = command.output()?;

    if !output.status.success() {
        return Err(NixError::NixCommandError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.trim().to_string())
}

pub fn get_flake_store_path(flake_uri: &str) -> Result<PathBuf, NixError> {
    let mut command = nix_command();
    command.args(["build", "--no-link", "--print-out-paths", flake_uri]);

    let output = command.output()?;

    if !output.status.success() {
        return Err(NixError::NixCommandError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(PathBuf::from(stdout.trim()))
}

pub fn nixfmt_file(path: &PathBuf) -> Result<()> {
    let path = path.to_str().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid path",
    ))?;

    Command::new("nixfmt").args([&path]).output()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn clean_string(s: &str) -> String {
        s.split_whitespace().collect::<String>()
    }

    #[test]
    fn test_valid_nix_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.nix");
        let mut file = File::create(&file_path)?;
        write!(
            file,
            r#"
              {{
                description = "Test description";
                inputs = {{
                  test.url = "github:test/repo";
                }};
              }}
            "#
        )?;

        let result = eval_nix_file(&file_path, true)?;
        let expected =
            r#"{"description":"Test description","inputs":{"test":{"url":"github:test/repo"}}}"#;

        assert_eq!(clean_string(&result), clean_string(expected));

        Ok(())
    }

    #[test]
    fn test_nonexistent_path() {
        let invalid_path = PathBuf::from("/nonexistent/path");
        let result = eval_nix_file(&invalid_path, true);
        assert!(matches!(result, Err(NixError::NixCommandError(_))));
    }

    #[test]
    fn test_invalid_path() {
        let invalid_path = PathBuf::from("");
        let result = eval_nix_file(&invalid_path, true);
        assert!(matches!(result, Err(NixError::NixCommandError(_))));
    }

    #[test]
    fn test_non_json_output() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.nix");
        let mut file = File::create(&file_path)?;
        write!(file, r#""Hello, World!""#)?;

        let result = eval_nix_file(&file_path, false)?;
        assert_eq!(clean_string(&result), clean_string("\"Hello, World!\""));

        Ok(())
    }

    #[test]
    fn test_complex_nix_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.nix");
        let mut file = File::create(&file_path)?;
        write!(
            file,
            r#"
              {{
                description = "Flake bindings for the `github:cachix/devenv` development environment.";
                inputs = {{
                  devenv.url = "github:cachix/devenv";
                  devenv-root = {{
                    url = "file+file:///dev/null";
                    flake = false;
                  }};
                  mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
                  nix2container = {{
                    url = "github:nlewo/nix2container";
                    inputs.nixpkgs.follows = "nixpkgs";
                  }};
                }};
                conflicts = [ "shells" ];
                extraTrustedPublicKeys = [ "https://devenv.cachix.org" ];
                extraSubstituters = [ "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=" ];
              }}
            "#
        )?;

        let result = eval_nix_file(&file_path, true)?;
        let expected = r#"
          {
            "conflicts":["shells"],
            "description":"Flake bindings for the `github:cachix/devenv` development environment.",
            "extraSubstituters":["devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="],
            "extraTrustedPublicKeys":["https://devenv.cachix.org"],
            "inputs":{
              "devenv":{"url":"github:cachix/devenv"},
              "devenv-root":{"flake":false,"url":"file+file:///dev/null"},
              "mk-shell-bin":{"url":"github:rrbutani/nix-mk-shell-bin"},
              "nix2container":{
                "inputs":{"nixpkgs":{"follows":"nixpkgs"}},
                "url":"github:nlewo/nix2container"
              }
            }
          }
        "#;

        assert_eq!(clean_string(&result), clean_string(expected));

        Ok(())
    }

    #[test]
    fn test_nix_command_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.nix");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "this is not a valid nix expression").unwrap();

        let result = eval_nix_file(&file_path, true);
        assert!(matches!(result, Err(NixError::NixCommandError(_))));
    }

    #[test]
    fn test_json_vs_non_json_output() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.nix");
        let mut file = File::create(&file_path)?;
        write!(
            file,
            r#"
              {{
                x = 42;
                y = "Hello";
              }}
            "#
        )?;

        let json_result = eval_nix_file(&file_path, true)?;
        let non_json_result = eval_nix_file(&file_path, false)?;

        let expected_json = r#"{"x":42,"y":"Hello"}"#;
        assert_eq!(clean_string(&json_result), clean_string(expected_json));

        // For non-JSON output, we can't predict the exact formatting, so we'll check for key elements
        assert!(non_json_result.contains("x = 42"));
        assert!(non_json_result.contains("y = \"Hello\""));

        Ok(())
    }
}
