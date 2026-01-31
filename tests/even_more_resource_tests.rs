use s4pi_reforged::{ClipResource, CasPartResource, JazzResource, Resource};

#[test]
fn test_clip_parsing() {
    let mut data = vec![];
    data.extend_from_slice(&14u32.to_le_bytes()); // Version
    data.extend_from_slice(&[0u8; 100]); // Some dummy data

    let res = ClipResource::from_bytes(&data).unwrap();
    assert_eq!(res.version, 14);
}

#[test]
fn test_caspart_parsing() {
    let mut data = vec![];
    data.extend_from_slice(&27u32.to_le_bytes()); // Version 0x1B
    data.extend_from_slice(&[0u8; 100]); // Some dummy data

    let res = CasPartResource::from_bytes(&data).unwrap();
    assert_eq!(res.version, 27);
}

#[test]
fn test_jazz_parsing() {
    let data = vec![1, 2, 3, 4, 5];
    let res = JazzResource::from_bytes(&data).unwrap();
    assert_eq!(res.raw_data, data);
    assert_eq!(res.to_bytes().unwrap(), data);
}
