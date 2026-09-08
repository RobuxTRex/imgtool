use crate::scanner::Scanner;

/// A partition entry in the MBR sector.
#[derive(Clone, Copy, Debug)]
pub struct MbrPartition {
    /// The LBA of the absolute first sector in the partition.
    pub lba: u32,

    /// The number of sectors in the partition.
    pub size: u32,

    /// Whether the partition is active or not.
    pub active: bool,

    /// Index of the physical boot unit.
    pub unit: u8,

    /// The type of partition.
    pub kind: u8,
}

impl MbrPartition {
    /// Creates a new [MbrPartition] by reading `16` bytes from `buf` at
    /// `pos` and parsing the result.
    ///
    /// Returns [None] if a read overruns the buffer or the partition is
    /// empty (kind `0x00`).
    // src: https://en.wikipedia.org/wiki/Master_boot_record#PTE
    pub fn load(buf: &[u8], pos: usize) -> Option<MbrPartition> {
        let mut scanner = Scanner::new_with_pos(buf, pos);

        let byte_0 = scanner.read()?;
        // read bit 7
        let active = (byte_0 >> 7) & 0x01 != 0;
        // read bits 0-6
        let unit = byte_0 & ((1 << 7) - 1);

        // skip chs starting address
        scanner.skip::<3>();

        // read partition type
        let kind = scanner.read()?;

        // if the partition kind is 0x00, return None; this is an empty entry
        if kind == 0x00 {
            return None;
        }

        // skip chs ending address
        scanner.skip::<3>();

        // read LBA address
        let lba = u32::from_le_bytes(scanner.read_buffer::<4>()?);

        // read sector count
        let size = u32::from_le_bytes(scanner.read_buffer::<4>()?);

        // construct the result
        Some(MbrPartition {
            lba,
            size,
            active,
            unit,
            kind,
        })
    }

    /// Writes a [MbrPartition] into a stack-allocated 16 byte
    /// buffer.
    // src: https://en.wikipedia.org/wiki/Master_boot_record#PTE
    pub fn write_partition(&self) -> [u8; 16] {
        // buffer to write partition data to
        let mut buf = [0u8; 16];

        // some fields that are annoying
        let active = if self.active { 0u8 } else { 1u8 };

        // supply data from fields to buf
        buf[0] = (self.unit & (1 << 7) - 1) & (active << 7);
        // skip: 1..4
        buf[4] = self.kind;
        // skip: 5..8
        buf[8..12].copy_from_slice(&self.lba.to_le_bytes());
        buf[12..16].copy_from_slice(&self.size.to_le_bytes());

        buf
    }

    /// Writes a null [MbrPartition] into a stack-allocated 16
    /// byte buffer.
    #[inline]
    pub const fn null() -> [u8; 16] {
        [0u8; 16]
    }
}
