use s4pi_reforged::{Package, TypedResource, NameMapResource, StblResource, ObjectDefinitionResource, SimDataResource, TextResource};
use std::fs::File;
use std::io::{Write, Cursor};
use binrw::{BinWrite, BinRead, BinReaderExt};

#[test]
fn test_stbl_parsing() {
    let mut data = Vec::new();
    let mut cursor = Cursor::new(&mut data);
    
    // Write STBL header
    cursor.write_all(b"STBL").unwrap();
    cursor.write_all(&5u16.to_le_bytes()).unwrap(); // version
    cursor.write_all(&0u8.to_le_bytes()).unwrap();  // is_compressed
    cursor.write_all(&1u64.to_le_bytes()).unwrap(); // count
    cursor.write_all(&[0u8, 0u8]).unwrap();         // reserved
    cursor.write_all(&6u32.to_le_bytes()).unwrap(); // string_length (length + 1 for null terminator if any, but our wrapper uses length)
    
    // Write entry
    cursor.write_all(&0x12345678u32.to_le_bytes()).unwrap(); // key_hash
    cursor.write_all(&0u8.to_le_bytes()).unwrap();           // flags
    cursor.write_all(&5u16.to_le_bytes()).unwrap();          // length
    cursor.write_all(b"Hello").unwrap();                     // string
    
    let res = TypedResource::from_bytes(0x220557AA, &data).unwrap();
    if let TypedResource::Stbl(stbl) = res {
        assert_eq!(stbl.version, 5);
        assert_eq!(stbl.entries.len(), 1);
        assert_eq!(stbl.entries[0].string_value, "Hello");
    } else {
        panic!("Expected Stbl resource");
    }
}

#[test]
fn test_simdata_parsing() {
    let mut data = Vec::new();
    let mut cursor = Cursor::new(&mut data);
    
    cursor.write_all(b"DATA").unwrap();
    cursor.write_all(&0x100u32.to_le_bytes()).unwrap(); // version
    // ... rest of simdata doesn't matter for our basic wrapper yet
    cursor.write_all(&[0u8; 20]).unwrap();
    
    let res = TypedResource::from_bytes(0x545AC67A, &data).unwrap();
    if let TypedResource::SimData(sim) = res {
        assert_eq!(sim.version, 0x100);
    } else {
        panic!("Expected SimData resource");
    }
}

#[test]
fn test_text_parsing() {
    let data = b"<?xml version=\"1.0\"?><tuning>Content</tuning>";
    let res = TypedResource::from_bytes(0x034AEECB, data).unwrap();
    if let TypedResource::Text(text) = res {
        assert_eq!(text.content, String::from_utf8_lossy(data));
    } else {
        panic!("Expected Text resource");
    }
}
