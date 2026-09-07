use std::{
    fs::{File, OpenOptions},
    io::{Read, Result, Seek, SeekFrom},
    path::Path,
};

use crate::image::RW_CHUNK_SIZE;

pub struct FileIterator<'a> {
    chunk_size: u64,
    length: u64,
    cursor: u64,
    buf: &'a mut Vec<u8>,
    file: File,
}

impl<'a> FileIterator<'a> {
    /// Loads a file at `path` and provides a [FileIterator] that
    /// can be used to perform a buffered read on the file.
    ///
    /// # Panics
    /// If the buffer is smaller than the sector size, then the function will panic.
    pub fn load(
        path: &Path,
        chunk_size: Option<u64>,
        buf: &'a mut Vec<u8>,
    ) -> Result<FileIterator<'a>> {
        let chunk_size = chunk_size.unwrap_or(RW_CHUNK_SIZE);

        // open the file at path in read-only mode
        let file = OpenOptions::new().read(true).open(path)?;

        // determine the file size
        let size = file.metadata()?.len();
        let length = size.div_ceil(chunk_size);

        // ensure safety
        let buffer_length = buf.len() as u64;
        assert!(
            buffer_length >= chunk_size,
            "expected buffer length to be greater than or equal to sector size, got {buffer_length} vs {chunk_size}"
        );

        Ok(FileIterator {
            chunk_size,
            length: length as u64,
            cursor: 0,
            buf,
            file,
        })
    }

    /// Reads the next sector from this iterator, loading the result into
    /// the instance's buffer. The value returned is the number of bytes
    /// read; this should always be the chunk size unless EOF was reached.
    pub fn read(&mut self) -> Result<usize> {
        // seek to starting position
        let pos = self.cursor * self.chunk_size;
        self.file.seek(SeekFrom::Start(pos))?;

        self.buf.fill(0);

        // read the chunk size into the buffer, returning the total number of bytes read
        let mut chunk = (&self.file).take(self.chunk_size);
        let res = chunk.read(&mut self.buf[..self.chunk_size as usize])?;

        self.cursor += 1;

        Ok(res)
    }

    /// Provides the byte length of the enclosed file.
    #[inline]
    pub fn file_length(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }

    /// Provides the length of this iterator in sectors.
    #[inline]
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Provides the number of sectors remaining in this iterator read.
    #[inline]
    pub fn remaining(&self) -> u64 {
        self.length - self.cursor
    }
}
