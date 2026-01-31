use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[br(little)]
#[bw(little)]
pub struct TGI {
    pub res_type: u32,
    pub res_group: u32,
    pub instance: u64,
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub tgi: TGI,
    pub offset: u32,
    pub filesize: u32, // Compressed size (highest bit is often 1, meaning compressed)
    pub memsize: u32,  // Decompressed size
    pub compression: u16, // 0x5A42 for Zlib/Deflate, 0x0000 for uncompressed
    pub committed: u16,   // Usually 0x0001
}

impl IndexEntry {
    pub fn is_compressed(&self) -> bool {
        self.compression != 0
    }
}
