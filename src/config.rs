pub static FLAKE_TEMPLATE: &'static str = include_str!("assets/flake.nix.template");
pub static FLAKE_INPUTS_TEMPLATE: &'static str = include_str!("assets/flake-inputs.nix.template");
pub static META_FILE: &'static str = "meta.nix";
pub static NAMEPLACEHOLDER: &'static str = "NAMEPLACEHOLDER";
pub static BASE_DERIVATION_NAME: &'static str = "flake-parts";
pub static BOOTSTRAP_DERIVATION_NAME: &'static str = "flake-parts-bootstrap";
pub static SELF_FLAKE_URI: &'static str = "github:tsandrini/flake-parts-builder/v1";
