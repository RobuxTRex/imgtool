// SPDX-License-Identifier: MIT OR Apache-2.0

use std::env;

use crate::cli::CliFunction;

mod cfg;
mod cli;
mod default;
mod image;
mod scanner;
mod setup;

fn main() -> anyhow::Result<()> {
    // get the current working directory
    let current_directory = env::current_dir()?;
    assert!(
        current_directory.exists(),
        "expected the current working directory to exist, but it doesn't"
    );

    // now get the config path, which should be present in the working directory
    // if not, that's what setup is for!
    let config_path = current_directory.join("image.toml");

    // are we creating a config (setup) or executing the image tool?
    match CliFunction::parse() {
        CliFunction::Default => default::execute(current_directory, config_path),
        CliFunction::Setup => setup::execute(config_path),
    }
}
