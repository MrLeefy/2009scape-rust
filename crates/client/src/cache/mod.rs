//! JS5 cache system — reads real RuneScape cache files (idx/dat2).
//!
//! Translated from rt4/Cache.java, Js5Index.java, Js5Compression.java.
//!
//! The RS2 cache consists of:
//! - `main_file_cache.dat2` — main data file containing all cached data
//! - `main_file_cache.idx0` through `main_file_cache.idx28` — index files
//! - `main_file_cache.idx255` — master index (reference table)
//!
//! Each index points to groups of files. Groups are compressed with:
//! - Type 0: No compression (raw)
//! - Type 1: BZIP2
//! - Type 2: GZIP

pub mod loader;

use rs2_common::buffer::Buffer;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use anyhow::{Result, bail, Context};
use flate2::read::GzDecoder;
use log::{info, warn, debug};

/// Sector size in the data file (520 bytes).
const SECTOR_SIZE: usize = 520;
/// Header size within a sector for normal (small) groups.
const SECTOR_HEADER: usize = 8;
/// Data payload per sector for normal groups.
const SECTOR_DATA: usize = 512;
/// Header size for extended (large group ID) sectors.
const SECTOR_HEADER_EXT: usize = 10;
/// Data payload per sector for extended groups.
const SECTOR_DATA_EXT: usize = 510;

/// A complete JS5 cache system backed by disk files.
pub struct Js5Cache {
    pub data_file: Vec<u8>,
    pub indices: HashMap<u8, Vec<u8>>,
    pub parsed_indices: HashMap<u8, Js5Index>,
    pub cache_dir: String,
}

/// Parsed JS5 index (reference table) for one archive.
#[derive(Debug, Clone)]
pub struct Js5Index {
    pub version: i32,
    pub size: usize,
    pub group_ids: Vec<u16>,
    pub capacity: usize,
    pub group_checksums: Vec<i32>,
    pub group_versions: Vec<i32>,
    pub group_sizes: Vec<u16>,
    pub group_capacities: Vec<u16>,
    pub file_ids: Vec<Option<Vec<u16>>>,
    pub group_name_hashes: Option<Vec<i32>>,
}

/// A decompressed file from the cache.
#[derive(Debug, Clone)]
pub struct CacheFile {
    pub data: Vec<u8>,
}

impl Js5Cache {
    /// Open a cache from a directory containing main_file_cache.dat2 and .idx* files.
    pub fn open(cache_dir: &str) -> Result<Self> {
        let path = Path::new(cache_dir);

        // Read the main data file
        let dat_path = path.join("main_file_cache.dat2");
        let data_file = if dat_path.exists() {
            info!("Loading cache data file: {} ({} MB)", dat_path.display(),
                fs::metadata(&dat_path).map(|m| m.len() / 1024 / 1024).unwrap_or(0));
            fs::read(&dat_path).context("Failed to read main_file_cache.dat2")?
        } else {
            warn!("Cache data file not found: {}", dat_path.display());
            Vec::new()
        };

        // Read all index files (idx0-idx28, idx255)
        let mut indices = HashMap::new();
        for i in 0..=28 {
            let idx_path = path.join(format!("main_file_cache.idx{}", i));
            if idx_path.exists() {
                let data = fs::read(&idx_path)?;
                debug!("Loaded index {} ({} bytes)", i, data.len());
                indices.insert(i as u8, data);
            }
        }
        // Master index (255)
        let idx255_path = path.join("main_file_cache.idx255");
        if idx255_path.exists() {
            let data = fs::read(&idx255_path)?;
            debug!("Loaded master index 255 ({} bytes)", data.len());
            indices.insert(255, data);
        }

        info!("Cache opened: {} indices, {} MB data",
            indices.len(),
            data_file.len() / 1024 / 1024);

        let mut cache = Js5Cache {
            data_file,
            indices,
            parsed_indices: HashMap::new(),
            cache_dir: cache_dir.to_string(),
        };

        // Parse all available index reference tables
        cache.parse_indices()?;

        Ok(cache)
    }

