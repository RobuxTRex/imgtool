/// Retrieves a string value at `path` and converts it to a [Path].
///
/// This returns an error *if* the string is borrowed due to lifetime rules.
///
/// If `true` is evaluated as the 4th argument, the result is an `Option<T>`.
macro_rules! get_path {
    ($table:expr, $value:expr, $key:expr) => {{
        let path: Cow<'de, str> = $table.required($key)?;
        match path {
            Cow::Borrowed(s) => Path::new(s),
            Cow::Owned(_) => {
                return Err(toml_span::Error::from((
                    toml_span::ErrorKind::Custom(Cow::Owned(format!(
                        "the string provided at {} contains escapes, cannot borrow",
                        $key
                    ))),
                    $value.span,
                ))
                .into());
            }
        }
    }};

    ($table:expr, $value:expr, $key:expr, $required:expr) => {{
        let path: Option<Cow<'de, str>> = if $required {
            $table.optional($key)
        } else {
            Some($table.required($key)?)
        };

        match path {
            Some(Cow::Borrowed(s)) => Some(Path::new(s)),
            Some(Cow::Owned(_)) => {
                return Err(toml_span::Error::from((
                    toml_span::ErrorKind::Custom(Cow::Owned(format!(
                        "the string provided at {} contains escapes, cannot borrow",
                        $key
                    ))),
                    $value.span,
                ))
                .into());
            }
            _ => None,
        }
    }};
}

pub(crate) use get_path;

use std::{borrow::Cow, fmt::Display, path::Path};

use anyhow::bail;
use smallvec::SmallVec;
use toml_span::{
    Deserialize, ErrorKind,
    de_helpers::{TableHelper, expected},
    value::ValueInner,
};

/// A binary file with an optional magic byte signature to be appended into
/// the BIOS boot partition.
#[derive(Debug)]
pub struct BiosBinary<'a> {
    /// The path to the binary relative to the current working directory.
    pub path: &'a Path,

    /// An optional magic byte signature to ensure the validity of the
    /// read binary.
    pub magic: Option<Cow<'a, str>>,
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

/// A wrapper for the [SmallVec] type that allows for deserialization.
#[derive(Debug)]
pub struct SmallVecValue<T, const N: usize>(pub SmallVec<[T; N]>)
where
    [T; N]: smallvec::Array<Item = T>;

impl<'de> Deserialize<'de> for BiosBinary<'de> {
    fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
        let mut table = TableHelper::new(value)?;

        let path = get_path!(table, value, "bin");
        let magic = table.optional("magic");

        table.finalize(None)?;

        Ok(Self { path, magic })
    }
}

impl<'de> Deserialize<'de> for DiskKind {
    fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
        let str = value.take_string(Some("expected the disk kind field to be a string"))?;
        Ok(DiskKind::try_from(str).map_err(|e| {
            toml_span::Error::from((ErrorKind::Custom(e.to_string().into()), value.span))
        })?)
    }
}

impl Display for DiskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gpt => write!(f, "GPT"),
            Self::Mbr => write!(f, "MBR"),
        }
    }
}

impl TryFrom<Cow<'_, str>> for DiskKind {
    type Error = anyhow::Error;

    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "gpt" => Ok(Self::Gpt),
            "mbr" => Ok(Self::Mbr),
            _ => bail!("expected either 'gpt' or 'mbr' drive kind, got {value}"),
        }
    }
}

impl<'de> Deserialize<'de> for FilesystemKind {
    fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
        let str = value.take_string(Some("expected the filesystem kind field to be a string"))?;
        Ok(FilesystemKind::try_from(str).map_err(|e| {
            toml_span::Error::from((ErrorKind::Custom(e.to_string().into()), value.span))
        })?)
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

impl TryFrom<Cow<'_, str>> for FilesystemKind {
    type Error = anyhow::Error;

    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "fat32" => Ok(Self::Fat32),
            "raw" => Ok(Self::Raw),
            _ => bail!("expected either 'fat32' or 'raw' filesystem kind, got {value}"),
        }
    }
}

impl<'de> Deserialize<'de> for FirmwareKind {
    fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
        let str = value.take_string(Some("expected the firmware kind field to be a string"))?;
        Ok(FirmwareKind::try_from(str).map_err(|e| {
            toml_span::Error::from((
                toml_span::ErrorKind::Custom(e.to_string().into()),
                value.span,
            ))
        })?)
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

impl TryFrom<Cow<'_, str>> for FirmwareKind {
    type Error = anyhow::Error;

    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "bios" => Ok(Self::Bios),
            "uefi" => Ok(Self::Uefi),
            _ => bail!("expected either 'bios' or 'uefi' firmware kind, got {value}"),
        }
    }
}

impl<'de> Deserialize<'de> for KernelEntry<'de> {
    fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
        let mut table = TableHelper::new(value)?;

        let src = get_path!(table, value, "src");
        let dest = get_path!(table, value, "dest", true);

        table.finalize(None)?;

        Ok(Self {
            source: src,
            destination: dest,
        })
    }
}

impl<'de, T, const N: usize> Deserialize<'de> for SmallVecValue<T, N>
where
    [T; N]: smallvec::Array<Item = T>,
    T: Deserialize<'de>,
{
    fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
        // parse into a vec
        let values = match value.take() {
            ValueInner::Array(arr) => arr,
            other => return Err(expected("an array", other, value.span).into()),
        };

        // convert into a SmallVec
        let mut items = SmallVec::with_capacity(values.len());

        // push items to SmallVec
        for mut item in values {
            items.push(T::deserialize(&mut item)?);
        }

        Ok(Self(items))
    }
}
