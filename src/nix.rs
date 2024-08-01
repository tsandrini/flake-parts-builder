use color_eyre::eyre::Result;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NixCmdInterfaceError {
    #[error("failed to run cmd due to invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("failed to convert output to UTF-8: {0}")]
    UTF8ConversionError(String),

    #[error("failed to run nix command: {0}")]
    NixCommandError(String),
}

pub trait NixCmdInterface {
    // TODO figure out how to remove the static lifetime
    type Error: From<NixCmdInterfaceError> + std::error::Error + Send + Sync + 'static;

    fn eval_nix_file(&self, path: &PathBuf, to_json: bool) -> Result<String, Self::Error>;
    fn store_path_of_flake(&self, flake_uri: &str) -> Result<PathBuf, Self::Error>; // TODO maybe rename
    fn nixfmt_file(&self, path: &PathBuf) -> Result<(), Self::Error>;
}

pub struct NixExecutor {
    nix_binary: PathBuf,
}

#[derive(Error, Debug)]
pub enum NixExecutorError {
    #[error("{0}")]
    NixCmdInterfaceError(#[from] NixCmdInterfaceError),

    #[error("nix binary not found")]
    NixBinaryNotFound,

    #[error("nix command failed with nonzero status: {0}")]
    NonzeroStatusError(String),
}

impl NixExecutor {
    pub fn new(nix_binary: PathBuf) -> Self {
        Self { nix_binary }
    }

    pub fn from_env() -> Result<Self, NixExecutorError> {
        let nix_binary = std::env::var_os("NIX_BIN_PATH")
            .map(PathBuf::from)
            .or_else(|| which::which("nix").ok())
            .ok_or(NixExecutorError::NixBinaryNotFound)?;

        Ok(Self::new(nix_binary))
    }

    fn nix_command(&self) -> Command {
        let mut cmd = Command::new(&self.nix_binary);
        cmd.args(&[
            "--extra-experimental-features",
            "nix-command",
            "--extra-experimental-features",
            "flakes",
        ]);
        cmd
    }
}

impl NixCmdInterface for NixExecutor {
    type Error = NixExecutorError;

