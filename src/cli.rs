use clap_derive::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Create(CreateArgs),
}

#[derive(Args, Debug)]
pub(crate) struct CreateArgs {}
