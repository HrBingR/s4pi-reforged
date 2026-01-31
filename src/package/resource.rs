use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use anyhow::{Result, Context};
use binrw::{BinRead, BinWrite, binrw, BinReaderExt, BinWriterExt};
use crate::package::index::TGI;
use std::collections::HashMap;

pub trait Resource: std::fmt::Debug {
    fn from_bytes(data: &[u8]) -> Result<Self> where Self: Sized;
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

/// A wrapper for unknown or generic resources
#[derive(Debug)]
pub struct GenericResource {
    pub data: Vec<u8>,
}

impl Resource for GenericResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(Self { data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.data.clone())
    }
}

/// NameMap resource (0x0166038C)
#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct NameMapResource {
    pub version: u32,
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub entries: Vec<NameMapEntry>,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct NameMapEntry {
    pub instance: u64,
    #[br(temp)]
    #[bw(calc = name.len() as u32)]
    name_len: u32,
    #[br(count = name_len, map = |s: Vec<u8>| String::from_utf8_lossy(&s).into_owned())]
    #[bw(map = |s: &String| s.as_bytes().to_vec())]
    pub name: String,
}

impl Resource for NameMapResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read NameMapResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write NameMapResource")?;
        Ok(data)
    }
}

/// Clip resource (0x6B20C4F3)
#[derive(Debug)]
pub struct ClipResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for ClipResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = cursor.read_le::<u32>()?;
        Ok(Self { version, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// CAS Part resource (0x034AE111)
#[derive(Debug)]
pub struct CasPartResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for CasPartResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = cursor.read_le::<u32>()?;
        Ok(Self { version, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// Jazz resource (0x02D5DF13)
#[derive(Debug)]
pub struct JazzResource {
    pub raw_data: Vec<u8>,
}

impl Resource for JazzResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(Self { raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

pub enum TypedResource {
    NameMap(NameMapResource),
    Stbl(StblResource),
    ObjectDefinition(ObjectDefinitionResource),
    SimData(SimDataResource),
    Text(TextResource),
    Catalog(CatalogResource),
    Rle(RleResource),
    Dst(DstResource),
    Script(ScriptResource),
    Clip(ClipResource),
    CasPart(CasPartResource),
    Jazz(JazzResource),
    Rcol(RcolResource),
    Rig(RigResource),
    Lite(LiteResource),
    Thumbnail(ThumbnailResource),
    Complate(ComplateResource),
    Txtc(TxtcResource),
    ObjKey(ObjKeyResource),
    SimModifier(SimModifierResource),
    Bone(BoneResource),
    Cwal(CwalResource),
    Cfnd(CfndResource),
    Cstr(CstrResource),
    Mtbl(MtblResource),
    Trim(TrimResource),
    Geom(GeomResource),
    Manifest(ManifestResource),
    Xml(GenericStubResource),
    Audio(GenericStubResource),
    Image(GenericStubResource),
    Binary(GenericStubResource),
    World(GenericStubResource),
    Generic(GenericResource),
}

#[derive(Debug)]
pub struct GenericStubResource {
    pub res_type: u32,
    pub data: Vec<u8>,
}

impl Resource for GenericStubResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(Self { res_type: 0, data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.data.clone())
    }
}

impl GenericStubResource {
    pub fn from_bytes_with_type(res_type: u32, data: &[u8]) -> Result<Self> {
        Ok(Self { res_type, data: data.to_vec() })
    }
}

impl TypedResource {
    pub fn from_bytes(res_type: u32, data: &[u8]) -> Result<Self> {
        match res_type {
            // Already handled specific types (NameMap, Stbl, etc.)
            0x0166038C | 0xF3A38370 => Ok(TypedResource::NameMap(NameMapResource::from_bytes(data)?)),
            0x220557AA | 0x220557DA => Ok(TypedResource::Stbl(StblResource::from_bytes(data)?)),
            0xC0DB5AE7 => Ok(TypedResource::ObjectDefinition(ObjectDefinitionResource::from_bytes(data)?)),
            0x545AC67A => Ok(TypedResource::SimData(SimDataResource::from_bytes(data)?)),
            0x034AEECB | 0xE882D22F | 0x738E14F4 | 0x6017E351 => Ok(TypedResource::Text(TextResource::from_bytes(data)?)),
            
            // Catalog resources
            0x319E4F1D | 0x9F5CFF10 | 0xB4F762C9 | 0x07936CE0 | 0x1D6DF1CF | 0x2FAE983E |
            0xA057811C | 0xEBCBB16C | 0x9A20CD1C | 0xD5F0F921 | 0x1C1CF1F7 | 0xE7ADA79D |
            0xA5DFFCF3 | 0x0418FE2A | 0xF1EDBD86 | 0x3F0C529A | 0xB0311D0F | 0x84C23219 |
            0x74050B1F | 0x91EDBD3E | 0x48C28979 | 0xA8F7B517 => {
                match res_type {
                    0xD5F0F921 => Ok(TypedResource::Cwal(CwalResource::from_bytes(data)?)),
                    0x2FAE983E => Ok(TypedResource::Cfnd(CfndResource::from_bytes(data)?)),
                    0x9A20CD1C => Ok(TypedResource::Cstr(CstrResource::from_bytes(data)?)),
                    _ => Ok(TypedResource::Catalog(CatalogResource::from_bytes(data)?)),
                }
            }
            0x3453CF95 => Ok(TypedResource::Rle(RleResource::from_bytes(data)?)),
            0x00B2D882 | 0xB6C8B6A0 => Ok(TypedResource::Dst(DstResource::from_bytes(data)?)),
            0x073FAA07 => Ok(TypedResource::Script(ScriptResource::from_bytes(data)?)),
            0x6B20C4F3 => Ok(TypedResource::Clip(ClipResource::from_bytes(data)?)),
            0x034AE111 => Ok(TypedResource::CasPart(CasPartResource::from_bytes(data)?)),
            0x02D5DF13 => Ok(TypedResource::Jazz(JazzResource::from_bytes(data)?)),
            0x015A1849 | 0x01D0E75D | 0x01D10F34 | 0x01661233 => Ok(TypedResource::Rcol(RcolResource::from_bytes(data)?)),
            0x8EAF13DE => Ok(TypedResource::Rig(RigResource::from_bytes(data)?)),
            0x03B4C61D => Ok(TypedResource::Lite(LiteResource::from_bytes(data)?)),
            0x0D338A3A | 0x16CCF748 | 0x3BD45407 | 0x3C1AF1F2 | 0x3C2A8647 | 0x5B282D45 | 
            0xCD9DE247 | 0xE18CAEE2 | 0xE254AE6E | 0x0580A2B4 | 0x0580A2B5 | 0x0580A2B6 |
            0x0589DC44 | 0x0589DC45 | 0x0589DC46 | 0x0589DC47 | 0x05B17698 | 0x05B17699 |
            0x05B1769A | 0x05B1B524 | 0x05B1B525 | 0x05B1B526 | 0x2653E3C8 | 0x2653E3C9 |
            0x2653E3CA | 0x2D4284F0 | 0x2D4284F1 | 0x2D4284F2 | 0x5DE9DBA0 | 0x5DE9DBA1 |
            0x5DE9DBA2 | 0x626F60CC | 0x626F60CD | 0x626F60CE | 0x9C925813 | 0xA1FF2FC4 |
            0xAD366F95 | 0xAD366F96 | 0xFCEAB65B => Ok(TypedResource::Thumbnail(ThumbnailResource::from_bytes(data)?)),
            0x044AE110 => Ok(TypedResource::Complate(ComplateResource::from_bytes(data)?)),
            0x033A1435 | 0x0341ACC9 => Ok(TypedResource::Txtc(TxtcResource::from_bytes(data)?)),
            0x02DC343F => Ok(TypedResource::ObjKey(ObjKeyResource::from_bytes(data)?)),
            0xC5F6763E => Ok(TypedResource::SimModifier(SimModifierResource::from_bytes(data)?)),
            0x00AE6C67 => Ok(TypedResource::Bone(BoneResource::from_bytes(data)?)),
            0x81CA1A10 => Ok(TypedResource::Mtbl(MtblResource::from_bytes(data)?)),
            0x76BCF80C => Ok(TypedResource::Trim(TrimResource::from_bytes(data)?)),

            // Manifest stub
            0x73E93EEB | 0x7FB6AD8A => Ok(TypedResource::Manifest(ManifestResource::from_bytes(data)?)),

            // Legacy stubs (XML/Text)
            0x0069453E | 0x0333406C | 0x03B33DDF | 0x03E9D964 | 0x04D2B465 | 0x074DFB83 |
            0x0C772E27 | 0x0CA4C78B | 0x0E4D15FB | 0x0EEB823A | 0x11E72A63 | 0x122FC66A |
            0x12496650 | 0x1A8506C5 | 0x1B25A024 | 0x1C12D458 | 0x2451C101 | 0x2553F435 |
            0x2673076D | 0x28B64675 | 0x2C01BC15 | 0x2C70ADF8 | 0x2E47A104 | 0x2F59B437 |
            0x31397645 | 0x339BC5BD | 0x37B999F1 | 0x37EF2EE7 | 0x3F163505 | 0x3FD6243E |
            0x4115F9D5 | 0x457FC032 | 0x48C2D5ED | 0x48C75CE3 | 0x49395302 | 0x4DB8251E |
            0x4F739CEE | 0x51077643 | 0x51E7A18D | 0x54BD4618 | 0x598F28E7 | 0x5B02819E |
            0x6017E896 | 0x6224C9D6 | 0x69A5DAA4 | 0x6E0DDA9F | 0x6FA49828 | 0x7147A350 |
            0x738E6C56 | 0x73996BEB | 0x78559E9E | 0x7DF2169C | 0x800A3690 | 0x86136AA5 |
            0x893E429C | 0x8FB3E0B1 | 0x99CBC754 | 0x99D98089 | 0x9C07855F | 0x9CC21262 |
            0x9DB989FD | 0x9DDB5FDA | 0x9DF2F1F2 | 0xA576C2E7 | 0xAD6FDF1F | 0xAFADAC48 |
            0xB61DE6B4 | 0xB7FF8F95 | 0xB9881120 | 0xBA7B60B8 | 0xBE04173A | 0xC020FCAD |
            0xC202C770 | 0xC2CAA646 | 0xC582D2FB | 0xCB5FDDC7 | 0xD2DC5BAD | 0xD70DD79E |
            0xD83892B7 | 0xD8800D66 | 0xDD057DCC | 0xDE6AD3CF | 0xDEBAFB73 |
            0xE04A24A3 | 0xE06AE65E | 0xE0D75679 | 0xE1477E18 | 0xE231B3D8 | 0xE24B5287 |
            0xE350DBD8 | 0xE5105066 | 0xE5105068 | 0xE55EEACB | 0xE6BBD7DE | 0xEB97F823 |
            0xEC3DA10E | 0xEC6A8FC6 | 0xEE17C6AD | 0xF3ABFF3C | 0xF93B40CF | 0xF958A092 |
            0xFA0FFA34 | 0xFBC3AEEB => Ok(TypedResource::Xml(GenericStubResource::from_bytes_with_type(res_type, data)?)),

            // Legacy stubs (Audio)
            0x01A527DB | 0x01EEF63A | 0xBDD82221 | 0x01131757 => Ok(TypedResource::Audio(GenericStubResource::from_bytes_with_type(res_type, data)?)),

            // Legacy stubs (Image)
            0x2E75C764 | 0x2E75C765 | 0x2E75C766 | 0x2E75C767 | 0x2F7D0004 | 0x3F8662EA |
            0xD84E7FC5 | 0xD84E7FC6 | 0xD84E7FC7 => Ok(TypedResource::Image(GenericStubResource::from_bytes_with_type(res_type, data)?)),

            // Legacy stubs (World)
            0x19301120 | 0x1CC04273 | 0x370EFD6E | 0x3924DE26 | 0x9063660D | 0x9151E6BC |
            0xDB43E069 | 0xAC16FBEC | 0x025ED6F4 | 0x0354796A | 0x71BDB8A2 | 0xCF9A4ACE => Ok(TypedResource::World(GenericStubResource::from_bytes_with_type(res_type, data)?)),

            // Legacy stubs (Binary)
            0x00DE5AC5 | 0x010FAF71 | 0x02019972 | 0x033260E3 | 0x033B2B66 | 0x067CAA11 |
            0x0A227BCF | 0x105205BA | 0x12952634 | 0x153D2219 | 0x16CA6BC4 |
            0x17C0C281 | 0x18F3C673 | 0x1C99B344 | 0x20D81496 | 0x25796DCA |
            0x26978421 | 0x276CA4B9 | 0x2A8A5E22 | 0x2AD195F2 | 0x3BF8FD86 |
            0x4F726BBE | 0x56278554 | 0x5BE29703 | 0x62E94D38 | 0x62ECC59A |
            0x6F40796A | 0x71A449C9 | 0x729F6C4F | 0x78C8BCE4 | 
            0x892C4B8A | 0x8B18FF6E | 0x91568FD8 | 0x9917EACD | 0xA0451CBD |
            0xAC03A936 | 0xB0118C15 | 
            0xB3C438F0 | 0xBA856C78 | 0xBC4A5044 | 0xBC80ED59 | 
            0xC71CA490 | 0xD3044521 | 0xD33C281E | 0xD382BF57 | 0xD65DAFF9 | 0xD99F5E5C |
            0xD9BD0909 | 0xEA5118B0 | 0xEAA32ADD | 0xF0633989 | 
            0xFD04E3BE => Ok(TypedResource::Binary(GenericStubResource::from_bytes_with_type(res_type, data)?)),

            _ => Ok(TypedResource::Generic(GenericResource::from_bytes(data)?)),
        }
    }
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct ColorList {
    #[br(temp)]
    #[bw(calc = colors.len() as u8)]
    count: u8,
    #[br(count = count)]
    pub colors: Vec<u32>,
}

/// Wall resource (0xD5F0F921)
#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct CwalResource {
    pub version: u32,
    pub common: CatalogCommon,
    pub matd_list: WallMATDEntryList,
    pub img_group_list: WallImgGroupList,
    pub unk01: u32,
    pub colors: ColorList,
    pub unk_iid01: u64,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct WallMATDEntryList {
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub entries: Vec<WallMATDEntry>,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct WallMATDEntry {
    pub matd_label: u32, // MainWallHeight enum in C#
    pub matd_ref: TGI,   // Order is ITG in C# but TGIBlock default is usually ITG for catalog
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct WallImgGroupList {
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub entries: Vec<WallImgGroup>,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct WallImgGroup {
    pub unk01: u32,
    pub img_ref: TGI,
}

impl Resource for CwalResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read CwalResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write CwalResource")?;
        Ok(data)
    }
}

/// Foundation resource (0x2FAE983E)
#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct CfndResource {
    pub version: u32,
    pub common: CatalogCommon,
    pub unk01: u8,
    pub unk02: u8,
    pub modl_ref1: TGI,
    pub material_variant: u32,
    pub swatch_grouping: u64,
    pub float1: f32,
    pub float2: f32,
    pub trim_ref: TGI,
    pub modl_ref2: TGI,
    pub colors: ColorList,
}

impl Resource for CfndResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read CfndResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write CfndResource")?;
        Ok(data)
    }
}

/// Stairs resource (0x9A20CD1C)
#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct CstrResource {
    pub version: u32,
    pub common: CatalogCommon,
    pub hash_indicator: u32,
    pub hash01: u32,
    pub hash02: u32,
    pub hash03: u32,
    pub ref_list: CstrReferences,
    pub unk01: u8,
    pub unk02: u8,
    pub unk03: u8,
    pub material_variant: u32,
    pub swatch_grouping: u64,
    pub colors: ColorList,
    pub unk05: u8,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct CstrReferences {
    pub modl_ref01: TGI,
    pub modl_ref02: TGI,
    pub modl_ref03: TGI,
    pub unk_ref01: TGI,
    pub wall_ref: TGI,
    pub obj_ref: TGI,
}

impl Resource for CstrResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read CstrResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write CstrResource")?;
        Ok(data)
    }
}

/// Material Table resource (0x81CA1A10)
#[binrw]
#[derive(Debug)]
#[br(little, magic = b"MTBL")]
#[bw(little, magic = b"MTBL")]
pub struct MtblResource {
    pub version: u32,
    pub entries: MtblEntryList,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct MtblEntryList {
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub entries: Vec<MtblEntry>,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct MtblEntry {
    pub model_iid: u64,
    pub base_file_name_hash: u64,
    pub width_and_mapping_flags: u8,
    pub minimum_wall_height: u8,
    pub number_of_levels: u8,
    pub unused: u8,
    pub thumbnail_bounds_min_x: f32,
    pub thumbnail_bounds_min_z: f32,
    pub thumbnail_bounds_min_y: f32,
    pub thumbnail_bounds_max_x: f32,
    pub thumbnail_bounds_max_z: f32,
    pub thumbnail_bounds_max_y: f32,
    pub model_flags: u32,
    pub vfx_hash: u64,
}

impl Resource for MtblResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read MtblResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write MtblResource")?;
        Ok(data)
    }
}

/// Trim resource (0x76BCF80C)
#[binrw]
#[derive(Debug)]
#[br(little, magic = b"TRIM")]
#[bw(little, magic = b"TRIM")]
pub struct TrimResource {
    pub version: u32,
    #[br(args(version))]
    pub entries: TrimEntryList,
    pub material_set_key: TGI,
    pub has_footprint: u8,
}

#[derive(Debug)]
pub enum TrimEntryList {
    V3(Vec<TrimPt3Entry>),
    V4(Vec<TrimPt4Entry>),
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct TrimPt3Entry {
    pub x: f32,
    pub y: f32,
    pub v: f32,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct TrimPt4Entry {
    pub x: f32,
    pub y: f32,
    pub v: f32,
    pub mapping_mode: f32,
}

impl BinRead for TrimEntryList {
    type Args<'a> = (u32,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _options: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let version = args.0;
        let count = reader.read_le::<u32>()?;
        if version == 3 {
            let mut entries = Vec::with_capacity(count as usize);
            for _ in 0..count {
                entries.push(TrimPt3Entry::read_le(reader)?);
            }
            Ok(TrimEntryList::V3(entries))
        } else {
            let mut entries = Vec::with_capacity(count as usize);
            for _ in 0..count {
                entries.push(TrimPt4Entry::read_le(reader)?);
            }
            Ok(TrimEntryList::V4(entries))
        }
    }
}

impl BinWrite for TrimEntryList {
    type Args<'a> = (u32,);

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _options: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            TrimEntryList::V3(entries) => {
                writer.write_le(&(entries.len() as u32))?;
                for entry in entries {
                    entry.write_le(writer)?;
                }
            }
            TrimEntryList::V4(entries) => {
                writer.write_le(&(entries.len() as u32))?;
                for entry in entries {
                    entry.write_le(writer)?;
                }
            }
        }
        Ok(())
    }
}

impl Resource for TrimResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read TrimResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write TrimResource")?;
        Ok(data)
    }
}

