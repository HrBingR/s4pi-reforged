use s4pi_reforged::{TypedResource, ObjectProperty};
use std::io::{Write, Cursor};

#[test]
fn test_object_definition_parsing() {
    let mut data = Vec::new();
    let mut cursor = Cursor::new(&mut data);
    
    // Header
    cursor.write_all(&1u16.to_le_bytes()).unwrap(); // version
    cursor.write_all(&6u32.to_le_bytes()).unwrap(); // table offset
    
    // Property Table
    cursor.write_all(&1u16.to_le_bytes()).unwrap(); // entry count
    cursor.write_all(&0xE7F07786u32.to_le_bytes()).unwrap(); // PropertyID::Name
    cursor.write_all(&16u32.to_le_bytes()).unwrap(); // offset to data
    
    // Property Data (Name)
    cursor.write_all(&4u32.to_le_bytes()).unwrap(); // string len
    cursor.write_all(b"Test").unwrap();
    
    let res = TypedResource::from_bytes(0xC0DB5AE7, &data).unwrap();
    if let TypedResource::ObjectDefinition(obj) = res {
        assert_eq!(obj.version, 1);
        assert_eq!(obj.properties.len(), 1);
        if let Some(ObjectProperty::String(name)) = obj.properties.get(&0xE7F07786) {
            assert_eq!(name, "Test");
        } else {
            panic!("Expected String property for Name");
        }
    } else {
        panic!("Expected ObjectDefinition resource");
    }
}
