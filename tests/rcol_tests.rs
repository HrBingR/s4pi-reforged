use s4pi_reforged::{Resource, RcolResource, TGI};

#[test]
fn test_rcol_parsing() {
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_le_bytes()); // version
    data.extend_from_slice(&1i32.to_le_bytes()); // public_chunks
    data.extend_from_slice(&0u32.to_le_bytes()); // unused
    data.extend_from_slice(&1i32.to_le_bytes()); // count_resources
    data.extend_from_slice(&1i32.to_le_bytes()); // count_chunks

    // Chunk TGIs
    data.extend_from_slice(&0x015A1849u32.to_le_bytes()); // type (GEOM)
    data.extend_from_slice(&0x00000000u32.to_le_bytes()); // group
    data.extend_from_slice(&0x123456789ABCDEF0u64.to_le_bytes()); // instance

    // External Resources
    data.extend_from_slice(&0x220557AAu32.to_le_bytes()); // type (STBL)
    data.extend_from_slice(&0x00000000u32.to_le_bytes()); // group
    data.extend_from_slice(&0xFFFFFFFFFFFFFFFFu64.to_le_bytes()); // instance

    // Chunk Index
    let chunk_pos = data.len() as u32 + 8;
    data.extend_from_slice(&chunk_pos.to_le_bytes()); // position
    data.extend_from_slice(&8i32.to_le_bytes()); // length

    // Chunk Data (GEOM chunk starts with "GEOM")
    data.extend_from_slice(b"GEOM");
    data.extend_from_slice(&0x0000000Cu32.to_le_bytes()); // GEOM version

    let rcol = RcolResource::from_bytes(&data).unwrap();
    assert_eq!(rcol.version, 1);
    assert_eq!(rcol.public_chunks, 1);
    assert_eq!(rcol.external_resources.len(), 1);
    assert_eq!(rcol.chunks.len(), 1);
    assert_eq!(rcol.chunks[0].tag, "GEOM");
    assert_eq!(rcol.chunks[0].tgi.res_type, 0x015A1849);
}