/// RCOL (Resource Collection) base wrapper
#[derive(Debug)]
pub struct RcolResource {
    pub version: u32,
    pub public_chunks: i32,
    pub unused: u32,
    pub external_resources: Vec<TGI>,
    pub chunks: Vec<RcolChunk>,
}

#[derive(Debug)]
pub struct RcolChunk {
    pub tgi: TGI,
    pub tag: String,
    pub data: Vec<u8>,
}

impl Resource for RcolResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = cursor.read_le::<u32>()?;
        let public_chunks = cursor.read_le::<i32>()?;
        let unused = cursor.read_le::<u32>()?;
        let count_resources = cursor.read_le::<i32>()?;
        let count_chunks = cursor.read_le::<i32>()?;

        if count_resources < 0 || count_chunks < 0 {
            return Err(anyhow::anyhow!("Invalid RCOL header: negative count"));
        }

        // Basic sanity check: each resource/chunk entry takes at least some bytes
        let data_len = data.len();
        if (count_resources as usize * 16) > data_len || (count_chunks as usize * 28) > data_len {
             return Err(anyhow::anyhow!("Invalid RCOL header: count too large for data size"));
        }

        let mut chunk_tgis = Vec::with_capacity(count_chunks as usize);
        for _ in 0..count_chunks {
            let res_type = cursor.read_le::<u32>()?;
            let res_group = cursor.read_le::<u32>()?;
            let instance = cursor.read_le::<u64>()?;
            chunk_tgis.push(TGI { res_type, res_group, instance });
        }

        let mut external_resources = Vec::with_capacity(count_resources as usize);
        for _ in 0..count_resources {
            let res_type = cursor.read_le::<u32>()?;
            let res_group = cursor.read_le::<u32>()?;
            let instance = cursor.read_le::<u64>()?;
            external_resources.push(TGI { res_type, res_group, instance });
        }

        let mut chunk_index = Vec::with_capacity(count_chunks as usize);
        for _ in 0..count_chunks {
            let position = cursor.read_le::<u32>()?;
            let length = cursor.read_le::<i32>()?;
            chunk_index.push((position, length));
        }

        if count_chunks == 1 {
            let (pos, len) = chunk_index[0];
            // If pos and len are both 0 (not explicitly set in some malformed RCOLs like GEOM)
            if pos == 0 && len == 0 {
                let current_pos = cursor.stream_position()?;
                chunk_index[0].0 = current_pos as u32;
                chunk_index[0].1 = (data_len as u64 - current_pos) as i32;
            }
        }

        let mut chunks = Vec::with_capacity(count_chunks as usize);
        for i in 0..count_chunks as usize {
            let (pos, len) = chunk_index[i];
            let tgi = chunk_tgis[i];

            if len < 0 {
                return Err(anyhow::anyhow!("Invalid RCOL chunk length: {}", len));
            }
            if pos as u64 + len as u64 > data_len as u64 {
                return Err(anyhow::anyhow!("RCOL chunk extends beyond data bounds: pos={}, len={}", pos, len));
            }

            cursor.seek(SeekFrom::Start(pos as u64))?;
            let mut tag_buf = [0u8; 4];
            cursor.read_exact(&mut tag_buf)?;
            let tag = String::from_utf8_lossy(&tag_buf).into_owned();

            cursor.seek(SeekFrom::Start(pos as u64))?;
            let mut chunk_data = vec![0u8; len as usize];
            cursor.read_exact(&mut chunk_data)?;

            chunks.push(RcolChunk {
                tgi,
                tag,
                data: chunk_data,
            });
        }

        Ok(Self {
            version,
            public_chunks,
            unused,
            external_resources,
            chunks,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        // Implementation of RCOL serialization would be complex due to offset management.
        // For investigation purposes, we might not need to write it back yet.
        Err(anyhow::anyhow!("RcolResource writing not yet implemented"))
    }
}

