use s4pi_reforged::{Resource, ThumbnailResource, ComplateResource, TxtcResource, ObjKeyResource, SimModifierResource, BoneResource};

#[test]
fn test_thumbnail_parsing() {
    let mut data = vec![0u8; 32];
    data[24..28].copy_from_slice(&0x41464C41u32.to_le_bytes());
    
    let thumb = ThumbnailResource::from_bytes(&data).unwrap();
    assert!(thumb.has_alpha);
    assert_eq!(thumb.raw_data, data);
    
    let to_bytes = thumb.to_bytes().unwrap();
    assert_eq!(to_bytes, data);
}

#[test]
fn test_complate_parsing() {
    let mut data = Vec::new();
    data.extend_from_slice(&2u32.to_le_bytes()); // unknown1
    let content = "Hello";
    let utf16: Vec<u16> = content.encode_utf16().collect();
    data.extend_from_slice(&(utf16.len() as i32).to_le_bytes());
    for &u in &utf16 {
        data.extend_from_slice(&u.to_le_bytes());
    }
    data.extend_from_slice(&0u32.to_le_bytes()); // unknown2
    
    let complate = ComplateResource::from_bytes(&data).unwrap();
    assert_eq!(complate.unknown1, 2);
    assert_eq!(complate.content, "Hello");
    assert_eq!(complate.unknown2, 0);
    
    let to_bytes = complate.to_bytes().unwrap();
    assert_eq!(to_bytes, data);
}

#[test]
fn test_txtc_parsing() {
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&7u32.to_le_bytes());
    
    let txtc = TxtcResource::from_bytes(&data).unwrap();
    assert_eq!(txtc.version, 7);
    
    let to_bytes = txtc.to_bytes().unwrap();
    assert_eq!(to_bytes, data);
}

#[test]
fn test_obj_key_parsing() {
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&7u32.to_le_bytes());
    
    let obj_key = ObjKeyResource::from_bytes(&data).unwrap();
    assert_eq!(obj_key.format, 7);
    
    let to_bytes = obj_key.to_bytes().unwrap();
    assert_eq!(to_bytes, data);
}

#[test]
fn test_sim_modifier_parsing() {
    let data = vec![0u8; 100];
    let sim_mod = SimModifierResource::from_bytes(&data).unwrap();
    assert_eq!(sim_mod.raw_data, data);
}

#[test]
fn test_bone_parsing() {
    let mut data = vec![0u8; 4];
    data[0..4].copy_from_slice(&1u32.to_le_bytes());
    let bone = BoneResource::from_bytes(&data).unwrap();
    assert_eq!(bone.version, 1);
    assert_eq!(bone.raw_data, data);
}
