use std::{env, fmt::Display};

/// The subcommand identifier for the setup function.
const SETUP_SUBCOMMAND: &str = "setup";

/// Declares whether the `setup` subcommand was included in the
/// command invocation or not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CliFunction {
    /// Normal function; do the usual stuff...
    #[default]
    Default,

    /// Create a disk image in the current directory.
    Setup,
}

impl CliFunction {
    /// Parses the command line arguments using [env].
    ///
    /// This essentially just ignores every argument except the first, and checks
    /// whether it equals `setup`.
    pub fn parse() -> CliFunction {
        env::args()
            .nth(1)
            .map(|s| s.eq(SETUP_SUBCOMMAND))
            .map_or_else(|| CliFunction::default(), |v| v.into())
    }
}

impl Display for CliFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Setup => write!(f, "setup"),
        }
    }
}

impl From<bool> for CliFunction {
    fn from(value: bool) -> Self {
        if value { Self::Setup } else { Self::Default }
    }
}