/// Rig resource (0x8EAF13DE)
#[derive(Debug)]
pub struct RigResource {
    pub format: String,
    pub raw_data: Vec<u8>,
}

impl Resource for RigResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let mut format = "RawGranny";
        if data.len() >= 8 {
            let dw1 = cursor.read_le::<u32>()?;
            let dw2 = cursor.read_le::<u32>()?;
            if dw1 == 0x8EAF13DE && dw2 == 0x00000000 {
                format = "WrappedGranny";
            } else if (dw1 == 0x00000003 || dw1 == 0x00000004) && (dw2 == 0x00000001 || dw2 == 0x00000002) {
                format = "Clear";
            }
        }
        Ok(Self { format: format.to_string(), raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// Lite resource (0x03B4C61D)
#[derive(Debug)]
pub struct LiteResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for LiteResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let _tag = cursor.read_le::<u32>()?; // "LITE"
        let version = cursor.read_le::<u32>()?;
        Ok(Self { version, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// SimData resource (0x545AC67A)
#[derive(Debug)]
pub struct SimDataResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for SimDataResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let magic = cursor.read_le::<u32>()?;
        if magic != 0x41544144 { // "DATA"
            return Err(anyhow::anyhow!("Invalid SimData magic"));
        }
        let version = cursor.read_le::<u32>()?;
        Ok(Self { version, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// Text resource (various types like Tuning 0x034AEECB, XML 0x738E14F4, etc.)
#[derive(Debug)]
pub struct TextResource {
    pub content: String,
}

impl Resource for TextResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(Self { content: String::from_utf8_lossy(data).into_owned() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.content.as_bytes().to_vec())
    }
}

/// Object Definition resource (0xC0DB5AE7)
#[derive(Debug)]
pub struct ObjectDefinitionResource {
    pub version: u16,
    pub properties: HashMap<u32, ObjectProperty>,
}

#[derive(Debug)]
pub enum ObjectProperty {
    String(String),
    UInt64(u64),
    TGIBlockList(Vec<TGI>),
    UInt32List(Vec<u32>),
    Byte(u8),
    UInt32(u32),
    Float(f32),
    Bool(bool),
    UInt16List(Vec<u16>),
    FloatList(Vec<f32>),
    ByteList(Vec<u8>),
    Unknown(Vec<u8>),
}

impl Resource for ObjectDefinitionResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = cursor.read_le::<u16>()?;
        let table_offset = cursor.read_le::<u32>()?;

        cursor.seek(SeekFrom::Start(table_offset as u64))?;
        let entry_count = cursor.read_le::<u16>()?;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let prop_id = cursor.read_le::<u32>()?;
            let offset = cursor.read_le::<u32>()?;
            entries.push((prop_id, offset));
        }

        let mut properties = HashMap::new();
        for (prop_id, offset) in entries {
            cursor.seek(SeekFrom::Start(offset as u64))?;
            let property = match prop_id {
                0xE7F07786 | 0x790FA4BC | 0xECD5A95F => { // Name, Tuning, MaterialVariant
                    let len = cursor.read_le::<u32>()?;
                    let mut buf = vec![0u8; len as usize];
                    cursor.read_exact(&mut buf)?;
                    ObjectProperty::String(String::from_utf8_lossy(&buf).into_owned())
                }
                0xB994039B | 0x52F7F4BC => { // TuningID, Unknown3
                    ObjectProperty::UInt64(cursor.read_le::<u64>()?)
                }
                0xCADED888 | 0xE206AE4F | 0x8A85AFF3 | 0x8D20ACC6 | 0x6C737AD8 => { // Icon, Rig, Slot, Model, Footprint
                    let byte_count = cursor.read_le::<u32>()?;
                    let count = byte_count / 16; // 16 bytes per TGI (8 + 4 + 4)
                    let mut tgis = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        // Swapped ITG order in legacy code: instance(8), type(4), group(4)
                        let mut instance = cursor.read_le::<u64>()?;
                        instance = (instance << 32) | (instance >> 32); // swap hi/lo
                        let res_type = cursor.read_le::<u32>()?;
                        let res_group = cursor.read_le::<u32>()?;
                        tgis.push(TGI { res_type, res_group, instance });
                    }
                    ObjectProperty::TGIBlockList(tgis)
                }
                0xE6E421FB => { // Components
                    let count = cursor.read_le::<u32>()?;
                    let mut components = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        components.push(cursor.read_le::<u32>()?);
                    }
                    ObjectProperty::UInt32List(components)
                }
                0xAC8E1BC0 => { // Unknown1
                    ObjectProperty::Byte(cursor.read_le::<u8>()?)
                }
                0xE4F4FAA4 | 0x4233F8A0 => { // SimoleonPrice, ThumbnailGeometryState
                    ObjectProperty::UInt32(cursor.read_le::<u32>()?)
                }
                0x7236BEEA | 0x44FC7512 => { // PositiveEnvironmentScore, NegativeEnvironmentScore
                    ObjectProperty::Float(cursor.read_le::<f32>()?)
                }
                0xEC3712E6 | 0xAEE67A1C => { // Unknown2, IsBaby
                    ObjectProperty::Bool(cursor.read_le::<u8>()? != 0)
                }
                0x2172AEBE => { // EnvironmentScoreEmotionTags
                    let count = cursor.read_le::<u32>()?;
                    let mut tags = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        tags.push(cursor.read_le::<u16>()?);
                    }
                    ObjectProperty::UInt16List(tags)
                }
                0xDCD08394 => { // EnvironmentScores
                    let count = cursor.read_le::<u32>()?;
                    let mut scores = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        scores.push(cursor.read_le::<f32>()?);
                    }
                    ObjectProperty::FloatList(scores)
                }
                0xF3936A90 => { // Unknown4
                    let len = cursor.read_le::<u32>()?;
                    let mut buf = vec![0u8; len as usize];
                    cursor.read_exact(&mut buf)?;
                    ObjectProperty::ByteList(buf)
                }
                _ => {
                    // We don't know the size for unknown properties easily without reading the whole file
                    // But we can try to infer it if it's the last property or by looking at next offset
                    // For now, let's just mark it as Unknown
                    ObjectProperty::Unknown(vec![])
                }
            };
            properties.insert(prop_id, property);
        }

        Ok(Self { version, properties })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        // Implementation of writing would be complex due to table and offsets.
        // For now, let's just return an error or a placeholder.
        // Since we are primarily interested in "Investigate" mode (reading),
        // we can leave this for later if merging requires re-writing modified resources.
        Err(anyhow::anyhow!("Writing ObjectDefinitionResource not yet implemented"))
    }
}

