use std::{fs, path::PathBuf};

use anyhow::bail;
use toml_span::Deserialize;

use crate::{
    cfg::{Config, DiskKind, FirmwareKind},
    image::{Image, RW_CHUNK_SIZE},
    load::FileIterator,
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

    // buffer to use for reading binaries
    let mut buf = vec![
        0u8;
        RW_CHUNK_SIZE.try_into().expect(
            "expected the chunk read size to fit into the platform integer limit"
        )
    ];

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
                let mut partitions = vec![None; 4];

                if config.disk.kind == DiskKind::Mbr {
                    for i in 0..4 {
                        partitions[i as usize] = mbr_sector.read_partition(i);
                    }
                }

                // next we null the sector,
                mbr_sector.null()?;

                // then we write the stage 1 binary
                let init_config = &config.init.expect("unreachable"); // always available on BIOS
                let mut bin = FileIterator::load(init_config.0, None, &mut buf)?;

                // confirm the binary is *exactly* 512 bytes long
                let bin_length = bin.file_length()?;
                if bin_length != 0x0200 {
                    panic!(
                        "expected the stage 1 binary to be exactly 512 bytes in length, got {bin_length}"
                    );
                }

                // read the contents of the stage 1 binary
                let bin_read = bin.read()?;
                // sanity: verify for 100% certainty that the binary is 512
                if bin_read != 0x0200 {
                    panic!(
                        "expected the stage 1 binary to be exactly 512 bytes in length, got {bin_read} (invalid meta; {bin_length})"
                    );
                }

                // and finally rewrite the partitions / protective MBR

                // write the resulting sector to sector 0 (0..512 bytes)
                mbr_sector.write(&buf[0..512])?;

                // debug: log result again
                let mbr = image.read_lba(0, 1)?;
                println!("mbr: {:?}", mbr);
            }

            FirmwareKind::Uefi => {
                // we don't really have to worry about overwriting stuff here; we always
                // need to write the binary, and the protective MBR doesn't change.
            }
        }
    }

    Ok(())
}
