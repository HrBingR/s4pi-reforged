use s4pi_reforged::{CatalogResource, RleResource, DstResource, Resource};

#[test]
fn test_catalog_parsing() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x19u32.to_le_bytes()); // version
    data.extend_from_slice(&0x09u32.to_le_bytes()); // common version
    data.extend_from_slice(&0x11223344u32.to_le_bytes()); // name hash
    data.extend_from_slice(&0x55667788u32.to_le_bytes()); // desc hash
    data.extend_from_slice(&100u32.to_le_bytes()); // price
    data.extend_from_slice(&0x123456789ABCDEF0u64.to_le_bytes()); // thumbnail hash
    
    let res = CatalogResource::from_bytes(&data).unwrap();
    assert_eq!(res.version, 0x19);
    assert_eq!(res.common.version, 0x09);
    assert_eq!(res.common.name_hash, 0x11223344);
    assert_eq!(res.common.thumbnail_hash, 0x123456789ABCDEF0);
}

#[test]
fn test_rle_parsing() {
    let mut data = Vec::new();
    data.extend_from_slice(b"RLES");
    data.extend_from_slice(&0x00020002u32.to_le_bytes()); // version
    data.extend_from_slice(&128u16.to_le_bytes()); // width
    data.extend_from_slice(&256u16.to_le_bytes()); // height
    data.extend_from_slice(&5u16.to_le_bytes()); // mips
    
    let res = RleResource::from_bytes(&data).unwrap();
    assert_eq!(&res.magic, b"RLES");
    assert_eq!(res.width, 128);
    assert_eq!(res.height, 256);
}

#[test]
fn test_dst_parsing() {
    let mut data = Vec::new();
    data.extend_from_slice(&2u32.to_le_bytes()); // version
    data.extend_from_slice(&[0xAA, 0xBB]); // some data
    
    let res = DstResource::from_bytes(&data).unwrap();
    assert_eq!(res.version, 2);
    assert_eq!(res.raw_data.len(), 6);
}