    /// Read raw data for a group from the data file using an index.
    pub fn read_group(&self, index_id: u8, group_id: u32) -> Result<Vec<u8>> {
        let idx_data = self.indices.get(&index_id)
            .ok_or_else(|| anyhow::anyhow!("Index {} not found", index_id))?;

        // Each index entry is 6 bytes: 3 bytes size + 3 bytes sector
        let entry_offset = (group_id as usize) * 6;
        if entry_offset + 6 > idx_data.len() {
            bail!("Group {} out of range for index {}", group_id, index_id);
        }

        let size = ((idx_data[entry_offset] as u32) << 16)
            | ((idx_data[entry_offset + 1] as u32) << 8)
            | (idx_data[entry_offset + 2] as u32);
        let sector = ((idx_data[entry_offset + 3] as u32) << 16)
            | ((idx_data[entry_offset + 4] as u32) << 8)
            | (idx_data[entry_offset + 5] as u32);

        if size == 0 || sector == 0 {
            bail!("Group {}:{} is empty (size={}, sector={})", index_id, group_id, size, sector);
        }

        let mut result = Vec::with_capacity(size as usize);
        let mut current_sector = sector;
        let mut remaining = size as usize;
        let mut chunk = 0;
        let extended = group_id >= 65536;

        while remaining > 0 {
            let offset = (current_sector as usize) * SECTOR_SIZE;
            if offset + SECTOR_SIZE > self.data_file.len() {
                bail!("Sector {} out of data file bounds", current_sector);
            }

            let sector_data = &self.data_file[offset..offset + SECTOR_SIZE];

            let (header_size, data_size) = if extended {
                (SECTOR_HEADER_EXT, SECTOR_DATA_EXT)
            } else {
                (SECTOR_HEADER, SECTOR_DATA)
            };

            let read_size = remaining.min(data_size);
            result.extend_from_slice(&sector_data[header_size..header_size + read_size]);
            remaining -= read_size;

            // Read next sector pointer from header
            if extended {
                current_sector = ((sector_data[6] as u32) << 16)
                    | ((sector_data[7] as u32) << 8)
                    | (sector_data[8] as u32);
            } else {
                current_sector = ((sector_data[4] as u32) << 16)
                    | ((sector_data[5] as u32) << 8)
                    | (sector_data[6] as u32);
            }

            chunk += 1;
            if remaining > 0 && current_sector == 0 {
                bail!("Unexpected end of sector chain at chunk {}", chunk);
            }
        }

        Ok(result)
    }

    /// Decompress cache data (type 0=raw, 1=bzip2, 2=gzip).
    /// Translated from Js5Compression.uncompress().
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 5 {
            bail!("Data too short for decompression header");
        }

        let compression_type = data[0];
        let compressed_len = ((data[1] as u32) << 24)
            | ((data[2] as u32) << 16)
            | ((data[3] as u32) << 8)
            | (data[4] as u32);

