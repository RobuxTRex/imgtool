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
    pub fn new(buf: &[u8], pos: usize) -> Option<MbrPartition> {
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
}
