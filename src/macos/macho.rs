//! Mach-O binary parsing and patching.
//!
//! Supports both FAT (universal) binaries and thin Mach-O 64-bit binaries.
//! Translates virtual addresses to file offsets and writes patch bytes.

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::macos::config::Entry;
use crate::macos::Error;

// Mach-O magic numbers
const FAT_MAGIC: u32 = 0xCAFEBABE;
const FAT_CIGAM: u32 = 0xBEBAFECA;
const MH_MAGIC_64: u32 = 0xFEEDFACF;

// Load command types
const LC_SEGMENT_64: u32 = 0x19;

// CPU types
const CPU_TYPE_ARM64: u32 = 0x0100000C; // CPU_TYPE_ARM | CPU_ARCH_ABI64
const CPU_TYPE_X86_64: u32 = 0x01000007; // CPU_TYPE_X86 | CPU_ARCH_ABI64

/// Map arch name string to Mach-O CPU type constant.
fn arch_to_cputype(arch: &str) -> Option<u32> {
    match arch {
        "arm64" => Some(CPU_TYPE_ARM64),
        "x86_64" => Some(CPU_TYPE_X86_64),
        _ => None,
    }
}

/// Result of a single patch operation, for logging/reporting.
#[derive(Debug)]
pub struct PatchResult {
    pub arch: String,
    pub va: u64,
    pub file_offset: u64,
    pub old_bytes: Vec<u8>,
    pub new_bytes: Vec<u8>,
}

/// Apply patch entries to a Mach-O binary file.
/// Returns details of each applied patch (including original bytes for restore).
pub fn patch_binary(binary_path: &Path, entries: &[Entry]) -> Result<Vec<PatchResult>, Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(binary_path)
        .map_err(|e| Error::Io(format!("open {}: {}", binary_path.display(), e)))?;

    // Read magic to determine FAT vs thin
    let magic_be = read_u32_be(&mut file, 0)?;

    let mut results = Vec::new();

    if magic_be == FAT_MAGIC || magic_be == FAT_CIGAM {
        let is_swapped = magic_be == FAT_CIGAM;
        let nfat = read_fat_u32(&mut file, 4, is_swapped)?;

        // Read all fat_arch entries first (each 20 bytes, starting at offset 8)
        let mut arch_table: Vec<(u32, u32)> = Vec::new(); // (cputype, offset)
        for i in 0..nfat {
            let base = 8 + (i as u64) * 20;
            let cputype = read_fat_u32(&mut file, base, is_swapped)?;
            let offset = read_fat_u32(&mut file, base + 8, is_swapped)?;
            arch_table.push((cputype, offset));
        }

        for (cputype, slice_offset) in &arch_table {
            for entry in entries {
                let entry_cpu = arch_to_cputype(&entry.arch)
                    .ok_or_else(|| Error::Patch(format!("unknown arch: {}", entry.arch)))?;
                if entry_cpu != *cputype {
                    continue;
                }
                let target_va = entry
                    .addr_u64()
                    .map_err(|e| Error::Patch(format!("invalid addr '{}': {}", entry.addr, e)))?;
                let patch_bytes = entry
                    .asm_bytes()
                    .ok_or_else(|| Error::Patch(format!("invalid asm hex: {}", entry.asm)))?;

                let result = patch_one_slice(
                    &mut file,
                    *slice_offset as u64,
                    target_va,
                    &patch_bytes,
                    &entry.arch,
                )?;
                results.push(result);
            }
        }
    } else {
        // Thin Mach-O: read magic as little-endian
        let magic_le = read_u32_le(&mut file, 0)?;
        if magic_le != MH_MAGIC_64 {
            return Err(Error::Patch(format!(
                "not a 64-bit Mach-O: magic=0x{:08x}",
                magic_le
            )));
        }

        let cputype = read_u32_le(&mut file, 4)?;

        for entry in entries {
            let entry_cpu = arch_to_cputype(&entry.arch)
                .ok_or_else(|| Error::Patch(format!("unknown arch: {}", entry.arch)))?;
            if entry_cpu != cputype {
                continue;
            }
            let target_va = entry
                .addr_u64()
                .map_err(|e| Error::Patch(format!("invalid addr '{}': {}", entry.addr, e)))?;
            let patch_bytes = entry
                .asm_bytes()
                .ok_or_else(|| Error::Patch(format!("invalid asm hex: {}", entry.asm)))?;

            let result = patch_one_slice(&mut file, 0, target_va, &patch_bytes, &entry.arch)?;
            results.push(result);
        }
    }

    if results.is_empty() {
        return Err(Error::Patch("no matching arch entries found".into()));
    }

    Ok(results)
}

