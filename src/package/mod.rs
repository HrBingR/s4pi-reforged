pub mod header;
pub mod index;
pub mod resource;

use header::PackageHeader;
use index::{IndexEntry, TGI};
use resource::TypedResource;
use std::io::{Read, Seek, SeekFrom, Write};
use std::fs::File;
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use log::warn;
use rayon::prelude::*;

pub struct Package {
    pub header: PackageHeader,
    pub entries: Vec<IndexEntry>,
    file: Option<File>,
}

impl Package {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let header = PackageHeader::read(&mut file)
            .context("Failed to read package header")?;

        if !header.is_valid() {
            return Err(anyhow!("Invalid DBPF header or unsupported version"));
        }

        file.seek(SeekFrom::Start(header.index_position))?;
        
        // Reading index
        // The index starts with a 4-byte index type
        let mut type_buf = [0u8; 4];
        file.read_exact(&mut type_buf)?;
        let index_type = u32::from_le_bytes(type_buf);

        // Sanity check for index_count to prevent excessive pre-allocation
        let file_len = file.metadata()?.len();
        if header.index_count as u64 * 20 > file_len {
            return Err(anyhow!("Invalid package header: index_count too large for file size"));
        }

        let mut entries = Vec::with_capacity(header.index_count as usize);

        // Constant header parts if bits are set in index_type
        let mut constant_type = None;
        let mut constant_group = None;
        let mut constant_instance_hi = None;

        if (index_type & 0x01) != 0 {
            let mut buf = [0u8; 4];
            file.read_exact(&mut buf)?;
            constant_type = Some(u32::from_le_bytes(buf));
        }
        if (index_type & 0x02) != 0 {
            let mut buf = [0u8; 4];
            file.read_exact(&mut buf)?;
            constant_group = Some(u32::from_le_bytes(buf));
        }
        if (index_type & 0x04) != 0 {
            let mut buf = [0u8; 4];
            file.read_exact(&mut buf)?;
            constant_instance_hi = Some(u32::from_le_bytes(buf));
        }

        for _ in 0..header.index_count {
            let res_type = if let Some(t) = constant_type { t } else {
                let mut buf = [0u8; 4];
                file.read_exact(&mut buf)?;
                u32::from_le_bytes(buf)
            };
            let res_group = if let Some(g) = constant_group { g } else {
                let mut buf = [0u8; 4];
                file.read_exact(&mut buf)?;
                u32::from_le_bytes(buf)
            };
            let instance_hi = if let Some(ihi) = constant_instance_hi { ihi } else {
                let mut buf = [0u8; 4];
                file.read_exact(&mut buf)?;
                u32::from_le_bytes(buf)
            };
            let mut buf_rest = [0u8; 20];
            file.read_exact(&mut buf_rest)?;
            
            let instance_lo = u32::from_le_bytes(buf_rest[0..4].try_into().unwrap());
            let instance = ((instance_hi as u64) << 32) | (instance_lo as u64);
            
            let offset = u32::from_le_bytes(buf_rest[4..8].try_into().unwrap());
            let filesize_raw = u32::from_le_bytes(buf_rest[8..12].try_into().unwrap());
            let filesize = filesize_raw & 0x7FFFFFFF;
            let memsize = u32::from_le_bytes(buf_rest[12..16].try_into().unwrap());
            let mut compression = u16::from_le_bytes(buf_rest[16..18].try_into().unwrap());
            let committed = u16::from_le_bytes(buf_rest[18..20].try_into().unwrap());

            // If high bit of filesize is set, it's compressed.
            // Ensure compression field is non-zero so is_compressed() returns true.
            if (filesize_raw & 0x80000000) != 0 && compression == 0 && filesize != memsize {
                compression = 0x5A42;
            }

            entries.push(IndexEntry {
                tgi: TGI { res_type, res_group, instance },
                offset,
                filesize,
                memsize,
                compression,
                committed,
            });
        }

