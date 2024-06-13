use serde_json::json;
use serde_json::{Result, Value};
use std::path::{Path, PathBuf};

use clap::{arg, command, value_parser, ArgAction, ArgGroup};

struct PartVendor {
    name: String,
    identifier: String,
    store_path: PathBuf,
}

struct PartInfo {
    name: String,
    vendor: PartVendor,
    metadata: Value,
}

fn main() {
    let matches = command!()
        .arg(arg!([parts] "Flake parts that should be used"))
        .arg(
            arg!(
                -I --include <PATHS> "Additional flake parts stores"
            )
            .required(false)
            .num_args(1..)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                -N --name <NAME> "Name for the newly created flake"
            )
            .required(true)
            .num_args(1),
        )
        .arg(arg!(
            -v --verbose ... "Turn on verbose logging"
        ))
        .arg(arg!(
            -d --disable-base ... "Disable parts provided by this flake"
        ))
        .get_matches();

    println!("Hello, world!");
}