/// Patch a single Mach-O slice at a given offset within the file.
/// Walks LC_SEGMENT_64 load commands to translate VA → file offset.
fn patch_one_slice(
    file: &mut std::fs::File,
    slice_offset: u64,
    target_va: u64,
    patch: &[u8],
    arch_name: &str,
) -> Result<PatchResult, Error> {
    // Read mach_header_64 (32 bytes): magic(4) cputype(4) cpusubtype(4) filetype(4) ncmds(4) sizeofcmds(4) flags(4) reserved(4)
    let magic = read_u32_le(file, slice_offset)?;
    if magic != MH_MAGIC_64 {
        return Err(Error::Patch(format!(
            "[{}] not MH_MAGIC_64 at offset 0x{:x}: 0x{:08x}",
            arch_name, slice_offset, magic
        )));
    }

    let ncmds = read_u32_le(file, slice_offset + 16)?;

    // Walk load commands (start right after mach_header_64 = 32 bytes)
    let mut lc_offset = slice_offset + 32;

    for _ in 0..ncmds {
        let cmd = read_u32_le(file, lc_offset)?;
        let cmdsize = read_u32_le(file, lc_offset + 4)?;

        if cmd == LC_SEGMENT_64 {
            // segment_command_64 layout after cmd(4)+cmdsize(4):
            //   segname(16) vmaddr(8) vmsize(8) fileoff(8) filesize(8) ...
            let vmaddr = read_u64_le(file, lc_offset + 8 + 16)?;
            let vmsize = read_u64_le(file, lc_offset + 8 + 16 + 8)?;
            let fileoff = read_u64_le(file, lc_offset + 8 + 16 + 16)?;

            if target_va >= vmaddr && target_va < vmaddr + vmsize {
                let file_offset = slice_offset + fileoff + (target_va - vmaddr);

                println!(
                    "[{}] vmaddr=0x{:x}, fileoff=0x{:x}, sliceoff=0x{:x}",
                    arch_name, vmaddr, fileoff, slice_offset
                );
                println!(
                    "[{}] patch VA=0x{:x} -> file offset=0x{:x}",
                    arch_name, target_va, file_offset
                );

                // Read original bytes for backup
                let mut old_bytes = vec![0u8; patch.len()];
                file.seek(SeekFrom::Start(file_offset))
                    .map_err(|e| Error::Io(format!("seek to 0x{:x}: {}", file_offset, e)))?;
                file.read_exact(&mut old_bytes)
                    .map_err(|e| Error::Io(format!("read at 0x{:x}: {}", file_offset, e)))?;

                // Write patch bytes
                file.seek(SeekFrom::Start(file_offset))
                    .map_err(|e| Error::Io(format!("seek to 0x{:x}: {}", file_offset, e)))?;
                file.write_all(patch)
                    .map_err(|e| Error::Io(format!("write at 0x{:x}: {}", file_offset, e)))?;

                return Ok(PatchResult {
                    arch: arch_name.to_string(),
                    va: target_va,
                    file_offset,
                    old_bytes,
                    new_bytes: patch.to_vec(),
                });
            }
        }

        lc_offset += cmdsize as u64;
    }

    Err(Error::Patch(format!(
        "[{}] VA 0x{:x} not found in any segment",
        arch_name, target_va
    )))
}

// -- Helper I/O functions --

fn read_u32_be(file: &mut std::fs::File, offset: u64) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| Error::Io(e.to_string()))?;
    file.read_exact(&mut buf)
        .map_err(|e| Error::Io(e.to_string()))?;
    Ok(u32::from_be_bytes(buf))
}

fn read_u32_le(file: &mut std::fs::File, offset: u64) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| Error::Io(e.to_string()))?;
    file.read_exact(&mut buf)
        .map_err(|e| Error::Io(e.to_string()))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64_le(file: &mut std::fs::File, offset: u64) -> Result<u64, Error> {
    let mut buf = [0u8; 8];
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| Error::Io(e.to_string()))?;
    file.read_exact(&mut buf)
        .map_err(|e| Error::Io(e.to_string()))?;
    Ok(u64::from_le_bytes(buf))
}

/// Read a u32 from FAT header (big-endian by default, little-endian if swapped).
fn read_fat_u32(file: &mut std::fs::File, offset: u64, is_swapped: bool) -> Result<u32, Error> {
    if is_swapped {
        read_u32_le(file, offset)
    } else {
        read_u32_be(file, offset)
    }
}