        Ok(Self {
            header,
            entries,
            file: Some(file),
        })
    }

    pub fn read_raw_resource(&mut self, entry: &IndexEntry) -> Result<Vec<u8>> {
        let file = self.file.as_mut().ok_or_else(|| anyhow!("Package file not open"))?;
        file.seek(SeekFrom::Start(entry.offset as u64))?;
        let mut buf = vec![0u8; entry.filesize as usize];
        file.read_exact(&mut buf)?;

        if entry.is_compressed() {
            if buf.len() >= 2 && buf[1] == 0xFB {
                // RefPack/LZ77
                return decompress_refpack(&buf, entry.memsize as usize);
            }

            // Assume Zlib
            use flate2::read::ZlibDecoder;
            let mut decoder = ZlibDecoder::new(&buf[..]);
            let mut decompressed = Vec::with_capacity(entry.memsize as usize);
            decoder.read_to_end(&mut decompressed)
                .context("Failed to decompress resource data (Zlib)")?;
            
            if decompressed.len() != entry.memsize as usize {
                warn!("Decompressed size mismatch for resource: expected {}, got {}", entry.memsize, decompressed.len());
            }
            return Ok(decompressed);
        }

        Ok(buf)
    }

    pub fn read_resource(&mut self, entry: &IndexEntry) -> Result<TypedResource> {
        let data = self.read_raw_resource(entry)?;
        // Handle decompression here if needed before passing to TypedResource
        TypedResource::from_bytes(entry.tgi.res_type, &data)
    }

    pub fn write_merged<P: AsRef<Path>>(
        output_path: P,
        merged_entries: &std::collections::HashMap<TGI, (Vec<u8>, u32, u16, u16)>,
        compress: bool,
    ) -> Result<()> {
        let mut file = File::create(output_path)?;
        
    let mut header = PackageHeader::default();
    header.magic = *b"DBPF";
    header.major = 2;
    header.minor = 1;
    header.index_version = 0; 
    header.index_count = merged_entries.len() as u32;
    header.unused4 = 0; 
    header.index_size = 0; 
    header.unused5[2] = 3; 
    header.write(&mut file)?;

        file.seek(SeekFrom::Start(PackageHeader::SIZE))?;

        // Sort entries, but try to put Manifest (0x7FB6AD8A) first if it exists
        let mut sorted_keys: Vec<_> = merged_entries.keys().collect();
        sorted_keys.sort_by(|a, b| {
            let a_is_manifest = a.res_type == 0x7FB6AD8A;
            let b_is_manifest = b.res_type == 0x7FB6AD8A;
            if a_is_manifest && !b_is_manifest {
                std::cmp::Ordering::Less
            } else if !a_is_manifest && b_is_manifest {
                std::cmp::Ordering::Greater
            } else {
                (a.res_type, a.res_group, a.instance).cmp(&(b.res_type, b.res_group, b.instance))
            }
        });

        // Parallel compression
        let processed_entries: Vec<(TGI, Vec<u8>, u32, u16, u16)> = sorted_keys
            .par_iter()
            .map(|&tgi| {
                let (raw_data, memsize, compression_flag, committed) = &merged_entries[tgi];
                
                let (final_data, final_compression) = if compress || *compression_flag != 0 {
                    // Check if it's already compressed by looking at the data head (0x78 or 0xFB)
                    let is_already_compressed = raw_data.len() >= 2 && (raw_data[0] == 0x78 || raw_data[1] == 0xFB);
                    
                    if is_already_compressed {
                        (raw_data.clone(), 0x5A42)
                    } else {
                        use flate2::Compression;
                        use flate2::write::ZlibEncoder;
                        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                        if let Err(e) = encoder.write_all(raw_data) {
                            warn!("Compression error for {:?}: {}", tgi, e);
                            return (*tgi, raw_data.clone(), *memsize, 0, *committed);
                        }
                        let compressed = match encoder.finish() {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Compression finish error for {:?}: {}", tgi, e);
                                return (*tgi, raw_data.clone(), *memsize, 0, *committed);
                            }
                        };
                        
                        if compressed.len() < raw_data.len() {
                            (compressed, 0x5A42)
                        } else {
                            (raw_data.clone(), 0x5A42)
                        }
                    }
                } else {
                    (raw_data.clone(), 0x0000)
                };
                
                (*tgi, final_data, *memsize, final_compression, *committed)
            })
            .collect();

        let mut entries = Vec::with_capacity(processed_entries.len());
        for (tgi, final_data, memsize, final_compression, committed) in processed_entries {
            let offset = file.stream_position()? as u32;
            file.write_all(&final_data)?;
            
            entries.push(IndexEntry {
                tgi,
                offset,
                filesize: final_data.len() as u32,
                memsize,
                compression: final_compression,
                committed,
            });
        }

        let index_position = file.stream_position()?;
        
            file.write_all(&0u32.to_le_bytes())?;

        for entry in &entries {
            file.write_all(&entry.tgi.res_type.to_le_bytes())?;
            file.write_all(&entry.tgi.res_group.to_le_bytes())?;
            let instance_hi = (entry.tgi.instance >> 32) as u32;
            file.write_all(&instance_hi.to_le_bytes())?;
            let instance_lo = entry.tgi.instance as u32;
            file.write_all(&instance_lo.to_le_bytes())?;
            file.write_all(&entry.offset.to_le_bytes())?;
            let fs_val = if entry.compression != 0 { entry.filesize | 0x80000000 } else { entry.filesize };
            file.write_all(&fs_val.to_le_bytes())?;
            file.write_all(&entry.memsize.to_le_bytes())?;
            // Use 0x5A42 for Zlib as observed in original Gorilla file
            let compression_to_write: u16 = if entry.compression != 0 { 0x5A42 } else { 0x0000 };
            file.write_all(&compression_to_write.to_le_bytes())?;
            file.write_all(&entry.committed.to_le_bytes())?;
        }

        let index_size = (file.stream_position()? - index_position) as u32;

        // Go back and update header
        header.index_position = index_position;
        header.index_size = 0; // Use 0 for index_size field in header if index_version is 0, matching original
        header.unused4 = index_size; 
        
        file.seek(SeekFrom::Start(0))?;
        header.write(&mut file)?;

        Ok(())
    }
}