/// String Table resource (0x220557AA)
#[binrw]
#[derive(Debug)]
#[br(little, magic = b"STBL")]
#[bw(little, magic = b"STBL")]
pub struct StblResource {
    pub version: u16,
    pub is_compressed: u8,
    #[br(temp)]
    #[bw(calc = entries.len() as u64)]
    count: u64,
    pub reserved: [u8; 2],
    pub string_length: u32,
    #[br(count = count)]
    pub entries: Vec<StblEntry>,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct StblEntry {
    pub key_hash: u32,
    pub flags: u8,
    #[br(temp)]
    #[bw(calc = string_value.len() as u16)]
    length: u16,
    #[br(count = length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).into_owned())]
    #[bw(map = |s: &String| s.as_bytes().to_vec())]
    pub string_value: String,
}

impl Resource for StblResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read StblResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write StblResource")?;
        Ok(data)
    }
}


/// Catalog resource (COBJ 0x319E4F1D, CSTL 0x9F5CFF10, etc.)
#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct CatalogResource {
    pub version: u32,
    pub common: CatalogCommon,
    pub aural_materials_version: u32,
    #[br(temp)]
    #[bw(calc = 0)]
    _aural_materials1: u32,
    #[br(temp)]
    #[bw(calc = 0)]
    _aural_materials2: u32,
    #[br(temp)]
    #[bw(calc = 0)]
    _aural_materials3: u32,
    pub aural_properties_version: u32,
    #[br(temp)]
    #[bw(calc = 0)]
    _aural_quality: u32,
    #[br(if(aural_properties_version > 1))]
    pub aural_ambient_object: Option<u32>,
    #[br(if(aural_properties_version == 3))]
    pub ambience_file_instance_id: Option<u64>,
    #[br(if(aural_properties_version == 3))]
    pub is_override_ambience: Option<u8>,
    #[br(if(aural_properties_version == 4))]
    pub unknown01: Option<u8>,
    #[br(temp)]
    #[bw(calc = 0)]
    _unused0: u32,
    #[br(temp)]
    #[bw(calc = 0)]
    _unused1: u32,
    #[br(temp)]
    #[bw(calc = 0)]
    _unused2: u32,
    pub placement_flags_high: u32,
    pub placement_flags_low: u32,
    pub slot_type_set: u64,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct CatalogCommon {
    pub version: u32,
    pub name_hash: u32,
    pub description_hash: u32,
    pub price: u32,
    pub thumbnail_hash: u64,
    pub dev_category_flags: u32,
    #[br(temp)]
    #[bw(calc = product_styles.len() as u8)]
    product_styles_count: u8,
    #[br(count = product_styles_count)]
    pub product_styles: Vec<TGI>, // ITG swapped in C#
    #[br(if(version >= 10))]
    pub pack_id: Option<i16>,
    #[br(if(version >= 10))]
    pub pack_flags: Option<u8>,
    #[br(if(version >= 10), count = 9)]
    pub reserved_bytes: Option<Vec<u8>>,
    #[br(if(version < 10))]
    pub unused2: Option<u8>,
    #[br(if(version < 10 && unused2.unwrap_or(0) > 0))]
    pub unused3: Option<u8>,
    #[br(if(version >= 11))]
    pub tags: Option<CatalogTagList>,
    #[br(if(version < 11))]
    pub legacy_tags: Option<LegacyTagList>,
    pub selling_points: SellingPointList,
    pub unlock_by_hash: u32,
    pub unlocked_by_hash: u32,
    pub swatch_colors_sort_priority: u16,
    pub varient_thumb_image_hash: u64,
}

