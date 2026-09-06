use std::{fs, path::PathBuf};

use anyhow::bail;
use toml_span::Deserialize;

use crate::{
    cfg::{Config, DiskKind, FirmwareKind},
    image::Image,
    mbr::{MbrPartition, MbrSector},
};

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

    // create a new disk image
    let mut image = Image::new(&config)?;

    // read the MBR partition just for the sake of it
    let mbr = image.read_lba(0, 1)?;
    println!("mbr: {:?}", mbr);

    // first of all, we need to write the MBR to LBA 0
    // the logic here is dependent on whether we're using MBR or GPT, and
    // BIOS or UEFI.
    //
    // BIOS / MBR -> write boot binary (partitions empty for now, written later)
    // BIOS / GPT -> write boot binary, then protective MBR
    // UEFI -> write protective MBR
    {
        // mbr_sector is dropped at the end of this block; holds a mutable ref to image
        let mut mbr_sector = image.get_mbr();

        match config.boot.0 {
            FirmwareKind::Bios => {
                // firstly, if we're using MBR, we should read the partitions already
                // present so we don't overwrite them blindly
                let mut partitions = Vec::with_capacity(4);

                if config.disk.kind == DiskKind::Mbr {
                    for i in 0..4 {
                        partitions[i as usize] = mbr_sector.read_partition(i);
                    }
                }

                // next we null the sector, write the stage 1 binary, and then
                // rewrite the partitions / protective MBR
            }

            FirmwareKind::Uefi => {
                // we don't really have to worry about overwriting stuff here; we always
                // need to write the binary, and the protective MBR doesn't change.
            }
        }
    }

    Ok(())
}
