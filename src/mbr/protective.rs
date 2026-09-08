use crate::mbr::MbrPartition;

const PROTECTED_PARTITION: MbrPartition = MbrPartition {
    active: true,
    lba: 0x00000001,
    size: 0xFFFFFFFF,
    unit: 0x00,
    kind: 0xEE,
};

const NULL_PARTITION: [u8; 16] = MbrPartition::null();

/// Writes the protective MBR to bytes 446-510 of the provided
/// `buf`.
///
/// Callers should note that this function will **corrupt
/// the MBR**; all partitions are zero'd before the protected
/// partition is written to the first index.
#[inline]
pub fn write_protective(buf: &mut [u8]) {
    // write protective partition
    buf[446..462].copy_from_slice(&PROTECTED_PARTITION.write_partition());

    // write null partitions for the remaining 3
    for i in 1..4 {
        let offset = 446 + (i * 16);
        buf[offset..offset + 16].copy_from_slice(&NULL_PARTITION);
    }
}