    fn eval_nix_file(&self, path: &PathBuf, to_json: bool) -> Result<String, Self::Error> {
        let path = path.to_str().ok_or(NixExecutorError::NixCmdInterfaceError(
            NixCmdInterfaceError::InvalidPath(path.clone()),
        ))?;

        let mut command = self.nix_command();
        command.arg("eval");
        command.arg("--file").arg(path);
        if to_json {
            command.arg("--json");
        }

        let output = command.output().map_err(|e| {
            NixExecutorError::NixCmdInterfaceError(NixCmdInterfaceError::NixCommandError(
                e.to_string(),
            ))
        })?;

        if !output.status.success() {
            return Err(NixExecutorError::NonzeroStatusError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| {
            NixExecutorError::NixCmdInterfaceError(NixCmdInterfaceError::UTF8ConversionError(
                e.to_string(),
            ))
        })?;

        Ok(stdout.trim().to_string())
    }

    fn store_path_of_flake(&self, flake_uri: &str) -> Result<PathBuf, Self::Error> {
        let mut command = self.nix_command();
        command.args(["build", "--no-link", "--print-out-paths", flake_uri]);

        let output = command.output().map_err(|e| {
            NixExecutorError::NixCmdInterfaceError(NixCmdInterfaceError::NixCommandError(
                e.to_string(),
            ))
        })?;

        if !output.status.success() {
            return Err(NixExecutorError::NonzeroStatusError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| {
            NixExecutorError::NixCmdInterfaceError(NixCmdInterfaceError::UTF8ConversionError(
                e.to_string(),
            ))
        })?;

        Ok(PathBuf::from(stdout.trim()))
    }

    fn nixfmt_file(&self, path: &PathBuf) -> Result<(), Self::Error> {
        let path = path.to_str().ok_or(NixExecutorError::NixCmdInterfaceError(
            NixCmdInterfaceError::InvalidPath(path.clone()),
        ))?;

        let output = Command::new("nixfmt").arg(&path).output().map_err(|e| {
            NixExecutorError::NixCmdInterfaceError(NixCmdInterfaceError::NixCommandError(
                e.to_string(),
            ))
        })?;

        if !output.status.success() {
            return Err(NixExecutorError::NonzeroStatusError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    // Helper function to clean strings for comparison
    fn clean_string(s: &str) -> String {
        s.split_whitespace().collect::<String>()
    }

    mod mock_tests {
        use crate::nix::{NixCmdInterface, NixCmdInterfaceError};
        use std::collections::HashMap;
        use std::path::{Path, PathBuf};

        pub struct MockExecutor {
            eval_results: HashMap<PathBuf, Result<String, NixCmdInterfaceError>>,
            store_paths: HashMap<String, Result<PathBuf, NixCmdInterfaceError>>,
            nixfmt_results: HashMap<PathBuf, Result<(), NixCmdInterfaceError>>,
        }

        impl MockExecutor {
            pub fn new() -> Self {
                Self {
                    eval_results: HashMap::new(),
                    store_paths: HashMap::new(),
                    nixfmt_results: HashMap::new(),
                }
            }

            pub fn mock_eval<P: AsRef<Path>>(
                &mut self,
                path: P,
                result: Result<String, NixCmdInterfaceError>,
            ) {
                self.eval_results
                    .insert(path.as_ref().to_path_buf(), result);
            }

            pub fn mock_store_path(
                &mut self,
                flake_uri: String,
                result: Result<PathBuf, NixCmdInterfaceError>,
            ) {
                self.store_paths.insert(flake_uri, result);
            }

            pub fn mock_nixfmt<P: AsRef<Path>>(
                &mut self,
                path: P,
                result: Result<(), NixCmdInterfaceError>,
            ) {
                self.nixfmt_results
                    .insert(path.as_ref().to_path_buf(), result);
            }
        }

        impl NixCmdInterface for MockExecutor {
            type Error = NixCmdInterfaceError;

            fn eval_nix_file(&self, path: &PathBuf, _to_json: bool) -> Result<String, Self::Error> {
                self.eval_results
                    .get(path)
                    .cloned()
                    .unwrap_or(Err(NixCmdInterfaceError::InvalidPath(path.clone())))
            }

            fn store_path_of_flake(&self, flake_uri: &str) -> Result<PathBuf, Self::Error> {
                self.store_paths.get(flake_uri).cloned().unwrap_or(Err(
                    NixCmdInterfaceError::NixCommandError(format!(
                        "Flake URI not mocked: {}",
                        flake_uri
                    )),
                ))
            }

            fn nixfmt_file(&self, path: &PathBuf) -> Result<(), Self::Error> {
                self.nixfmt_results
                    .get(path)
                    .cloned()
                    .unwrap_or(Err(NixCmdInterfaceError::InvalidPath(path.clone())))
            }
        }
        #[cfg(test)]
        mod tests {
            use super::*;
            use std::path::PathBuf;

            #[test]
            fn test_mock_eval_nix_file_valid() {
                let mut mock = MockExecutor::new();
                let path = PathBuf::from("/test/valid.nix");
                let expected_output = r#"{"description":"Test description","inputs":{"test":{"url":"github:test/repo"}}}"#.to_string();

                mock.mock_eval(&path, Ok(expected_output.clone()));

                let result = mock.eval_nix_file(&path, true).unwrap();
                assert_eq!(result, expected_output);
            }

            #[test]
            fn test_mock_eval_nix_file_error() {
                let mut mock = MockExecutor::new();
                let path = PathBuf::from("/test/error.nix");
                let error_message = "Nix evaluation error".to_string();

                mock.mock_eval(
                    &path,
                    Err(NixCmdInterfaceError::NixCommandError(error_message.clone())),
                );

                let result = mock.eval_nix_file(&path, true);
                assert!(
                    matches!(result, Err(NixCmdInterfaceError::NixCommandError(msg)) if msg == error_message)
                );
            }

            #[test]
            fn test_mock_eval_nix_file_utf8_error() {
                let mut mock = MockExecutor::new();
                let path = PathBuf::from("/test/utf8_error.nix");
                let error_message = "UTF-8 conversion error".to_string();

                mock.mock_eval(
                    &path,
                    Err(NixCmdInterfaceError::UTF8ConversionError(
                        error_message.clone(),
                    )),
                );

                let result = mock.eval_nix_file(&path, true);
                assert!(
                    matches!(result, Err(NixCmdInterfaceError::UTF8ConversionError(msg)) if msg == error_message)
                );
            }

            #[test]
            fn test_mock_eval_nix_file_not_mocked() {
                let mock = MockExecutor::new();
                let path = PathBuf::from("/test/not_mocked.nix");

                let result = mock.eval_nix_file(&path, true);
                assert!(matches!(result, Err(NixCmdInterfaceError::InvalidPath(p)) if p == path));
            }

            #[test]
            fn test_mock_eval_nix_file_multiple_calls() {
                let mut mock = MockExecutor::new();
                let path = PathBuf::from("/test/multiple_calls.nix");
                let expected_output = "Test output".to_string();

                mock.mock_eval(&path, Ok(expected_output.clone()));

                // First call should succeed
                let result1 = mock.eval_nix_file(&path, true).unwrap();
                assert_eq!(result1, expected_output);

                // Second call should also succeed with the same result
                let result2 = mock.eval_nix_file(&path, true).unwrap();
                assert_eq!(result2, expected_output);
            }

            #[test]
            fn test_mock_eval_nix_file_different_paths() {
                let mut mock = MockExecutor::new();
                let path1 = PathBuf::from("/test/path1.nix");
                let path2 = PathBuf::from("/test/path2.nix");
                let output1 = "Output 1".to_string();
                let output2 = "Output 2".to_string();

                mock.mock_eval(&path1, Ok(output1.clone()));
                mock.mock_eval(&path2, Ok(output2.clone()));

                let result1 = mock.eval_nix_file(&path1, true).unwrap();
                let result2 = mock.eval_nix_file(&path2, true).unwrap();

                assert_eq!(result1, output1);
                assert_eq!(result2, output2);
            }

            #[test]
            fn test_mock_eval_nix_file_to_json_ignored() {
                let mut mock = MockExecutor::new();
                let path = PathBuf::from("/test/json_ignored.nix");
                let expected_output = r#"{"key": "value"}"#.to_string();

                mock.mock_eval(&path, Ok(expected_output.clone()));

                // The to_json parameter should be ignored in the mock
                let result_true = mock.eval_nix_file(&path, true).unwrap();
                let result_false = mock.eval_nix_file(&path, false).unwrap();

                assert_eq!(result_true, expected_output);
                assert_eq!(result_false, expected_output);
            }
        }
    }

    mod nix_executor_tests {
        use super::*;

        #[test]
        #[serial(nix_transaction)]
        fn test_valid_nix_file() -> Result<()> {
            let nix_cmd = NixExecutor::from_env()?;
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

            let result = nix_cmd.eval_nix_file(&file_path, true)?;
            let expected = r#"{"description":"Test description","inputs":{"test":{"url":"github:test/repo"}}}"#;

            assert_eq!(clean_string(&result), clean_string(expected));

            Ok(())
        }

        #[test]
        #[serial(nix_transaction)]
        fn test_nonexistent_path() -> Result<()> {
            let nix_cmd = NixExecutor::from_env()?;
            let invalid_path = PathBuf::from("/nonexistent/path");
            let result = nix_cmd.eval_nix_file(&invalid_path, true);
            assert!(matches!(
                result,
                Err(NixExecutorError::NonzeroStatusError(_))
            ));
            Ok(())
        }

        #[test]
        #[serial(nix_transaction)]
        fn test_invalid_path() -> Result<()> {
            let nix_cmd = NixExecutor::from_env()?;
            let invalid_path = PathBuf::from("");
            let result = nix_cmd.eval_nix_file(&invalid_path, true);
            assert!(matches!(
                result,
                Err(NixExecutorError::NonzeroStatusError(_))
            ));
            Ok(())
        }

        #[test]
        #[serial(nix_transaction)]
        fn test_non_json_output() -> Result<()> {
            let nix_cmd = NixExecutor::from_env()?;
            let temp_dir = TempDir::new()?;
            let file_path = temp_dir.path().join("test.nix");
            let mut file = File::create(&file_path)?;
            write!(file, r#""Hello, World!""#)?;

            let result = nix_cmd.eval_nix_file(&file_path, false)?;
            assert_eq!(clean_string(&result), clean_string("\"Hello, World!\""));

            Ok(())
        }

        #[test]
        #[serial(nix_transaction)]
        fn test_complex_nix_file() -> Result<()> {
            let nix_cmd = NixExecutor::from_env()?;
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

            let result = nix_cmd.eval_nix_file(&file_path, true)?;
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
        #[serial(nix_transaction)]
        fn test_nix_command_error() -> Result<()> {
            let nix_cmd = NixExecutor::from_env()?;
            let temp_dir = TempDir::new()?;
            let file_path = temp_dir.path().join("invalid.nix");
            let mut file = File::create(&file_path)?;
            write!(file, "this is not a valid nix expression")?;

            let result = nix_cmd.eval_nix_file(&file_path, true);
            assert!(matches!(
                result,
                Err(NixExecutorError::NonzeroStatusError(_))
            ));

            Ok(())
        }
    }
}
