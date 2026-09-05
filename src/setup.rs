use std::{fs, path::PathBuf};

/// The template `image.toml` file found in the root of the project.
const TEMPLATE_IMAGE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/image.toml"));

pub(crate) fn execute(cfg: PathBuf) -> anyhow::Result<()> {
    // first, check if the config already exists...
    let exists = cfg.exists();
    // if it already exists, we just alert the user and exit
    if exists {
        println!(
            "You executed the setup command, but the config file already exists in this directory!"
        );
        return Ok(());
    }

    // now we should write the contents of the template file to the path...
    fs::write(cfg, TEMPLATE_IMAGE)?;

    // ... and we're done!
    println!("Successfully written a template config file to the current working directory.");
    Ok(())
}
