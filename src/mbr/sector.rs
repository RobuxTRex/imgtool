use std::io;

use crate::{
    image::{Image, ImageWriteData},
    mbr::MbrPartition,
};

/// The LBA for the MBR sector.
const MBR_LBA: u64 = 0x00;

/// Wrapper for RW operations on LBA 0.
pub(crate) struct MbrSector<'a>(&'a mut Image);

impl<'a> MbrSector<'a> {
    /// Creates a new [MbrSector] with a provided [Image] reference.
    #[inline]
    pub fn new(img: &'a mut Image) -> MbrSector<'a> {
        MbrSector(img)
    }

    /// Writes 0s to the MBR sector.
    pub fn null(&mut self) -> io::Result<()> {
        let sector_size = self.0.geometry.sector_size;
        let vec = vec![0u8; sector_size as usize];
        let buf = vec.as_slice();

        let write = ImageWriteData::new(sector_size, &buf);
        self.0.write_lba(MBR_LBA, [write])
    }

    /// Reads partition `i` and returns the resulting [MbrPartition],
    /// if any. Note that `i` is zero-indexed.
    ///
    /// # Panics
    /// This method panics when `i` is greater than 3.
    pub fn read_partition(&mut self, i: u8) -> Option<MbrPartition> {
        assert!(
            i < 4,
            "expected the partition index to be less than 4, got {i}"
        );

        let buf = self.0.read_lba(MBR_LBA, 1).ok()?;

        // the partition index is 440 + the partition offset (index * 16)
        let cursor = 440 + ((i as u16) * 16);

        // read the partition, mapping it to None if it couldn't be read
        MbrPartition::new(&buf, cursor)
        /*
        .map(|p| Some(p))
        .unwrap_or_else(|| {
            println!("An error occurred whilst reading partition {i}: {e}");
            None
        })*/
    }
}
