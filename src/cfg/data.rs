use std::{borrow::Cow, path::Path};

use toml_span::{Deserialize, Value, de_helpers::TableHelper};

use crate::cfg::{
    BiosBinary, DiskKind, FilesystemKind, FirmwareKind, KernelEntry, SmallVecValue, get_path,
};

/// Configuration data structure for imgtool.
#[derive(Debug)]
pub struct Config<'a> {
    pub bios: Option<BiosConfig<'a>>,
    pub esp: Option<EspConfig<'a>>,
    pub kernel: KernelConfig<'a>,
    pub disk: DiskConfig<'a>,
    pub init: Option<InitConfig<'a>>,
    pub boot: BootConfig,
}

/// Configuration for the BIOS boot partition.
///
/// This field is ignored when the [FirmwareKind] is set to
/// [UEFI](FirmwareKind::Uefi).
#[derive(Debug)]
pub struct BiosConfig<'a> {
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
    pub contents: SmallVecValue<BiosBinary<'a>, 4>, // 4 is a conservative upper-limit
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
    pub kind: DiskKind,

    /// The path to the disk image.
    pub location: &'a Path,
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
pub struct KernelConfig<'a> {
    /// The contents of the BIOS boot partition.
    ///
    /// Each field is read and appended to the partition sequentially;
    /// the files must be raw binaries. If the sum of all files' sizes
    /// is greater than the size of the partition, an error is thrown.
    ///
    /// Fields may also contain a magic byte signature that is used
    /// to validate the files.
    pub contents: SmallVecValue<KernelEntry<'a>, 6>, // small to try avoiding bloating the vec

    /// The label of the partition, if any filesystem is present.
    pub label: Option<Cow<'a, str>>,

    /// The filesystem kind, if any.
    pub filesystem: FilesystemKind,
}

impl<'de> Deserialize<'de> for Config<'de> {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, toml_span::DeserError> {
        // acquire a table helper for the root structure
        let mut root = TableHelper::new(value)?;

        // first deserialize the required data
        let boot: BootConfig = root.required("boot")?;
        let disk: DiskConfig = root.required("disk")?;
        let kernel: KernelConfig<'de> = root.required("kernel")?;

        // populate BIOS-only fields
        let (bios, init) = if boot.get() == FirmwareKind::Bios {
            (Some(root.required("bios")?), Some(root.required("init")?))
        } else {
            // parse anyway so we don't get a complaint
            let _: Option<BiosConfig> = root.optional("bios");
            let _: Option<InitConfig> = root.optional("init");

            (None, None)
        };

        // populate UEFI-only fields
        let esp: Option<EspConfig<'de>> = if boot.get() == FirmwareKind::Uefi {
            Some(root.required("esp")?)
        } else {
            // parse anyway so we don't get a complaint
            let _: Option<EspConfig> = root.optional("esp");

            None
        };

        root.finalize(None)?;

        Ok(Self {
            boot,
            disk,
            kernel,
            bios,
            esp,
            init,
        })
    }
}

impl<'de> Deserialize<'de> for BiosConfig<'de> {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, toml_span::DeserError> {
        // get the bios config table
        let mut table = TableHelper::new(value)?;

        // deserialize fields
        let size = table.required("size")?;
        let contents = table.required("contents")?;

        table.finalize(None)?;

        Ok(Self { size, contents })
    }
}

impl BootConfig {
    /// Retrieves the [FirmwareKind] present in this [BootConfig].
    #[inline]
    pub fn get(&self) -> FirmwareKind {
        self.0
    }
}

impl<'de> Deserialize<'de> for BootConfig {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, toml_span::DeserError> {
        // get the boot config table
        let mut table = TableHelper::new(value)?;

        // deserialize fields
        let firmware = table.required("firmware")?;

        table.finalize(None)?;

        Ok(Self(firmware))
    }
}

impl<'de> Deserialize<'de> for DiskConfig<'de> {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, toml_span::DeserError> {
        // get the disk config table
        let mut table = TableHelper::new(value)?;

        // deserialize fields
        let disk_size = table.required("disk_size")?;
        let sector_size = table.required("sector_size")?;
        let kind = table.required("type")?;
        let location = get_path!(table, value, "location");

        table.finalize(None)?;

        Ok(Self {
            sector_size,
            disk_size,
            kind,
            location,
        })
    }
}

impl<'de> Deserialize<'de> for EspConfig<'de> {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, toml_span::DeserError> {
        // get the boot config table
        let mut table = TableHelper::new(value)?;

        // deserialize fields
        let size = table.required("size")?;
        let source = get_path!(table, value, "source");

        table.finalize(None)?;

        Ok(Self { size, source })
    }
}

impl<'de> Deserialize<'de> for InitConfig<'de> {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, toml_span::DeserError> {
        // get the boot config table
        let mut table = TableHelper::new(value)?;

        // deserialize fields
        let path = get_path!(table, value, "source");

        table.finalize(None)?;

        Ok(Self(path))
    }
}

impl<'de> Deserialize<'de> for KernelConfig<'de> {
    fn deserialize(value: &mut Value<'de>) -> Result<Self, toml_span::DeserError> {
        // get the kernel config table
        let mut table = TableHelper::new(value)?;

        // deserialize fields
        let label = table.optional("label");
        let filesystem = table.required("filesystem")?;
        let contents = table.required("contents")?;

        table.finalize(None)?;

        Ok(Self {
            label,
            filesystem,
            contents,
        })
    }
}
