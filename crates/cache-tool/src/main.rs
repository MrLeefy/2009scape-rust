//! Standalone cache inspector tool.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use anyhow::{Result, Context};

const SECTOR_SIZE: usize = 520;
const INDEX_ENTRY_SIZE: usize = 6;
const SECTOR_HEADER_SIZE: usize = 8;
const SECTOR_DATA_SIZE: usize = SECTOR_SIZE - SECTOR_HEADER_SIZE;

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rs2-cache-tool <cache_dir> [archive] [group]");
        eprintln!("  Inspects 2009Scape cache files.");
        std::process::exit(1);
    }

    let cache_dir = PathBuf::from(&args[1]);
    let data_path = cache_dir.join("main_file_cache.dat2");

    if !data_path.exists() {
        eprintln!("Cache data file not found: {:?}", data_path);
        std::process::exit(1);
    }

    // Count index files
    let mut idx_count = 0u32;
    for i in 0..=255u8 {
        if cache_dir.join(format!("main_file_cache.idx{}", i)).exists() {
            idx_count += 1;
        }
    }

    println!("Cache: {:?}", cache_dir);
    println!("Archives: {}", idx_count);

    if args.len() >= 4 {
        let archive: u8 = args[2].parse().context("Invalid archive number")?;
        let group: u32 = args[3].parse().context("Invalid group number")?;
        
        let idx_path = cache_dir.join(format!("main_file_cache.idx{}", archive));
        let mut idx_file = File::open(&idx_path)
            .with_context(|| format!("Cannot open {:?}", idx_path))?;
        let mut data_file = File::open(&data_path)?;
        
        // Read index entry
        let entry_offset = group as u64 * INDEX_ENTRY_SIZE as u64;
        idx_file.seek(SeekFrom::Start(entry_offset))?;
        let mut idx_buf = [0u8; INDEX_ENTRY_SIZE];
        idx_file.read_exact(&mut idx_buf)?;

        let size = ((idx_buf[0] as usize) << 16) | ((idx_buf[1] as usize) << 8) | (idx_buf[2] as usize);
        let mut sector = ((idx_buf[3] as u64) << 16) | ((idx_buf[4] as u64) << 8) | (idx_buf[5] as u64);

        println!("Archive={}, Group={}, Size={} bytes, Sector={}", archive, group, size, sector);

        // Read sectors
        let mut result = Vec::with_capacity(size);
        let mut remaining = size;
        let mut part = 0u16;

        while remaining > 0 {
            data_file.seek(SeekFrom::Start(sector * SECTOR_SIZE as u64))?;
            let mut sector_buf = [0u8; SECTOR_SIZE];
            data_file.read_exact(&mut sector_buf)?;

            let next_sector = ((sector_buf[4] as u64) << 16) | ((sector_buf[5] as u64) << 8) | (sector_buf[6] as u64);
            let bytes_to_read = remaining.min(SECTOR_DATA_SIZE);
            result.extend_from_slice(&sector_buf[SECTOR_HEADER_SIZE..SECTOR_HEADER_SIZE + bytes_to_read]);

            remaining -= bytes_to_read;
            sector = next_sector;
            part += 1;
        }

        // Hex dump
        println!("\nHex dump ({} bytes, {} sectors):", result.len(), part);
        for (i, chunk) in result.chunks(16).take(16).enumerate() {
            let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            let ascii: String = chunk.iter().map(|&b| if b.is_ascii_graphic() { b as char } else { '.' }).collect();
            println!("{:04x}: {:48} {}", i * 16, hex, ascii);
        }
        if result.len() > 256 {
            println!("... ({} more bytes)", result.len() - 256);
        }
    } else {
        // List all archives
        for i in 0..=255u8 {
            let idx_path = cache_dir.join(format!("main_file_cache.idx{}", i));
            if let Ok(f) = File::open(&idx_path) {
                let len = f.metadata().map(|m| m.len()).unwrap_or(0);
                let groups = len / INDEX_ENTRY_SIZE as u64;
                if groups > 0 {
                    println!("  idx{}: {} groups", i, groups);
                }
            }
        }
    }

    Ok(())
}
