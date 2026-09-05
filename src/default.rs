use std::{fs, path::PathBuf};

use anyhow::bail;
use toml_span::Deserialize;

use crate::{cfg::Config, image::handle::ImageHandle};

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

    // retrieve the disk image, or write it if it doesn't already exist
    let image_location = config.disk.location;
    let image = if fs::exists(image_location)? {
        ImageHandle::get(image_location)
    } else {
        ImageHandle::create(
            image_location,
            (config.disk.sector_size * config.disk.disk_size) as u64,
        )
    }?;

    Ok(())
}
