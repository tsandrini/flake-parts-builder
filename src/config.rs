pub static FLAKE_TEMPLATE: &'static str = include_str!("assets/flake.nix.template");
pub static FLAKE_INPUTS_TEMPLATE: &'static str = include_str!("assets/flake.nix.template");
pub static META_FILE: &'static str = "meta.nix";
pub static NAMEPLACEHOLDER: &'static str = "NAMEPLACEHOLDER";
pub static BASE_PARTS_DERIVATION: &'static str = "flake-parts";
pub static SELF_FLAKE_URI: &'static str = "."; // TODO only for dev
                                               // static SELF_FLAKE_URI: &'static str = "github:tsandrini/flake-parts-builder";
