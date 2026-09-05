use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

/// A handle containing [File] which acts as a disk image.
#[derive(Debug)]
pub(crate) struct ImageHandle(pub File);

impl ImageHandle {
    /// Creates an [ImageHandle] to a new image file located at `location`.
    #[inline]
    pub fn create(location: &Path, capacity: u64) -> io::Result<ImageHandle> {
        let image = OpenOptions::new()
            .create(true)
            .read(true)
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
    #[inline]
    pub fn retrieve(location: &Path) -> io::Result<ImageHandle> {
        let image = OpenOptions::new().read(true).write(true).open(location)?;

        Ok(Self(image))
    }

    /// Gets the [File] value within an [ImageHandle].
    #[inline]
    pub fn get(&self) -> &File {
        &self.0
    }

    /// Gets the [File] value within an [ImageHandle] with a mutable reference.
    #[inline]
    pub fn get_mut(&mut self) -> &mut File {
        &mut self.0
    }
}