#[binrw]
#[derive(Debug)]
pub struct CatalogTagList {
    #[br(temp)]
    #[bw(calc = tags.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub tags: Vec<u16>,
}

#[binrw]
#[derive(Debug)]
pub struct LegacyTagList {
    #[br(calc = 0)]
    #[bw(calc = 0)]
    pub _version: u32,
    #[br(temp)]
    #[bw(calc = tags.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub tags: Vec<u16>,
}

#[binrw]
#[derive(Debug)]
pub struct SellingPointList {
    #[br(temp)]
    #[bw(calc = points.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub points: Vec<SellingPoint>,
}

#[binrw]
#[derive(Debug)]
pub struct SellingPoint {
    pub hash: u32,
    pub value: f32,
}

impl Resource for CatalogResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read_le(&mut cursor).context("Failed to read CatalogResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write_le(&mut cursor).context("Failed to write CatalogResource")?;
        Ok(data)
    }
}


/// RLE Image resource (0x3453CF95)
#[derive(Debug)]
pub struct RleResource {
    pub magic: [u8; 4],
    pub version: u32,
    pub width: u16,
    pub height: u16,
    pub mip_count: u16,
}

impl Resource for RleResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        let version = cursor.read_le::<u32>()?;
        let width = cursor.read_le::<u16>()?;
        let height = cursor.read_le::<u16>()?;
        let mip_count = cursor.read_le::<u16>()?;
        
        Ok(Self { magic, version, width, height, mip_count })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Writing RleResource not yet implemented"))
    }
}

