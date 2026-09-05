use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

/// A handle containing [File] which acts as a disk image.
pub struct ImageHandle(File);

impl ImageHandle {
    /// Creates an [ImageHandle] to a new image file located at `location`.
    pub fn create(location: &Path, capacity: u64) -> io::Result<ImageHandle> {
        let image = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(location)?;
        image.set_len(capacity)?;

        let filename = location
            .file_name()
            .map(|s| s.to_str().unwrap_or("invalid"))
            .unwrap_or("unknown");
        println!("Successfully written an image file to {filename}!");

        Ok(Self(image))
    }

    /// Provides an [ImageHandle] to an image file located at `location`.
    pub fn get(location: &Path) -> io::Result<ImageHandle> {
        let image = OpenOptions::new().write(true).open(location)?;

        Ok(Self(image))
    }
}
