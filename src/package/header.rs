use binrw::binrw;

#[binrw]
#[derive(Debug, Default)]
#[br(little)]
#[bw(little)]
pub struct PackageHeader {
    #[br(map = |x: [u8; 4]| x)]
    pub magic: [u8; 4],      // "DBPF"
    pub major: u32,          // Usually 2
    pub minor: u32,          // Usually 1
    pub unused1: u32,
    pub unused2: u32,
    pub unused3: u32,
    pub created: u32,
    pub modified: u32,
    pub index_version: u32,  // Usually 3
    pub index_count: u32,
    pub index_size_total_deprecated: u32, // Not used in TS4 but placeholder for alignment
    pub unused4: u32,
    pub index_size: u32,
    pub unused5: [u32; 3],
    pub index_position: u64, // TS4 uses 64-bit offsets
    pub unused6: [u32; 6],
}

impl PackageHeader {
    pub const SIZE: u64 = 96;

    pub fn is_valid(&self) -> bool {
        &self.magic == b"DBPF" && self.major == 2
    }

    pub fn read<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Self, binrw::Error> {
        use binrw::BinReaderExt;
        reader.read_le()
    }

    pub fn write<W: std::io::Write + std::io::Seek>(&self, writer: &mut W) -> Result<(), binrw::Error> {
        use binrw::BinWriterExt;
        writer.write_le(self)
    }
}
