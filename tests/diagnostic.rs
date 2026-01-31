use s4pi_reforged::Package;
use std::env;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: diagnostic <package_path>");
        return Ok(());
    }

    let path = Path::new(&args[1]);
    let mut pkg = Package::open(path)?;

    println!("Package: {}", path.display());
    println!("Header: {:?}", pkg.header);
    println!("Index Count: {}", pkg.entries.len());

    for (i, entry) in pkg.entries.iter().enumerate().take(10) {
        println!("\nEntry {}:", i);
        println!("  TGI: {:08X}:{:08X}:{:016X}", entry.tgi.res_type, entry.tgi.res_group, entry.tgi.instance);
        println!("  Offset: 0x{:08X}", entry.offset);
        println!("  Filesize: {} (0x{:08X})", entry.filesize, entry.filesize);
        println!("  Memsize: {} (0x{:08X})", entry.memsize, entry.memsize);
        println!("  Compression: 0x{:04X}", entry.compression);
        println!("  Committed: 0x{:04X}", entry.committed);

        let mut file = std::fs::File::open(path)?;
        use std::io::{Seek, SeekFrom, Read};
        file.seek(SeekFrom::Start(entry.offset as u64))?;
        let mut head = [0u8; 8];
        file.read_exact(&mut head)?;
        println!("  Data Head: {:02X?}", head);
    }

    Ok(())
}