/// DST Texture resource (0x00B2D882)
#[derive(Debug)]
pub struct DstResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for DstResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = cursor.read_le::<u32>()?;
        Ok(Self { version, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}


/// Script resource (Encrypted Signed Assembly 0x073FAA07)
#[derive(Debug)]
pub struct ScriptResource {
    pub version: u8,
    pub game_version: String,
}

impl Resource for ScriptResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = cursor.read_le::<u8>()?;
        let game_version = if version > 1 {
            let len = cursor.read_le::<i32>()?;
            let mut buf = vec![0u8; (len * 2) as usize];
            cursor.read_exact(&mut buf)?;
            // UTF-16LE in legacy code
            let utf16: Vec<u16> = buf.chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&utf16)
        } else {
            String::new()
        };
        
        Ok(Self { version, game_version })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Writing ScriptResource not yet implemented"))
    }
}

/// Thumbnail resource
#[derive(Debug)]
pub struct ThumbnailResource {
    pub has_alpha: bool,
    pub raw_data: Vec<u8>,
}

impl Resource for ThumbnailResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut has_alpha = false;
        if data.len() > 28 {
            let mut cursor = Cursor::new(data);
            if cursor.seek(SeekFrom::Start(24)).is_ok() {
                if let Ok(magic) = cursor.read_le::<u32>() {
                    if magic == 0x41464C41 {
                        has_alpha = true;
                    }
                }
            }
        }
        Ok(Self { has_alpha, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// Complate resource (0x044AE110)
#[derive(Debug)]
pub struct ComplateResource {
    pub unknown1: u32,
    pub content: String,
    pub unknown2: u32,
}

impl Resource for ComplateResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let unknown1 = cursor.read_le::<u32>()?;
        let len = cursor.read_le::<i32>()?;
        let mut buf = vec![0u8; (len * 2) as usize];
        cursor.read_exact(&mut buf)?;
        let utf16: Vec<u16> = buf.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let content = String::from_utf16_lossy(&utf16);
        let unknown2 = cursor.read_le::<u32>()?;
        Ok(Self { unknown1, content, unknown2 })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        cursor.write_le(&self.unknown1)?;
        let utf16: Vec<u16> = self.content.encode_utf16().collect();
        cursor.write_le(&(utf16.len() as i32))?;
        for &u in &utf16 {
            cursor.write_le(&u)?;
        }
        cursor.write_le(&self.unknown2)?;
        Ok(data)
    }
}

