//! SD Card emulation using memory-mapped files
//!
//! Provides fast, persistent SD card storage backed by a host file.

use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use anyhow::{Result, Context};

const SECTOR_SIZE: usize = 512;

/// Memory-mapped SD card
pub struct SdCard {
    mmap: MmapMut,
    size_bytes: u64,
    path: String,
}

impl SdCard {
    /// Create or open an SD card image file
    ///
    /// If the file doesn't exist, it will be created with the specified size.
    /// If it exists, size_bytes is ignored (uses existing file size).
    pub fn new<P: AsRef<Path>>(path: P, size_bytes: u64) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .context("Failed to open SD card image")?;

        // Get existing size or set to requested size
        let metadata = file.metadata()?;
        let actual_size = if metadata.len() == 0 {
            file.set_len(size_bytes)?;
            size_bytes
        } else {
            metadata.len()
        };

        // Memory-map the file
        let mmap = unsafe {
            MmapMut::map_mut(&file)
                .context("Failed to memory-map SD card image")?
        };

        Ok(Self {
            mmap,
            size_bytes: actual_size,
            path: path_str,
        })
    }

    /// Read a sector (512 bytes) at the given LBA
    pub fn read_sector(&self, lba: u32, data: &mut [u8]) -> io::Result<()> {
        if data.len() != SECTOR_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Buffer must be {} bytes", SECTOR_SIZE),
            ));
        }

        let offset = lba as usize * SECTOR_SIZE;
        if offset + SECTOR_SIZE > self.mmap.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Read beyond SD card size",
            ));
        }

        data.copy_from_slice(&self.mmap[offset..offset + SECTOR_SIZE]);
        Ok(())
    }

    /// Write a sector (512 bytes) at the given LBA
    pub fn write_sector(&mut self, lba: u32, data: &[u8]) -> io::Result<()> {
        if data.len() != SECTOR_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Buffer must be {} bytes", SECTOR_SIZE),
            ));
        }

        let offset = lba as usize * SECTOR_SIZE;
        if offset + SECTOR_SIZE > self.mmap.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Write beyond SD card size",
            ));
        }

        self.mmap[offset..offset + SECTOR_SIZE].copy_from_slice(data);
        Ok(())
    }

    /// Read multiple sectors
    pub fn read_sectors(&self, lba: u32, count: u32, data: &mut [u8]) -> io::Result<()> {
        let expected_size = count as usize * SECTOR_SIZE;
        if data.len() < expected_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Buffer too small",
            ));
        }

        for i in 0..count {
            let offset = i as usize * SECTOR_SIZE;
            self.read_sector(lba + i, &mut data[offset..offset + SECTOR_SIZE])?;
        }

        Ok(())
    }

    /// Write multiple sectors
    pub fn write_sectors(&mut self, lba: u32, count: u32, data: &[u8]) -> io::Result<()> {
        let expected_size = count as usize * SECTOR_SIZE;
        if data.len() < expected_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Buffer too small",
            ));
        }

        for i in 0..count {
            let offset = i as usize * SECTOR_SIZE;
            self.write_sector(lba + i, &data[offset..offset + SECTOR_SIZE])?;
        }

        Ok(())
    }

    /// Flush changes to disk
    pub fn flush(&self) -> io::Result<()> {
        self.mmap.flush()
    }

    /// Get SD card size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// Get SD card size in sectors
    pub fn size_sectors(&self) -> u64 {
        self.size_bytes / SECTOR_SIZE as u64
    }

    /// Get the file path
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for SdCard {
    fn drop(&mut self) {
        // Flush on drop to ensure all writes are persisted
        self.flush().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sdcard_create() {
        let path = "test_sdcard.img";
        let size = 1024 * 1024; // 1 MB

        // Clean up if exists
        fs::remove_file(path).ok();

        let sdcard = SdCard::new(path, size).unwrap();
        assert_eq!(sdcard.size_bytes(), size);
        assert_eq!(sdcard.size_sectors(), size / 512);

        // Clean up
        drop(sdcard);
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_sdcard_read_write() {
        let path = "test_sdcard_rw.img";
        let size = 1024 * 1024;

        fs::remove_file(path).ok();

        let mut sdcard = SdCard::new(path, size).unwrap();

        // Write a sector
        let mut write_data = [0u8; 512];
        write_data[0] = 0xAA;
        write_data[511] = 0x55;
        sdcard.write_sector(0, &write_data).unwrap();

        // Read it back
        let mut read_data = [0u8; 512];
        sdcard.read_sector(0, &mut read_data).unwrap();

        assert_eq!(read_data[0], 0xAA);
        assert_eq!(read_data[511], 0x55);

        drop(sdcard);
        fs::remove_file(path).ok();
    }
}
