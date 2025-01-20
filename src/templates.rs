use color_eyre::eyre::Result;
use serde::Serialize;
use serde_json::Value as JsonValue;

use minijinja::{context, Environment};

use crate::config::{FLAKE_INPUTS_TEMPLATE, FLAKE_TEMPLATE};
use crate::parts::FlakePartMetadata;

#[derive(Debug, Serialize)]
pub struct FlakeInputsContext {
    pub inputs: JsonValue,
}

impl FlakeInputsContext {
    fn new(inputs: JsonValue) -> Self {
        Self { inputs }
    }

    pub fn from_merged_metadata(metadata: &[&FlakePartMetadata]) -> Self {
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

        Self::new(inputs)
    }

    pub fn render(&self) -> Result<String> {
        let mut env = Environment::new();
        env.add_template("flake-inputs.nix", FLAKE_INPUTS_TEMPLATE)
            .unwrap();
        let tmpl = env.get_template("flake-inputs.nix").unwrap();
        let rendered = tmpl.render(context! ( context => self))?;
        Ok(rendered)
    }
}

#[derive(Debug, Serialize)]
pub struct FlakeContext {
    pub flake_inputs_context: FlakeInputsContext,
    pub extra_trusted_public_keys: Vec<String>,
    pub extra_substituters: Vec<String>,
}

impl FlakeContext {
    fn new(
        flake_inputs_context: FlakeInputsContext,
        extra_trusted_public_keys: Vec<String>,
        extra_substituters: Vec<String>,
    ) -> Self {
        Self {
            flake_inputs_context,
            extra_trusted_public_keys,
            extra_substituters,
        }
    }

    pub fn from_merged_metadata(metadata: &[&FlakePartMetadata]) -> Self {
        let flake_inputs_context = FlakeInputsContext::from_merged_metadata(metadata);

        let extra_trusted_public_keys = metadata
            .iter()
            .flat_map(|m| m.extra_trusted_public_keys.iter().cloned())
            .collect::<Vec<String>>();

        let extra_substituters = metadata
            .iter()
            .flat_map(|m| m.extra_substituters.iter().cloned())
            .collect::<Vec<String>>();

        Self::new(
            flake_inputs_context,
            extra_trusted_public_keys,
            extra_substituters,
        )
    }