/// Txtc resource (0x033A1435, 0x0341ACC9)
#[derive(Debug)]
pub struct TxtcResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for TxtcResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = cursor.read_le::<u32>()?;
        Ok(Self { version, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// ObjKey resource (0x02DC343F)
#[derive(Debug)]
pub struct ObjKeyResource {
    pub format: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for ObjKeyResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let format = cursor.read_le::<u32>()?;
        Ok(Self { format, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// SimModifier resource (0xC5F6763E)
#[derive(Debug)]
pub struct SimModifierResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for SimModifierResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        // contexData comes first in legacy, skip it for basic identification
        // Actually, let's just grab the version which is at a certain offset
        // Parse in legacy:
        // this.contexData = new ContexData(recommendedApiVersion, OnResourceChanged, s);
        // this.version = r.ReadUInt32();
        
        // ContexData header is 5 u32s, then 3 CountedTGIBlockList, then ObjectDataLIst
        // This is complex. Let's just store raw_data for now.
        Ok(Self { version: 0, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// Bone resource (0x00AE6C67)
#[derive(Debug)]
pub struct BoneResource {
    pub version: u32,
    pub raw_data: Vec<u8>,
}

impl Resource for BoneResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let version = if data.len() >= 4 {
            cursor.read_le::<u32>().unwrap_or(0)
        } else {
            0
        };
        Ok(Self { version, raw_data: data.to_vec() })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_data.clone())
    }
}

/// Geometry resource (0x015A1849)
#[binrw]
#[derive(Debug)]
#[br(little, magic = b"GEOM")]
#[bw(little, magic = b"GEOM")]
pub struct GeomResource {
    pub version: u32,
    pub tgi_offset: u32,
    pub tgi_size: u32,
    pub embedded_id: u32,
    #[br(if(embedded_id != 0))]
    pub mtnf: Option<GeomMtnf>,
    pub merge_group: u32,
    pub sort_order: u32,
    #[br(temp)]
    #[bw(calc = vertex_data.vertices.len() as u32)]
    pub num_verts: u32,
    pub vertex_formats: GeomVertexFormatList,
    #[br(args(num_verts, &vertex_formats))]
    pub vertex_data: GeomVertexDataList,
    #[br(temp)]
    #[bw(calc = 1)]
    pub item_count: u32,
    #[br(count = item_count)]
    pub bytes_per_face_point: Vec<u8>,
    #[br(temp)]
    #[bw(calc = faces.faces.len() as u32 * 3)]
    pub num_face_points: u32,
    #[br(args(num_face_points, &bytes_per_face_point))]
    pub faces: GeomFaceList,
    #[br(if(version == 0x05))]
    pub skin_index: Option<i32>,
    #[br(if(version == 0x0C))]
    pub unknown_things: Option<GeomUnknownThingList>,
    #[br(if(version == 0x0C))]
    pub unknown_things2: Option<GeomUnknownThing2List>,
    pub bone_hashes: GeomBoneHashList,
    #[br(seek_before = SeekFrom::Start(tgi_offset as u64 + 16), count = tgi_size / 16)]
    pub tgi_blocks: Vec<TGI>,
}

#[binrw]
#[derive(Debug)]
pub struct GeomMtnf {
    pub size: u32,
    #[br(count = size)]
    pub data: Vec<u8>,
}

#[binrw]
#[derive(Debug)]
pub struct GeomVertexFormatList {
    #[br(temp)]
    #[bw(calc = formats.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub formats: Vec<GeomVertexFormat>,
}

#[binrw]
#[derive(Debug)]
pub struct GeomVertexFormat {
    pub usage: u32,
    pub data_type: u32,
    pub element_size: u8,
}

#[derive(Debug)]
pub struct GeomVertexDataList {
    pub vertices: Vec<Vec<u8>>,
}

impl BinRead for GeomVertexDataList {
    type Args<'a> = (u32, &'a GeomVertexFormatList);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _options: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let (num_verts, formats) = args;
        let stride: usize = formats.formats.iter().map(|f| f.element_size as usize).sum();
        let mut vertices = Vec::with_capacity(num_verts as usize);
        for _ in 0..num_verts {
            let mut buf = vec![0u8; stride];
            reader.read_exact(&mut buf)?;
            vertices.push(buf);
        }
        Ok(Self { vertices })
    }
}

impl BinWrite for GeomVertexDataList {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _options: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        for v in &self.vertices {
            writer.write_all(v)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct GeomFaceList {
    pub faces: Vec<[u16; 3]>,
}

impl BinRead for GeomFaceList {
    type Args<'a> = (u32, &'a Vec<u8>);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _options: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let (num_face_points, bytes_per_face_point) = args;
        let bpf = if !bytes_per_face_point.is_empty() { bytes_per_face_point[0] } else { 2 };
        
        let mut faces = Vec::with_capacity((num_face_points / 3) as usize);
        for _ in 0..(num_face_points / 3) {
            let mut face = [0u16; 3];
            for j in 0..3 {
                if bpf == 1 {
                    face[j] = reader.read_le::<u8>()? as u16;
                } else {
                    face[j] = reader.read_le::<u16>()?;
                }
            }
            faces.push(face);
        }
        Ok(Self { faces })
    }
}

impl BinWrite for GeomFaceList {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        // Implement writing if needed
        Ok(())
    }
}

#[binrw]
#[derive(Debug)]
pub struct GeomUnknownThingList {
    #[br(temp)]
    #[bw(calc = things.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub things: Vec<GeomUnknownThing>,
}

#[binrw]
#[derive(Debug)]
pub struct GeomUnknownThing {
    pub unknown1: u32,
    #[br(temp)]
    #[bw(calc = unknown2.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub unknown2: Vec<[f32; 2]>,
}

#[binrw]
#[derive(Debug)]
pub struct GeomUnknownThing2List {
    #[br(temp)]
    #[bw(calc = things.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub things: Vec<GeomUnknownThing2>,
}

#[binrw]
#[derive(Debug)]
pub struct GeomUnknownThing2 {
    pub unknown1: u32,
    pub unknown2: u16,
    pub unknown3: u16,
    pub unknown4: u16,
    pub unknown5: [f32; 13],
    pub unknown18: u8,
}

#[binrw]
#[derive(Debug)]
pub struct GeomBoneHashList {
    #[br(temp)]
    #[bw(calc = hashes.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub hashes: Vec<u32>,
}

impl Resource for GeomResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read GeomResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write GeomResource")?;
        Ok(data)
    }
}

