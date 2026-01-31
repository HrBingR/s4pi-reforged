use s4pi_reforged::package::resource::{RcolResource, Resource};

#[test]
fn test_rcol_bounds_check() {
    // Version=3, PublicChunks=1, Unused=0, CountRes=1000000, CountChunks=0
    let mut data = vec![0u8; 100];
    data[0..4].copy_from_slice(&3u32.to_le_bytes()); // version
    data[4..8].copy_from_slice(&1i32.to_le_bytes()); // public_chunks
    data[8..12].copy_from_slice(&0u32.to_le_bytes()); // unused
    data[12..16].copy_from_slice(&1000000i32.to_le_bytes()); // count_resources
    data[16..20].copy_from_slice(&0i32.to_le_bytes()); // count_chunks

    let res = RcolResource::from_bytes(&data);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Invalid RCOL header: count too large for data size");
}

#[test]
fn test_rcol_chunk_bounds_check() {
    // Version=3, PublicChunks=1, Unused=0, CountRes=0, CountChunks=1
    let mut data = vec![0u8; 100];
    data[0..4].copy_from_slice(&3u32.to_le_bytes()); // version
    data[4..8].copy_from_slice(&1i32.to_le_bytes()); // public_chunks
    data[8..12].copy_from_slice(&0u32.to_le_bytes()); // unused
    data[12..16].copy_from_slice(&0i32.to_le_bytes()); // count_resources
    data[16..20].copy_from_slice(&1i32.to_le_bytes()); // count_chunks

    // TGI for chunk 1
    data[20..24].copy_from_slice(&0u32.to_le_bytes());
    data[24..28].copy_from_slice(&0u32.to_le_bytes());
    data[28..36].copy_from_slice(&0u64.to_le_bytes());

    // Index for chunk 1: position=0, length=1000000
    data[36..40].copy_from_slice(&0u32.to_le_bytes());
    data[40..44].copy_from_slice(&1000000i32.to_le_bytes());

    let res = RcolResource::from_bytes(&data);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("RCOL chunk extends beyond data bounds"));
}
