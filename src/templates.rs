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

    pub fn from_merged_metadata(metadata: &Vec<&FlakePartMetadata>) -> Self {
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
        env.add_template("flake-inputs.nix", &FLAKE_INPUTS_TEMPLATE)
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

    pub fn from_merged_metadata(metadata: &Vec<&FlakePartMetadata>) -> Self {
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
        env.add_template("flake.nix", &FLAKE_TEMPLATE).unwrap();
        env.add_template("flake-inputs.nix", &FLAKE_INPUTS_TEMPLATE)
            .unwrap();
        let tmpl = env.get_template("flake.nix").unwrap();
        let rendered = tmpl.render(context! ( context => self))?;
        Ok(rendered)
    }
}