/// Manifest resource (0x7FB6AD8A or 0x73E93EEB)
#[binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[br(little)]
#[bw(little)]
pub struct ManifestTGI {
    pub instance: u64,
    pub res_type: u32,
    pub res_group: u32,
}

impl From<ManifestTGI> for TGI {
    fn from(m: ManifestTGI) -> Self {
        Self {
            res_type: m.res_type,
            res_group: m.res_group,
            instance: m.instance,
        }
    }
}

impl From<TGI> for ManifestTGI {
    fn from(t: TGI) -> Self {
        Self {
            res_type: t.res_type,
            res_group: t.res_group,
            instance: t.instance,
        }
    }
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct ManifestResource {
    pub version: u32,
    pub padding: u64,
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,
    #[br(count = entry_count)]
    pub entries: Vec<ManifestEntry>,
}

#[binrw]
#[derive(Debug)]
#[br(little)]
#[bw(little)]
pub struct ManifestEntry {
    #[br(temp)]
    #[bw(calc = name.len() as u32)]
    pub name_len: u32,
    #[br(count = name_len, map = |s: Vec<u8>| String::from_utf8_lossy(&s).into_owned())]
    #[bw(map = |s: &String| s.as_bytes().to_vec())]
    pub name: String,
    #[br(temp)]
    #[bw(calc = resources.len() as u32)]
    pub resource_count: u32,
    #[br(count = resource_count, map = |v: Vec<ManifestTGI>| v.into_iter().map(TGI::from).collect())]
    #[bw(map = |v: &Vec<TGI>| v.iter().map(|&t| ManifestTGI::from(t)).collect::<Vec<_>>())]
    pub resources: Vec<TGI>,
}

impl Resource for ManifestResource {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Self::read(&mut cursor).context("Failed to read ManifestResource")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        self.write(&mut cursor).context("Failed to write ManifestResource")?;
        Ok(data)
    }
}