        match compression_type {
            0 => {
                // No compression — raw data
                let start = 5;
                let end = start + compressed_len as usize;
                if end > data.len() {
                    bail!("Raw data length {} exceeds available {}", compressed_len, data.len() - 5);
                }
                Ok(data[start..end].to_vec())
            }
            1 => {
                // BZIP2
                let _uncompressed_len = ((data[5] as u32) << 24)
                    | ((data[6] as u32) << 16)
                    | ((data[7] as u32) << 8)
                    | (data[8] as u32);
                let compressed = &data[9..9 + compressed_len as usize];
                // bzip2 in RS cache needs the "BZ" header prepended
                let mut bz_data = vec![b'B', b'Z', b'h', b'1'];
                bz_data.extend_from_slice(compressed);
                let mut decoder = bzip2::read::BzDecoder::new(&bz_data[..]);
                let mut output = Vec::new();
                decoder.read_to_end(&mut output)?;
                Ok(output)
            }
            2 => {
                // GZIP
                let _uncompressed_len = ((data[5] as u32) << 24)
                    | ((data[6] as u32) << 16)
                    | ((data[7] as u32) << 8)
                    | (data[8] as u32);
                let compressed = &data[9..9 + compressed_len as usize];
                let mut decoder = GzDecoder::new(compressed);
                let mut output = Vec::new();
                decoder.read_to_end(&mut output)?;
                Ok(output)
            }
            _ => bail!("Unknown compression type: {}", compression_type),
        }
    }

    /// Parse a JS5 index (reference table) from decompressed data.
    /// Translated from Js5Index.java constructor.
    fn parse_index(data: &[u8]) -> Result<Js5Index> {
        let mut buf = Buffer::wrap(data.to_vec());

        let format = buf.g1()? as u32;
        if format != 5 && format != 6 {
            bail!("Unsupported index format: {}", format);
        }

        let version = if format >= 6 { buf.g4()? } else { 0 };
        let flags = buf.g1()?;
        let size = buf.g2()? as usize;

        // Read group IDs (delta-encoded)
        let mut group_ids = Vec::with_capacity(size);
        let mut prev: u16 = 0;
        let mut max_id: u16 = 0;
        for _ in 0..size {
            let delta = buf.g2()?;
            prev = prev.wrapping_add(delta);
            group_ids.push(prev);
            if prev > max_id { max_id = prev; }
        }

        let capacity = max_id as usize + 1;
        let mut group_checksums = vec![0i32; capacity];
        let mut group_versions = vec![0i32; capacity];
        let mut group_sizes = vec![0u16; capacity];
        let mut group_capacities = vec![0u16; capacity];
        let mut file_ids: Vec<Option<Vec<u16>>> = vec![None; capacity];
        let mut group_name_hashes = None;

        // Group name hashes (optional, based on flags)
        if flags != 0 {
            let mut hashes = vec![-1i32; capacity];
            for i in 0..size {
                hashes[group_ids[i] as usize] = buf.g4()?;
            }
            group_name_hashes = Some(hashes);
        }

        // Group checksums
        for i in 0..size {
            group_checksums[group_ids[i] as usize] = buf.g4()?;
        }

        // Group versions
        for i in 0..size {
            group_versions[group_ids[i] as usize] = buf.g4()?;
        }

        // Group sizes (number of files per group)
        for i in 0..size {
            group_sizes[group_ids[i] as usize] = buf.g2()?;
        }

        // File IDs within each group (delta-encoded)
        for i in 0..size {
            let gid = group_ids[i] as usize;
            let num_files = group_sizes[gid] as usize;
            let mut fids = Vec::with_capacity(num_files);
            let mut fprev: u16 = 0;
            let mut fmax: u16 = 0;
            for _ in 0..num_files {
                let delta = buf.g2()?;
                fprev = fprev.wrapping_add(delta);
                fids.push(fprev);
                if fprev > fmax { fmax = fprev; }
            }
            group_capacities[gid] = fmax + 1;
            // If file IDs are sequential (0..n-1), set to None to save memory
            if fmax as usize + 1 == num_files {
                file_ids[gid] = None;
            } else {
                file_ids[gid] = Some(fids);
            }
        }

        Ok(Js5Index {
            version,
            size,
            group_ids,
            capacity,
            group_checksums,
            group_versions,
            group_sizes,
            group_capacities,
            file_ids,
            group_name_hashes,
        })
    }

    /// Parse all available index reference tables.
    fn parse_indices(&mut self) -> Result<()> {
        // Try to parse indices 0-28 using the master index (255)
        if !self.indices.contains_key(&255) {
            warn!("No master index (idx255) found — skipping index parsing");
            return Ok(());
        }

        for archive in 0..=28u8 {
            match self.read_group(255, archive as u32) {
                Ok(raw) => {
                    match Self::decompress(&raw) {
                        Ok(decompressed) => {
                            match Self::parse_index(&decompressed) {
                                Ok(index) => {
                                    info!("Parsed index {}: {} groups, capacity {}",
                                        archive, index.size, index.capacity);
                                    self.parsed_indices.insert(archive, index);
                                }
                                Err(e) => debug!("Failed to parse index {}: {}", archive, e),
                            }
                        }
                        Err(e) => debug!("Failed to decompress index {}: {}", archive, e),
                    }
                }
                Err(e) => debug!("Index {} not available: {}", archive, e),
            }
        }

        info!("Parsed {} archive indices", self.parsed_indices.len());
        Ok(())
    }

    /// Read and decompress a specific file from an archive group.
    pub fn read_file(&self, archive: u8, group: u32) -> Result<Vec<u8>> {
        let raw = self.read_group(archive, group)?;
        Self::decompress(&raw)
    }

    /// Get the number of groups in an archive.
    pub fn group_count(&self, archive: u8) -> usize {
        self.parsed_indices.get(&archive)
            .map(|idx| idx.size)
            .unwrap_or(0)
    }

    /// Get the checksum for a specific archive (used in login packet).
    pub fn archive_checksum(&self, archive: u8) -> i32 {
        // Read raw data from master index and CRC it
        match self.read_group(255, archive as u32) {
            Ok(data) => rs2_common::crc32(&data) as i32,
            Err(_) => 0,
        }
    }
}

/// Archive IDs matching the Java client constants.
pub mod archives {
    pub const ANIMS: u8 = 0;
    pub const BASES: u8 = 1;
    pub const CONFIG: u8 = 2;
    pub const INTERFACES: u8 = 3;
    pub const SYNTH_SOUNDS: u8 = 4;
    pub const MAPS: u8 = 5;
    pub const MUSIC_TRACKS: u8 = 6;
    pub const MODELS: u8 = 7;
    pub const SPRITES: u8 = 8;
    pub const TEXTURES: u8 = 9;
    pub const BINARY: u8 = 10;
    pub const MUSIC_JINGLES: u8 = 11;
    pub const CLIENTSCRIPTS: u8 = 12;
    pub const FONTMETRICS: u8 = 13;
    pub const VORBIS: u8 = 14;
    pub const CONFIG_OBJ: u8 = 16;
    pub const CONFIG_ENUM: u8 = 17;
    pub const CONFIG_NPC: u8 = 18;
    pub const CONFIG_LOC: u8 = 19;
    pub const CONFIG_SEQ: u8 = 20;
    pub const CONFIG_VAR: u8 = 22;
    pub const WORLDMAP_LEGACY: u8 = 23;
    pub const QUICKCHAT: u8 = 24;
    pub const QUICKCHAT_GLOBAL: u8 = 25;
    pub const MATERIALS: u8 = 26;
    pub const CONFIG_PARTICLE: u8 = 27;
}

use anyhow::anyhow;