fn decompress_refpack(data: &[u8], memsize: usize) -> Result<Vec<u8>> {
    let mut decompressed = vec![0u8; memsize];
    let mut r_pos = 0;
    let mut w_pos = 0;

    if data.len() < 2 {
        return Err(anyhow!("RefPack data too short"));
    }

    let compression_type = data[r_pos];
    r_pos += 1;
    let signature = data[r_pos];
    r_pos += 1;

    if signature != 0xFB {
        return Err(anyhow!("Invalid RefPack signature: expected 0xFB, got 0x{:02X}", signature));
    }

    let size_bytes = if compression_type & 0x80 != 0 { 3 } else { 4 };
    if r_pos + size_bytes > data.len() {
        return Err(anyhow!("RefPack data too short for size header"));
    }
    
    // We already know memsize from the index, but RefPack also stores it.
    // We'll just skip those bytes to avoid endianness/parsing issues if they match.
    r_pos += size_bytes;

    while w_pos < memsize && r_pos < data.len() {
        let byte0 = data[r_pos];
        r_pos += 1;

        if byte0 <= 0x7F {
            if r_pos >= data.len() { break; }
            let byte1 = data[r_pos];
            r_pos += 1;

            let num_plain = (byte0 & 0x03) as usize;
            let num_copy = (((byte0 & 0x1C) >> 2) + 3) as usize;
            let copy_offset = ((byte0 as usize & 0x60) << 3) + byte1 as usize + 1;

            copy_plain(&data, &mut r_pos, &mut decompressed, &mut w_pos, num_plain)?;
            copy_ref(&mut decompressed, &mut w_pos, num_copy, copy_offset)?;
        } else if byte0 <= 0xBF {
            if r_pos + 1 >= data.len() { break; }
            let byte1 = data[r_pos];
            r_pos += 1;
            let byte2 = data[r_pos];
            r_pos += 1;

            let num_plain = ((byte1 & 0xC0) >> 6) as usize;
            let num_copy = ((byte0 & 0x3F) + 4) as usize;
            let copy_offset = ((byte1 as usize & 0x3F) << 8) + byte2 as usize + 1;

            copy_plain(&data, &mut r_pos, &mut decompressed, &mut w_pos, num_plain)?;
            copy_ref(&mut decompressed, &mut w_pos, num_copy, copy_offset)?;
        } else if byte0 <= 0xDF {
            if r_pos + 2 >= data.len() { break; }
            let byte1 = data[r_pos];
            r_pos += 1;
            let byte2 = data[r_pos];
            r_pos += 1;
            let byte3 = data[r_pos];
            r_pos += 1;

            let num_plain = (byte0 & 0x03) as usize;
            let num_copy = ((byte0 as usize & 0x0C) << 6) + byte3 as usize + 5;
            let copy_offset = ((byte0 as usize & 0x10) << 12) + ((byte1 as usize) << 8) + byte2 as usize + 1;

            copy_plain(&data, &mut r_pos, &mut decompressed, &mut w_pos, num_plain)?;
            copy_ref(&mut decompressed, &mut w_pos, num_copy, copy_offset)?;
        } else if byte0 <= 0xFB {
            let num_plain = (((byte0 & 0x1F) << 2) + 4) as usize;
            copy_plain(&data, &mut r_pos, &mut decompressed, &mut w_pos, num_plain)?;
        } else {
            let num_plain = (byte0 & 0x03) as usize;
            copy_plain(&data, &mut r_pos, &mut decompressed, &mut w_pos, num_plain)?;
        }
    }

    Ok(decompressed)
}

fn copy_plain(src: &[u8], src_pos: &mut usize, dest: &mut [u8], dest_pos: &mut usize, count: usize) -> Result<()> {
    if *src_pos + count > src.len() || *dest_pos + count > dest.len() {
        return Err(anyhow!("RefPack: plain copy out of bounds"));
    }
    dest[*dest_pos..*dest_pos + count].copy_from_slice(&src[*src_pos..*src_pos + count]);
    *src_pos += count;
    *dest_pos += count;
    Ok(())
}

fn copy_ref(dest: &mut [u8], dest_pos: &mut usize, count: usize, offset: usize) -> Result<()> {
    if offset > *dest_pos || *dest_pos + count > dest.len() {
        return Err(anyhow!("RefPack: reference copy out of bounds (offset={}, pos={}, count={}, len={})", offset, *dest_pos, count, dest.len()));
    }
    for _ in 0..count {
        dest[*dest_pos] = dest[*dest_pos - offset];
        *dest_pos += 1;
    }
    Ok(())
}
