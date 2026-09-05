use std::path::Path;

use arrayvec::ArrayVec;

use crate::cfg::{BiosBinary, DiskKind, FilesystemKind, FirmwareKind, KernelEntry};

/// Configuration data structure for imgtool.
#[derive(Debug)]
pub struct Config<'a, const B: usize, const K: usize> {
    pub bios: Option<BiosConfig<'a, B>>,
    pub esp: Option<EspConfig<'a>>,
    pub kernel: KernelConfig<'a, K>,
    pub disk: DiskConfig<'a>,
    pub init: Option<InitConfig<'a>>,
    pub boot: BootConfig,
}

/// Configuration for the BIOS boot partition.
///
/// This field is ignored when the [FirmwareKind] is set to
/// [UEFI](FirmwareKind::Uefi).
#[derive(Debug)]
pub struct BiosConfig<'a, const N: usize> {
    /// The size of the partition in sectors.
    pub size: usize,

    /// The contents of the BIOS boot partition.
    ///
    /// Each field is read and appended to the partition sequentially;
    /// the files must be raw binaries. If the sum of all files' sizes
    /// is greater than the size of the partition, an error is thrown.
    ///
    /// Fields may also contain a magic byte signature that is used
    /// to validate the files.
    pub contents: ArrayVec<BiosBinary<'a>, N>,
}

/// Configuration for the target machine.
#[derive(Debug)]
pub struct BootConfig(pub FirmwareKind);

/// Configuration for the target disk.
#[derive(Debug)]
pub struct DiskConfig<'a> {
    /// The size of a logical sector on the disk, in bytes.
    pub sector_size: usize,

    /// The size of the disk in sectors.
    ///
    /// The logical capacity of the disk, in bytes, can be calculated with
    /// `sector_size * disk_size`.
    pub disk_size: usize,

    /// Whether GPT or MBR is used on this disk.
    pub kind: &'a DiskKind,
}

/// Configuration for the UEFI ESP partition.
///
/// This field is ignored when the [FirmwareKind] is set to
/// [BIOS](FirmwareKind::Bios).
#[derive(Debug)]
pub struct EspConfig<'a> {
    /// The size of the partition in sectors.
    pub size: usize,

    /// The directory to source the EFI files from, relative to the
    /// working directory.
    pub source: &'a Path,
}

/// Configuration for the stage 1 bootloader entry.
///
/// This field is ignored when the [FirmwareKind] is set to
/// [UEFI](FirmwareKind::Uefi).
#[derive(Debug)]
pub struct InitConfig<'a>(pub &'a Path);

/// Configuration for the kernel partition.
#[derive(Debug)]
pub struct KernelConfig<'a, const N: usize> {
    /// The contents of the BIOS boot partition.
    ///
    /// Each field is read and appended to the partition sequentially;
    /// the files must be raw binaries. If the sum of all files' sizes
    /// is greater than the size of the partition, an error is thrown.
    ///
    /// Fields may also contain a magic byte signature that is used
    /// to validate the files.
    pub contents: ArrayVec<KernelEntry<'a>, N>,

    /// The label of the partition, if any filesystem is present.
    pub label: Option<&'a str>,

    /// The filesystem kind, if any.
    pub filesystem: FilesystemKind,
}
