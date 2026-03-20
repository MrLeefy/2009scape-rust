//! RS2 Cache file reader.
//!
//! The 2009Scape cache uses idx/dat2 file format:
//! - main_file_cache.dat2: bulk data file
//! - main_file_cache.idx0..N: index files pointing into dat2
//! Each index entry is 6 bytes: 3 bytes size + 3 bytes sector offset.

pub mod loader;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use log::info;

const SECTOR_SIZE: usize = 520;
const INDEX_ENTRY_SIZE: usize = 6;
const SECTOR_HEADER_SIZE: usize = 8;
const SECTOR_DATA_SIZE: usize = SECTOR_SIZE - SECTOR_HEADER_SIZE;
const SECTOR_HEADER_SIZE_EXT: usize = 10; // for archive IDs >= 65536
const SECTOR_DATA_SIZE_EXT: usize = SECTOR_SIZE - SECTOR_HEADER_SIZE_EXT;

pub struct CacheReader {
    data_file: File,
    index_files: Vec<Option<File>>,
    cache_dir: PathBuf,
}

impl CacheReader {
    /// Open a cache directory containing main_file_cache.dat2 and .idx files.
    pub fn open(path: &Path) -> Result<Self> {
        let data_path = path.join("main_file_cache.dat2");
        let data_file = File::open(&data_path)
            .with_context(|| format!("Cannot open cache data file: {:?}", data_path))?;

        let mut index_files = Vec::new();
        for i in 0..256 {
            let idx_path = path.join(format!("main_file_cache.idx{}", i));
            if idx_path.exists() {
                index_files.push(Some(
                    File::open(&idx_path)
                        .with_context(|| format!("Cannot open index file: {:?}", idx_path))?
                ));
            } else {
                index_files.push(None);
            }
        }

        let count = index_files.iter().filter(|f| f.is_some()).count();
        info!("Cache opened: {:?} ({} index files)", path, count);

        Ok(CacheReader {
            data_file,
            index_files,
            cache_dir: path.to_path_buf(),
        })
    }

    /// Get the number of available archives (index files).
    pub fn archive_count(&self) -> usize {
        self.index_files.iter().filter(|f| f.is_some()).count()
    }

    /// Read a file from the cache.
    /// `archive`: index number (0-255)
    /// `group`: file/group ID within the archive
    pub fn read(&mut self, archive: u8, group: u32) -> Result<Vec<u8>> {
        let idx_file = self.index_files[archive as usize]
            .as_mut()
            .context("Archive index not found")?;

        // Read the index entry (6 bytes)
        let entry_offset = group as u64 * INDEX_ENTRY_SIZE as u64;
        idx_file.seek(SeekFrom::Start(entry_offset))?;
        let mut idx_buf = [0u8; INDEX_ENTRY_SIZE];
        idx_file.read_exact(&mut idx_buf)?;

        let size = ((idx_buf[0] as usize) << 16)
            | ((idx_buf[1] as usize) << 8)
            | (idx_buf[2] as usize);
        let mut sector = ((idx_buf[3] as u64) << 16)
            | ((idx_buf[4] as u64) << 8)
            | (idx_buf[5] as u64);

        if size == 0 || sector == 0 {
            anyhow::bail!("Empty cache entry: archive={}, group={}", archive, group);
        }

        let mut result = Vec::with_capacity(size);
        let mut remaining = size;
        let mut part = 0u16;
        let extended = group >= 65536;

        while remaining > 0 {
            let sector_offset = sector * SECTOR_SIZE as u64;
            self.data_file.seek(SeekFrom::Start(sector_offset))?;

            let mut sector_buf = [0u8; SECTOR_SIZE];
            self.data_file.read_exact(&mut sector_buf)?;

            let (header_size, data_size) = if extended {
                (SECTOR_HEADER_SIZE_EXT, SECTOR_DATA_SIZE_EXT)
            } else {
                (SECTOR_HEADER_SIZE, SECTOR_DATA_SIZE)
            };

            // Parse sector header
            let (entry_id, entry_part, next_sector, entry_archive) = if extended {
                let id = ((sector_buf[0] as u32) << 24)
                    | ((sector_buf[1] as u32) << 16)
                    | ((sector_buf[2] as u32) << 8)
                    | (sector_buf[3] as u32);
                let p = ((sector_buf[4] as u16) << 8) | (sector_buf[5] as u16);
                let ns = ((sector_buf[6] as u64) << 16)
                    | ((sector_buf[7] as u64) << 8)
                    | (sector_buf[8] as u64);
                let a = sector_buf[9];
                (id, p, ns, a)
            } else {
                let id = ((sector_buf[0] as u32) << 8) | (sector_buf[1] as u32);
                let p = ((sector_buf[2] as u16) << 8) | (sector_buf[3] as u16);
                let ns = ((sector_buf[4] as u64) << 16)
                    | ((sector_buf[5] as u64) << 8)
                    | (sector_buf[6] as u64);
                let a = sector_buf[7];
                (id, p, ns, a)
            };

            // Validate
            if entry_id != group {
                anyhow::bail!("Cache corruption: expected group {}, got {}", group, entry_id);
            }
            if entry_part != part {
                anyhow::bail!("Cache corruption: expected part {}, got {}", part, entry_part);
            }
            if entry_archive != archive {
                anyhow::bail!("Cache corruption: expected archive {}, got {}", archive, entry_archive);
            }

            let bytes_to_read = remaining.min(data_size);
            result.extend_from_slice(&sector_buf[header_size..header_size + bytes_to_read]);

            remaining -= bytes_to_read;
            sector = next_sector;
            part += 1;
        }

        Ok(result)
    }

    /// Get the number of groups (files) in an archive.
    pub fn group_count(&mut self, archive: u8) -> Result<u32> {
        let idx_file = self.index_files[archive as usize]
            .as_mut()
            .context("Archive index not found")?;
        let len = idx_file.seek(SeekFrom::End(0))?;
        Ok((len / INDEX_ENTRY_SIZE as u64) as u32)
    }
}
