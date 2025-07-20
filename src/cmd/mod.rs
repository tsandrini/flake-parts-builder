use clap::Args;

pub mod add;
pub mod init;
pub mod list;

#[derive(Debug, Args)]
pub struct SharedArgs {
    /// Additional parts templates stores to load. This currently accepts any
    /// valid flakes derivation URI. For example:
    ///
    /// - `github:tsandrini/flake-parts-builder#flake-parts`
    /// - `../myDir#flake-parts`
    /// - `.#different-flake-parts`
    ///
    /// NOTE: the derivation needs to have the parts stored at
    /// `$out/flake-parts`. You can also use `lib.mkFlakeParts` defined
    /// in `flake.nix` to make this easier.
    #[arg(
        short = 'I',
        long = "include",
        value_delimiter = ',',
        verbatim_doc_comment
    )]
    pub parts_stores: Vec<String>,

    /// Disable base parts provided by this flake, that is,
    /// `github:tsandrini/flake-parts-builder#flake-parts`. Useful in case
    /// you'd like to override certain parts or simply not use the one provided
    /// by this repo.
    ///
    /// NOTE: _bootstrap part is always included for the project to
    /// properly function (if you really need to you can override the files
    /// with your own versions)
    #[arg(long = "disable-base", default_value_t = false, verbatim_doc_comment)]
    pub disable_base_parts: bool,

    /// Write flake-parts-meta/<part>.nix file.
    /// 
    /// This is useful for other Nix tools that can process the meta file.
    /// 
    /// For example, `github:vic/flake-file` can update your flake inputs from them.
    /// See https://github.com/vic/flake-file#parts_templates
    #[arg(long = "write-meta", default_value_t = false, verbatim_doc_comment)]
    pub write_meta: bool,

    /// Enable verbose logging
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
