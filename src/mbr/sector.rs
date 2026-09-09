use std::io;

use crate::{
    image::{Image, ImageWriteData},
    mbr::MbrPartition,
};

/// The LBA for the MBR sector.
const MBR_LBA: u64 = 0x00;

/// Wrapper for RW operations on LBA 0.
pub(crate) struct MbrSector<'a> {
    img: &'a mut Image,
    cache: Option<Vec<u8>>,
}

impl<'a> MbrSector<'a> {
    /// Creates a new [MbrSector] with a provided [Image] reference.
    #[inline]
    pub fn new(img: &'a mut Image) -> MbrSector<'a> {
        MbrSector { img, cache: None }
    }

    /// Returns a slice of the contents of the cache, reading it
    /// if [None].
    #[inline]
    pub fn contents(&mut self) -> io::Result<&[u8]> {
        if self.cache.is_none() {
            self.read()?;
        }
        Ok(self.cache.as_ref().expect("unreachable").as_ref())
    }

    /// Returns a slice of the contents of the cache, if present.
    #[inline]
    pub fn contents_opt(&self) -> Option<&[u8]> {
        self.cache.as_ref().map(|v| v.as_ref())
    }

    /// Reads the MBR sector from LBA 0 and writes it into the `cache`.
    ///
    /// Callers *must* perform this after any write to ensure the read
    /// methods on this instance are synced.
    pub fn read(&mut self) -> io::Result<()> {
        let buf = self.img.read_lba(MBR_LBA, 1)?;
        self.cache = Some(buf);

        Ok(())
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

        // the partition index is 446 + the partition offset (index * 16)
        let cursor = 446 + ((i as u16) * 16);

        // read the partition
        let buf = self.contents().ok()?;
        MbrPartition::load(&buf, cursor as usize)
    }

    /// Overwrites the first 512 bytes of the MBR sector with the provided
    /// buffer.
    ///
    /// This implicitly updates the cache with the [read](Self::read) method.
    /// 
    /// # Panics
    /// This method panics when `i` is greater than 3.
    pub fn write_mbr(&mut self, buf: &[u8]) -> io::Result<()> {
        assert_eq!(
            buf.len(),
            512,
            "expected the MBR write buffer to be exactly 512 bytes in length, got {}",
            buf.len()
        );

        // write buf to 0..512, pad with 0s
        let mut vec = vec![0u8; self.img.geometry.sector_size as usize];
        vec[..buf.len()].copy_from_slice(buf);
        let buf = vec.as_slice();
        self.write(buf)
    }

    /// Writes 0s to the MBR sector.
    /// 
    /// This implicitly updates the cache with the [read](Self::read) method.
    pub fn write_null(&mut self) -> io::Result<()> {
        let sector_size = self.img.geometry.sector_size;
        let vec = vec![0u8; sector_size as usize];
        let buf = vec.as_slice();
        self.write(buf)
    }

    /// Writes `buf` to the MBR sector.
    /// 
    /// This updates the cache with the [read](Self::read) method.
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<()> {
        let write = ImageWriteData::new(self.img.geometry.sector_size, &buf);
        self.img.write_lba(MBR_LBA, [write])?;
        self.read()
    }
}
