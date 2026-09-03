// SPDX-License-Identifier: MIT OR Apache-2.0

use clap::Parser;

use crate::cli::Cli;

mod cli;

fn main() {
    // parse CLI arguments
    let args = Cli::parse();

    println!("{:#?}", args);
}
