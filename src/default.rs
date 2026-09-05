use std::{fs, path::PathBuf};

use anyhow::bail;
use toml_span::Deserialize;

use crate::cfg::Config;

pub(crate) fn execute(dir: PathBuf, cfg: PathBuf) -> anyhow::Result<()> {
    // verify the config file exists
    let exists = cfg.exists();
    // if it doesn't exist, bail
    if !exists {
        bail!("You executed the default command but the config file doesn't exist!");
    }

    // read the config
    let config_bytes = fs::read(cfg)?;
    let config_text = str::from_utf8(&config_bytes)?;

    // parse and deserialize the config
    let mut config_value = toml_span::parse(config_text)?;
    let config = Config::deserialize(&mut config_value)?;

    // log the result for now
    println!("{:#?}", config);

    Ok(())
}
