use std::{fmt::Display, path::Path};

use anyhow::bail;

/// A binary file with an optional magic byte signature to be appended into
/// the BIOS boot partition.
#[derive(Debug)]
pub struct BiosBinary<'a> {
    /// The path to the binary relative to the current working directory.
    pub path: &'a Path,

    /// An optional magic byte signature to ensure the validity of the
    /// read binary.
    pub magic: Option<&'a str>,
}

/// Whether GPT or MBR is being used on the disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskKind {
    /// The disk uses modern GPT.
    ///
    /// This option is strictly enforced when the [Firmware kind](FirmwareKind) is
    /// [UEFI](FirmwareKind::Uefi).
    ///
    /// On [BIOS](FirmwareKind::Bios) machines, this is optional, but a protective
    /// MBR is still written with the stage 1 of the bootloader being written here.
    Gpt,

    /// The disk uses legacy MBR.
    ///
    /// This limits the disk to 4 primary partitions (or 3 logical partitions with
    /// 1 extended partition), and each partition is limited to 2TiB in capacity.
    ///
    /// This option is only available when the [Firmware kind](FirmwareKind) is
    /// [BIOS](FirmwareKind::Bios).
    Mbr,
}

/// The kind of filesystem a partition is to be formatted with.
// NOTE: this will be eventually moved into the fs crate when implemented
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilesystemKind {
    /// The partition is formatted in FAT32.
    Fat32,

    /// The partition is formatted with no filesystem.
    #[default]
    Raw,
}

/// The kind of firmware the machine is running on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirmwareKind {
    /// The machine is running on legacy BIOS firmware, or is running
    /// in compatibiliy mode.
    ///
    /// This means either GPT or MBR can be used, and a BIOS boot partition
    /// must be written.
    Bios,

    /// The machine is running on modern UEFI firmware.
    ///
    /// This means GPT is assumed, and an ESP must be written.
    Uefi,
}

/// A path to a source file or directory with a corresponding destination
/// path in the kernel filesystem, if any filesystem is present.
#[derive(Debug)]
pub struct KernelEntry<'a> {
    /// The path to the file or directory relative to the current
    /// working directory.
    pub source: &'a Path,

    /// The path to the file or directory to write the source to
    /// in the kernel filesystem, if any.
    pub destination: Option<&'a Path>,
}

impl Display for DiskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gpt => write!(f, "GPT"),
            Self::Mbr => write!(f, "MBR"),
        }
    }
}

impl TryFrom<&str> for DiskKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "gpt" => Ok(Self::Gpt),
            "mbr" => Ok(Self::Mbr),
            _ => bail!("expected either 'gpt' or 'mbr' drive kind, got {value}"),
        }
    }
}

impl Display for FilesystemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fat32 => write!(f, "FAT32"),
            Self::Raw => write!(f, "Raw"),
        }
    }
}

impl TryFrom<&str> for FilesystemKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "fat32" => Ok(Self::Fat32),
            "raw" => Ok(Self::Raw),
            _ => bail!("expected either 'fat32' or 'raw' filesystem kind, got {value}"),
        }
    }
}

impl Display for FirmwareKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bios => write!(f, "BIOS"),
            Self::Uefi => write!(f, "UEFI"),
        }
    }
}

impl TryFrom<&str> for FirmwareKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "bios" => Ok(Self::Bios),
            "uefi" => Ok(Self::Uefi),
            _ => bail!("expected either 'bios' or 'uefi' firmware kind, got {value}"),
        }
    }
}
