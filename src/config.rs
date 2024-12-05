pub static FLAKE_TEMPLATE: &str = include_str!("assets/flake.nix.template");
pub static FLAKE_INPUTS_TEMPLATE: &str = include_str!("assets/flake-inputs.nix.template");
pub static META_FILE: &str = "meta.nix";
pub static NAMEPLACEHOLDER: &str = "NAMEPLACEHOLDER";
pub static BASE_DERIVATION_NAME: &str = "flake-parts";
pub static BOOTSTRAP_DERIVATION_NAME: &str = "flake-parts-bootstrap";
pub static SELF_FLAKE_URI: &str = "github:tsandrini/flake-parts-builder";