    pub fn render(&self) -> Result<String> {
        let mut env = Environment::new();
        env.add_template("flake.nix", FLAKE_TEMPLATE).unwrap();
        env.add_template("flake-inputs.nix", FLAKE_INPUTS_TEMPLATE)
            .unwrap();
        let tmpl = env.get_template("flake.nix").unwrap();
        let rendered = tmpl.render(context! ( context => self))?;
        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_flake_inputs_context_new() {
        let inputs = json!({"input1": "value1", "input2": "value2"});
        let context = FlakeInputsContext::new(inputs.clone());
        assert_eq!(context.inputs, inputs);
    }

    #[test]
    fn test_flake_inputs_context_from_merged_metadata() {
        let metadata1 = FlakePartMetadata {
            description: "Metadata 1".to_string(),
            inputs: json!({"input1": "value1"}),
            dependencies: vec![],
            conflicts: vec![],
            extra_trusted_public_keys: vec![],
            extra_substituters: vec![],
        };
        let metadata2 = FlakePartMetadata {
            description: "Metadata 2".to_string(),
            inputs: json!({"input2": "value2"}),
            dependencies: vec![],
            conflicts: vec![],
            extra_trusted_public_keys: vec![],
            extra_substituters: vec![],
        };
        let metadata = vec![&metadata1, &metadata2];

        let context = FlakeInputsContext::from_merged_metadata(&metadata);
        assert_eq!(
            context.inputs,
            json!({"input1": "value1", "input2": "value2"})
        );
    }

    #[test]
    fn test_flake_inputs_context_render_with_simple_inputs() -> Result<()> {
        let inputs = json!({"input1": {
            "url": "github:org1/repo1",
        }, "input2": {
            "url": "github:org2/repo2",
        }});
        let context = FlakeInputsContext::new(inputs);
        let rendered = context.render()?;
        let cleaned_rendered = rendered.split_whitespace().collect::<String>();

        let expected = r#"
          input1.url = "github:org1/repo1";
          input2.url = "github:org2/repo2";
        "#;

        let cleaned_expected = expected.split_whitespace().collect::<String>();

        assert_eq!(cleaned_rendered, cleaned_expected);
        Ok(())
    }

    #[test]
    fn test_flake_inputs_context_render_with_complex_inputs() -> Result<()> {
        let inputs = json!({"input1": {
            "url": "github:org1/repo1",
            "flake": false
        }, "input2": {
            "url": "github:org2/repo2",
            "inputs": {
                "input1": {
                    "follows": "input1"
                }
            }
        }});
        let context = FlakeInputsContext::new(inputs);
        let rendered = context.render()?;
        let cleaned_rendered = rendered.split_whitespace().collect::<String>();

        let expected = r#"
          input1 = {
            url = "github:org1/repo1";
            flake = false;
          };
          input2 = {
            url = "github:org2/repo2";
            inputs.input1.follows = "input1";
          };
        "#;

        let cleaned_expected = expected.split_whitespace().collect::<String>();

        assert_eq!(cleaned_rendered, cleaned_expected);
        Ok(())
    }

    #[test]
    fn test_flake_context_new() {
        let inputs_context = FlakeInputsContext::new(json!({}));
        let trusted_keys = vec!["key1".to_string(), "key2".to_string()];
        let substituters = vec!["sub1".to_string(), "sub2".to_string()];
        let context = FlakeContext::new(inputs_context, trusted_keys.clone(), substituters.clone());

        assert_eq!(context.extra_trusted_public_keys, trusted_keys);
        assert_eq!(context.extra_substituters, substituters);
    }

    #[test]
    fn test_flake_context_from_merged_metadata() {
        let metadata1 = FlakePartMetadata {
            description: "Metadata 1".to_string(),
            inputs: json!({"input1": "value1"}),
            dependencies: vec![],
            conflicts: vec![],
            extra_trusted_public_keys: vec!["key1".to_string()],
            extra_substituters: vec!["sub1".to_string()],
        };
        let metadata2 = FlakePartMetadata {
            description: "Metadata 2".to_string(),
            inputs: json!({"input2": "value2"}),
            dependencies: vec![],
            conflicts: vec![],
            extra_trusted_public_keys: vec!["key2".to_string()],
            extra_substituters: vec!["sub2".to_string()],
        };
        let metadata = vec![&metadata1, &metadata2];

        let context = FlakeContext::from_merged_metadata(&metadata);
        assert_eq!(
            context.flake_inputs_context.inputs,
            json!({"input1": "value1", "input2": "value2"})
        );
        assert_eq!(
            context.extra_trusted_public_keys,
            vec!["key1".to_string(), "key2".to_string()]
        );
        assert_eq!(
            context.extra_substituters,
            vec!["sub1".to_string(), "sub2".to_string()]
        );
    }

    #[test]
    fn test_flake_context_render() -> Result<()> {
        let inputs_context = FlakeInputsContext::new(json!({"input1": {
            "url": "github:org1/repo1",
        }, "input2": {
            "url": "github:org2/repo2",
        }}));
        let trusted_keys = vec!["key1".to_string(), "key2".to_string()];
        let substituters = vec!["sub1".to_string(), "sub2".to_string()];
        let context = FlakeContext::new(inputs_context, trusted_keys, substituters);

        let rendered = context.render()?;
        let cleaned_rendered = rendered.split_whitespace().collect::<String>();

        let expected = r#"
          # --- flake.nix
          {
            description = "TODO Add description of your new project";

            inputs = {
              # --- BASE DEPENDENCIES ---
              nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
              flake-parts.url = "github:hercules-ci/flake-parts";

              # --- YOUR DEPENDENCIES ---
              input1.url = "github:org1/repo1";
              input2.url = "github:org2/repo2";
            };

            # NOTE Here you can add additional binary cache substituers that you trust.
            # There are also some sensible default caches commented out that you
            # might consider using, however, you are advised to doublecheck the keys.
            nixConfig = {
              extra-trusted-public-keys = [
                # "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
                # "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
                # "key1"
                # "key2"
                ];
              extra-substituters = [
                # "https://cache.nixos.org"
                # "https://nix-community.cachix.org/"
                # "sub1"
                # "sub2"
                ];
            };

            outputs =
              inputs@{ flake-parts, ... }:
              let
                inherit (inputs.nixpkgs) lib;
                inherit (import ./flake-parts/_bootstrap.nix { inherit lib; }) loadParts;
              in
              flake-parts.lib.mkFlake { inherit inputs; } {

                # We recursively traverse all of the flakeModules in ./flake-parts and
                # import only the final modules, meaning that you can have an arbitrary
                # nested structure that suffices your needs. For example
                #
                # - ./flake-parts
                #   - modules/
                #     - nixos/
                #       - myNixosModule1.nix
                #       - myNixosModule2.nix
                #       - default.nix
                #     - home-manager/
                #       - myHomeModule1.nix
                #       - myHomeModule2.nix
                #       - default.nix
                #     - sharedModules.nix
                #   - pkgs/
                #     - myPackage1.nix
                #     - myPackage2.nix
                #     - default.nix
                #   - mySimpleModule.nix
                #   - _not_a_module.nix
                imports = loadParts ./flake-parts;
              };
          }
        "#;

        let cleaned_expected = expected.split_whitespace().collect::<String>();

        assert_eq!(cleaned_rendered, cleaned_expected);
        Ok(())
    }
}
