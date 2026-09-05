// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::CliFunction;

mod cli;

fn main() {
    // are we creating a config (setup) or executing the image tool?
    let function = CliFunction::parse();

    println!("{}", function);
}
