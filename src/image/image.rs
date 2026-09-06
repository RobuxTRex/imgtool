use std::io::{Read, Result, Seek, SeekFrom, Write};

use crate::{cfg::Config, image::ImageHandle, mbr::MbrSector};

/// The amount of bytes that should be read/written each `fs` call.
///
/// Note that this value also asserts the maximum read size of a
/// non-buffered IO operation.
pub const RW_CHUNK_SIZE: u64 = 1024 * 1024;

/// Data structure for an image.
#[derive(Debug)]
pub(crate) struct Image {
    /// The geometry for this disk image.
    pub geometry: ImageGeometry,

    /// Handle for the image file.
    handle: ImageHandle,
}

/// The geometry of a disk image.
#[derive(Debug)]
pub(crate) struct ImageGeometry {
    /// The logical capacity of the image, in bytes.
    ///
    /// This is evaluated by `sectors * sector_size`.
    pub capacity: u64,

    /// The number of sectors in the image.
    pub sectors: u64,

    /// The size, in bytes, of each sector in the image.
    pub sector_size: u64,
}

/// Handle for a data buffer to be written to the image.
///
/// This guarantees the length of the buffer is the sector
/// size.
#[derive(Debug)]
pub(crate) struct ImageWriteData<'a>(&'a [u8]);

impl Image {
    /// Creates a new [Image].
    ///
    /// Note that this *will* write a file to the location provided
    /// in the [Config], relative to your current working directory.
    /// The size of the image is sparse-written to the image file,
    /// which may be unsupported on legacy host filesystems.
    #[inline]
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let geometry = ImageGeometry {
            sectors: config.disk.disk_size as u64,
            sector_size: config.disk.sector_size as u64,
            capacity: (config.disk.disk_size * config.disk.sector_size) as u64,
        };

        Ok(Self {
            handle: ImageHandle::create(config.disk.location, geometry.capacity)?,
            geometry,
        })
    }

    /// Reads sector(s) from the image at an LBA.
    ///
    /// If you are reusing a [Vec], prefer to use [read_lba_into](Self::read_lba_into),
    /// which will overwrite the contents of the provided [Vec] with the read result.
    /// You must ensure the size of the [Vec] can hold the size of the read operation.
    ///
    /// # Panics
    /// This method panics when:
    /// - The size of the read is 0;
    /// - The size of the read is greater than 64KiB;
    /// - The read location is out of bounds;
    /// - The read location + the size of the read is out of bounds.
    pub fn read_lba(&mut self, lba: u64, sectors: u64) -> Result<Vec<u8>> {
        let size = sectors * self.geometry.sector_size;
        // edge: vec doesn't allocate when size == 0, so sectors == 0 is safe
        let mut buf = Vec::with_capacity(
            size.try_into()
                .expect("expected the total read side to fit into the platform integer limit"),
        );
        self.read_lba_into(lba, sectors, &mut buf)?;
        Ok(buf)
    }

    /// Reads `sectors` sectors from the image at an LBA.
    ///
    /// If you are reusing a [Vec], prefer to use [read_lba_into](Self::read_lba_into),
    /// which will overwrite the contents of the provided [Vec] with the read result,
    /// resizing if necessary.
    ///
    /// # Panics
    /// This method panics when:
    /// - The size of the read is greater than 64KiB;
    /// - The read location is out of bounds;
    /// - The read location + the size of the read is out of bounds.
    pub fn read_lba_into(&mut self, lba: u64, sectors: u64, buf: &mut Vec<u8>) -> Result<()> {
        let sector_size = self.geometry.sector_size;
        let size = sectors * sector_size; // calculate the total size in bytes of the read
        let location = lba * sector_size; // calculate the byte location of the read

        assert!(size > 0, "expected the read size to be greater than 0");
        assert!(
            size <= RW_CHUNK_SIZE,
            "expected the total read size to be less than the maximum chunk size, got {size} bytes"
        );
        assert!(
            location < self.geometry.capacity,
            "expected the location to be in range, got location {location} (overflows capacity)"
        );
        assert!(
            location + size < self.geometry.capacity,
            "expected the read amount to be in range, got location {} (overflows capacity)",
            location + size
        );

        buf.resize(size as usize, 0);

        let file = self.handle.get_mut();
        file.seek(SeekFrom::Start(location))?; // seek to the target location

        self.handle.get().read_exact(buf) // read
    }

    pub fn write_lba<'a, const S: usize>(
        &mut self,
        lba: u64,
        data: [ImageWriteData<'a>; S],
    ) -> Result<()> {
        let sector_size = self.geometry.sector_size;
        let location = lba * sector_size; // the byte location of the write
        let size = (S as u64) * sector_size; // the total size of the write

        assert!(
            size <= RW_CHUNK_SIZE,
            "expected the total write size to be less than the maximum chunk size, got {size} bytes"
        );
        assert!(
            location < self.geometry.capacity,
            "expected the location to be in range, got location {location} (overflows capacity)"
        );
        assert!(
            location + size < self.geometry.capacity,
            "expected the read amount to be in range, got location {} (overflows capacity)",
            location + size
        );

        let file = self.handle.get_mut();
        file.seek(SeekFrom::Start(location))?; // seek to the target location

        // loop through the amount of sectors, writing them sequentially
        for sector in 0..S {
            file.write(data[sector].get())?;
        }
        Ok(())
    }

    /// Returns the LBA for the provided byte location.
    /// [None] is returned if the location is out of range.
    #[inline]
    pub fn get_lba(&self, offset: u64) -> Option<u64> {
        let val = offset / self.geometry.sector_size;
        if val > self.geometry.capacity {
            None
        } else {
            Some(val)
        }
    }

    /// Returns a newly constructed [MbrSector] instance
    /// for this [Image].
    ///
    /// Note that this requires a mutable reference to this [Image];
    /// callers are suggested to drop the result of this method after
    /// usage.
    #[inline]
    pub fn get_mbr<'a>(&'a mut self) -> MbrSector<'a> {
        MbrSector::new(self)
    }
}

impl<'a> ImageWriteData<'a> {
    /// Wraps a byte buffer into an [ImageWriteData]
    /// in order to perform a write operation on an [Image].
    ///
    /// # Panics
    /// This function panics when the `ss` (sector size) does not
    /// match the length of the `buf`.
    #[inline]
    pub fn new(ss: u64, buf: &'a &[u8]) -> Self {
        assert_eq!(
            ss,
            buf.len() as u64,
            "expected the write data buffer to be {ss} bytes, got {} bytes",
            buf.len(),
        );

        Self(buf)
    }

    /// Provides a read-only reference to the byte buffer found
    /// in this [ImageWriteData].
    #[inline]
    pub fn get(&self) -> &[u8] {
        self.0
    }
}
