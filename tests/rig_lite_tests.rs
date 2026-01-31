use s4pi_reforged::{Resource, RigResource, LiteResource};

#[test]
fn test_rig_parsing() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x8EAF13DEu32.to_le_bytes()); // dw1
    data.extend_from_slice(&0x00000000u32.to_le_bytes()); // dw2
    data.extend_from_slice(&[0u8; 10]); // some data

    let rig = RigResource::from_bytes(&data).unwrap();
    assert_eq!(rig.format, "WrappedGranny");
}

#[test]
fn test_lite_parsing() {
    let mut data = Vec::new();
    data.extend_from_slice(b"LITE");
    data.extend_from_slice(&4u32.to_le_bytes()); // version
    data.extend_from_slice(&[0u8; 10]); // some data

    let lite = LiteResource::from_bytes(&data).unwrap();
    assert_eq!(lite.version, 4);
}
