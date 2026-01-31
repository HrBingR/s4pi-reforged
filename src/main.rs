use s4pi_reforged::{Package, TGI, TypedResource};
use rfd::FileDialog;
use std::collections::{HashMap, HashSet};
use std::path::{Path};
use walkdir::WalkDir;
use anyhow::{Result, Context, anyhow};
use log::{info, error, warn};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

#[cfg(windows)]
fn prepare_console() {
    extern "system" {
        fn AllocConsole() -> i32;
        fn GetConsoleWindow() -> isize;
    }
    unsafe {
        if GetConsoleWindow() == 0 {
            let _ = AllocConsole();
        }
    }
}

#[cfg(not(windows))]
fn prepare_console() {
    if !atty::is(atty::Stream::Stdout) {
        if std::env::var("S4PI_TERMINAL_RELAUNCH").is_err() {
            let exe = std::env::current_exe().expect("Failed to get current exe path");
            let mut command = None;

            // Try common terminal emulators
            for term in &["x-terminal-emulator", "gnome-terminal", "konsole", "xfce4-terminal", "xterm"] {
                if which::which(term).is_ok() {
                    let mut cmd = std::process::Command::new(term);
                    match *term {
                        "gnome-terminal" => {
                            cmd.arg("--").arg(&exe);
                        }
                        "konsole" => {
                            cmd.arg("-e").arg(&exe);
                        }
                        "xfce4-terminal" => {
                            cmd.arg("-e").arg(&exe);
                        }
                        _ => {
                            cmd.arg("-e").arg(&exe);
                        }
                    }
                    command = Some(cmd);
                    break;
                }
            }

            if let Some(mut cmd) = command {
                cmd.env("S4PI_TERMINAL_RELAUNCH", "1");
                if cmd.spawn().is_ok() {
                    std::process::exit(0);
                }
            }
        }
    }
}

fn is_debug_mode() -> bool {
    std::env::var("S4PI_DEBUG_MODE").map(|v| v == "1").unwrap_or(false)
}

struct GuiApp {
    log_buffer: Arc<Mutex<String>>,
}

impl GuiApp {
    fn new(_cc: &eframe::CreationContext<'_>, log_buffer: Arc<Mutex<String>>) -> Self {
        Self { log_buffer }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Merge").clicked() {
                    let folder = FileDialog::new()
                        .set_title("Select Folder containing .package files")
                        .pick_folder();
                    if let Some(f) = folder {
                        let log_arc = Arc::clone(&self.log_buffer);
                        std::thread::spawn(move || {
                            if let Err(e) = run_merge(&f) {
                                let mut log = log_arc.lock().unwrap();
                                log.push_str(&format!("Error during merge: {:?}\n", e));
                            }
                        });
                    }
                }

                if ui.button("Un-merge").clicked() {
                    let file = FileDialog::new()
                        .set_title("Select .package file to un-merge")
                        .add_filter("Package Files", &["package"])
                        .pick_file();
                    if let Some(f) = file {
                        let log_arc = Arc::clone(&self.log_buffer);
                        std::thread::spawn(move || {
                            if let Err(e) = run_unmerge(&f) {
                                let mut log = log_arc.lock().unwrap();
                                log.push_str(&format!("Error during un-merge: {:?}\n", e));
                            }
                        });
                    }
                }

                ui.menu_button("Extract", |ui| {
                    if ui.button("Thumbnail").clicked() {
                        let file = FileDialog::new()
                            .set_title("Select .package file to extract thumbnails")
                            .add_filter("Package Files", &["package"])
                            .pick_file();
                        if let Some(f) = file {
                            let log_arc = Arc::clone(&self.log_buffer);
                            std::thread::spawn(move || {
                                if let Err(e) = run_extract_thumbnails(&f) {
                                    let mut log = log_arc.lock().unwrap();
                                    log.push_str(&format!("Error during extraction: {:?}\n", e));
                                }
                            });
                        }
                        ui.close_menu();
                    }
                });

                if is_debug_mode() {
                    ui.menu_button("Advanced", |ui| {
                        if ui.button("Investigate").clicked() {
                            let file = FileDialog::new()
                                .set_title("Select .package file to investigate")
                                .add_filter("Package Files", &["package"])
                                .pick_file();
                            if let Some(f) = file {
                                let log_arc = Arc::clone(&self.log_buffer);
                                std::thread::spawn(move || {
                                    if let Err(e) = run_investigate(&f) {
                                        let mut log = log_arc.lock().unwrap();
                                        log.push_str(&format!("Error during investigation: {:?}\n", e));
                                    }
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Diagnostics").clicked() {
                            let file = FileDialog::new()
                                .set_title("Select .package file for diagnostics")
                                .add_filter("Package Files", &["package"])
                                .pick_file();
                            if let Some(f) = file {
                                let log_arc = Arc::clone(&self.log_buffer);
                                std::thread::spawn(move || {
                                    if let Err(e) = run_diagnostics(&f) {
                                        let mut log = log_arc.lock().unwrap();
                                        log.push_str(&format!("Error during diagnostics: {:?}\n", e));
                                    }
                                });
                            }
                            ui.close_menu();
                        }
                    });
                }

                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("S4PI Tool");

            ui.label("Console Output:");
            let log_text = self.log_buffer.lock().unwrap();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut log_text.clone())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                });
            drop(log_text);
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

struct LogWriter {
    buffer: Arc<Mutex<String>>,
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            let mut log = self.buffer.lock().unwrap();
            log.push_str(s);
        }
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let log_buffer = Arc::new(Mutex::new(String::new()));

    if args.len() > 1 {
        // CLI Mode
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
        
        let debug = is_debug_mode();
        let cmd = args[1].as_str();

        match cmd {
            "merge" => {
                if args.iter().any(|a| a == "--help") {
                    println!("Usage: s4pi-reforged merge <folder>");
                    println!("\nMerges all .package files in the specified folder into a single package.");
                    println!("\nExample:");
                    println!("  s4pi-reforged merge ./mods/to-merge");
                    return Ok(());
                }
                if args.len() < 3 {
                    return Err(anyhow!("Usage: s4pi-reforged merge <folder>\nTry 's4pi-reforged merge --help' for more information."));
                }
                run_merge(Path::new(&args[2]))?;
            }
            "unmerge" => {
                if args.iter().any(|a| a == "--help") {
                    println!("Usage: s4pi-reforged unmerge <file>");
                    println!("\nUn-merges a merged .package file into its original components using its manifest.");
                    println!("\nExample:");
                    println!("  s4pi-reforged unmerge ./merged_mod.package");
                    return Ok(());
                }
                if args.len() < 3 {
                    return Err(anyhow!("Usage: s4pi-reforged unmerge <file>\nTry 's4pi-reforged unmerge --help' for more information."));
                }
                run_unmerge(Path::new(&args[2]))?;
            }
            "extract" => {
                let subcommand = args.get(2).map(|s| s.as_str()).unwrap_or("");
                if subcommand == "--help" || subcommand == "" {
                    println!("Usage: s4pi-reforged extract <subcommand> <path>");
                    println!("\nSubcommands used for extracting data from merged and unmerged packages.");
                    println!("\nAvailable subcommands:");
                    println!("  thumbnails    Extracts thumbnail resources (0x3C1AF1F2) as .jpg files");
                    println!("\nRun 's4pi-reforged extract <subcommand> --help' for specific usage info.");
                    return Ok(());
                }
                match subcommand {
                    "thumbnails" => {
                        if args.iter().any(|a| a == "--help") {
                            println!("Usage: s4pi-reforged extract thumbnails <path>");
                            println!("\nExtracts all thumbnail resources from the specified package into a 'thumbs' directory.");
                            println!("\nExample:");
                            println!("  s4pi-reforged extract thumbnails ./clothes.package");
                            return Ok(());
                        }
                        if args.len() < 4 {
                            return Err(anyhow!("Usage: s4pi-reforged extract thumbnails <path>\nTry 's4pi-reforged extract thumbnails --help' for more information."));
                        }
                        run_extract_thumbnails(Path::new(&args[3]))?;
                    }
                    _ => {
                        println!("Unknown extract subcommand: {}", subcommand);
                        println!("Available subcommands: thumbnails");
                    }
                }
            }
            "investigate" => {
                if args.iter().any(|a| a == "--help") {
                    println!("Usage: s4pi-reforged investigate <file>");
                    println!("\nScans a package for resource types and reports known/unknown status.");
                    return Ok(());
                }
                if args.len() < 3 {
                    return Err(anyhow!("Usage: s4pi-reforged investigate <file>"));
                }
                run_investigate(Path::new(&args[2]))?;
            }
            "diagnostics" => {
                if args.iter().any(|a| a == "--help") {
                    println!("Usage: s4pi-reforged diagnostics <file>");
                    println!("\nDumps DBPF header and index entries for structural analysis.");
                    return Ok(());
                }
                if args.len() < 3 {
                    return Err(anyhow!("Usage: s4pi-reforged diagnostics <file>"));
                }
                run_diagnostics(Path::new(&args[2]))?;
            }
            "--help" | "-h" | "help" => {
                println!("S4PI Package Tool");
                println!("\nUsage: s4pi-reforged <command> [args]");
                println!("\nAvailable commands:");
                println!("  merge       Merge multiple packages into one");
                println!("  unmerge     Split a merged package into original files");
                println!("  extract     Extract specific resource types (e.g., thumbnails)");
                if debug {
                    println!("  investigate Scan for resource types (Debug)");
                    println!("  diagnostics Dump DBPF metadata (Debug)");
                }
                println!("\nRun 's4pi-reforged <command> --help' for more information on a specific command.");
                return Ok(());
            }
            _ => {
                println!("Unknown command: {}", cmd);
                println!("Available commands: merge, unmerge, extract{}", if debug { ", investigate, diagnostics" } else { "" });
                println!("Run 's4pi-reforged --help' for usage information.");
            }
        }
        return Ok(());
    }

    let is_terminal = atty::is(atty::Stream::Stdout);
    let force_gui = std::env::var("S4PI_FORCE_GUI").is_ok();
    let force_tui = std::env::var("S4PI_FORCE_TUI").is_ok();

    // On Windows, if we are NOT forced into TUI and either forced into GUI or NOT in a terminal, use GUI.
    // However, atty::is often returns true on Windows even when launched from Explorer if it's a console app.
    // A better check for "launched from explorer" on Windows is sometimes checking if the console title matches the executable path or other tricks, 
    // but here we will try to be more biased towards GUI for better UX when no args are provided.
    
    #[cfg(windows)]
    let prefer_gui = !is_terminal || !force_tui; // On Windows, prefer GUI unless TUI is forced.
    #[cfg(not(windows))]
    let prefer_gui = !is_terminal || force_gui;

    if (is_terminal && !prefer_gui) || force_tui {
        // TUI Mode
        prepare_console();
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
        loop {
            println!("\nChoose an action:");
            println!("1. Merge .package files");
            println!("2. Un-merge .package file (Using manifest)");
            println!("3. Extract options");
            if is_debug_mode() {
                println!("4. Advanced options");
            }
            println!("q. Exit");

            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;
            let choice = choice.trim().to_lowercase();

            match choice.as_str() {
                "1" => {
                    let folder = FileDialog::new()
                        .set_title("Select Folder containing .package files")
                        .pick_folder();

                    if let Some(f) = folder {
                        if let Err(e) = run_merge(&f) {
                            error!("Fatal error during merge: {:?}", e);
                        }
                    }
                }
                "2" => {
                    let file = FileDialog::new()
                        .set_title("Select .package file to un-merge")
                        .add_filter("Package Files", &["package"])
                        .pick_file();

                    if let Some(f) = file {
                        if let Err(e) = run_unmerge(&f) {
                            error!("Fatal error during un-merge: {:?}", e);
                        }
                    }
                }
                "3" => {
                    println!("Extract options:");
                    println!("1. Thumbnail");
                    println!("0. Back");

                    let mut ext_choice = String::new();
                    io::stdin().read_line(&mut ext_choice)?;
                    let ext_choice = ext_choice.trim();

                    match ext_choice {
                        "1" => {
                            let file = FileDialog::new()
                                .set_title("Select .package file to extract thumbnails")
                                .add_filter("Package Files", &["package"])
                                .pick_file();

                            if let Some(f) = file {
                                if let Err(e) = run_extract_thumbnails(&f) {
                                    error!("Fatal error during extraction: {:?}", e);
                                }
                            }
                        }
                        "0" => continue,
                        _ => println!("Invalid choice."),
                    }
                }
                "4" if is_debug_mode() => {
                    println!("Advanced options:");
                    println!("1. Investigate .package file (Scan for unknown resources)");
                    println!("2. Diagnostic .package file (Dump index and head)");
                    println!("0. Back");

                    let mut adv_choice = String::new();
                    io::stdin().read_line(&mut adv_choice)?;
                    let adv_choice = adv_choice.trim();

                    match adv_choice {
                        "1" => {
                            let file = FileDialog::new()
                                .set_title("Select .package file to investigate")
                                .add_filter("Package Files", &["package"])
                                .pick_file();

                            if let Some(f) = file {
                                if let Err(e) = run_investigate(&f) {
                                    error!("Fatal error during investigation: {:?}", e);
                                }
                            }
                        }
                        "2" => {
                            let file = FileDialog::new()
                                .set_title("Select .package file for diagnostics")
                                .add_filter("Package Files", &["package"])
                                .pick_file();

                            if let Some(f) = file {
                                if let Err(e) = run_diagnostics(&f) {
                                    error!("Fatal error during diagnostics: {:?}", e);
                                }
                            }
                        }
                        "0" => continue,
                        _ => println!("Invalid choice."),
                    }
                }
                "q" => break,
                _ => println!("Invalid choice."),
            }
            if choice != "q" {
                println!("\nPress Enter to return to the main menu...");
                let mut _pause = String::new();
                let _ = io::stdin().read_line(&mut _pause);
            }
        }
    } else {
        // GUI Mode
        let log_arc = Arc::clone(&log_buffer);
        let writer = LogWriter { buffer: log_arc };
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Off) // Default to off
            .filter_module("s4pi_merge", log::LevelFilter::Info)
            .filter_module("s4pi_reforged", log::LevelFilter::Info)
            .target(env_logger::Target::Pipe(Box::new(writer)))
            .init();

        let native_options = eframe::NativeOptions::default();
        let log_arc_gui = Arc::clone(&log_buffer);
        eframe::run_native(
            "S4PI Tool",
            native_options,
            Box::new(|cc| Ok(Box::new(GuiApp::new(cc, log_arc_gui)))),
        ).map_err(|e| anyhow!("GUI Error: {:?}", e))?;
    }

    Ok(())
}

fn run_diagnostics(path: &Path) -> Result<()> {
    info!("Running Diagnostics: {:?}", path);
    let pkg = Package::open(path)?;

    println!("Package: {}", path.display());
    println!("Header: {:?}", pkg.header);
    println!("Index Count: {}", pkg.entries.len());

    let mut compressed_count = 0;
    let mut uncompressed_entries = Vec::new();

    for (i, entry) in pkg.entries.iter().enumerate() {
        if entry.is_compressed() {
            compressed_count += 1;
        } else {
            uncompressed_entries.push((i, entry.tgi, entry.memsize));
        }

        if i < 20 || i >= pkg.entries.len() - 5 || entry.tgi.res_type == 0x7FB6AD8A || entry.tgi.res_type == 0x73E93EEB {
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
        } else if i == 20 {
            println!("\n... skipping intermediate entries ...");
        }
    }

    println!("\n--- Compression Summary ---");
    println!("Total Entries: {}", pkg.entries.len());
    println!("Compressed: {} ({:.2}%)", compressed_count, (compressed_count as f32 / pkg.entries.len() as f32) * 100.0);
    println!("Uncompressed: {} ({:.2}%)", uncompressed_entries.len(), (uncompressed_entries.len() as f32 / pkg.entries.len() as f32) * 100.0);

    if !uncompressed_entries.is_empty() {
        println!("\nUncompressed Samples (up to 10):");
        for (i, tgi, size) in uncompressed_entries.iter().take(10) {
            println!("  Entry {}: TGI: {:08X}:{:08X}:{:016X}, Size: {}", i, tgi.res_type, tgi.res_group, tgi.instance, size);
        }
    }

    Ok(())
}

fn run_investigate(path: &Path) -> Result<()> {
    info!("Investigating: {:?}", path);
    let mut pkg = Package::open(path)?;
    
    let mut type_counts: HashMap<u32, usize> = HashMap::new();
    let mut unknown_types: HashSet<u32> = HashSet::new();
    let mut parse_errors: HashMap<u32, Vec<String>> = HashMap::new();

    let entries = pkg.entries.clone();
    info!("Found {} resources.", entries.len());

    for entry in &entries {
        *type_counts.entry(entry.tgi.res_type).or_insert(0) += 1;
        
        match pkg.read_resource(entry) {
            Ok(TypedResource::Generic(_)) => {
                unknown_types.insert(entry.tgi.res_type);
            }
            Ok(TypedResource::Manifest(manifest)) => {
                println!("\n--- Manifest Found (Type: 0x{:08X}) ---", entry.tgi.res_type);
                println!("  Version: {}", manifest.version);
                println!("  Entries: {}", manifest.entries.len());
                for (i, entry) in manifest.entries.iter().enumerate() {
                    println!("    [{:>2}] Name: \"{}\"", i + 1, entry.name);
                    println!("         Resources: {}", entry.resources.len());
                    // Optional: print first few TGIs if needed
                }
                println!("----------------------------------------\n");
            }
            Ok(_) => {}
            Err(e) => {
                unknown_types.insert(entry.tgi.res_type);
                parse_errors.entry(entry.tgi.res_type).or_default().push(format!("{:?}", e));
            }
        }
    }

    println!("\nResource Type Summary:");
    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by_key(|a| a.0);

    for (res_type, count) in sorted_types {
        let status = if let Some(errors) = parse_errors.get(res_type) {
            format!("FAILED ({} errors)", errors.len())
        } else if unknown_types.contains(res_type) {
            "UNKNOWN".to_string()
        } else {
            "KNOWN".to_string()
        };
        println!("  Type: 0x{:08X} | Count: {:>5} | Status: {}", res_type, count, status);

        if unknown_types.contains(res_type) || parse_errors.contains_key(res_type) || *res_type == 0x7FB6AD8A {
            // Find a sample of this type to show magic bytes
            if let Some(entry) = entries.iter().find(|e| e.tgi.res_type == *res_type) {
                println!("    Size: {} bytes", entry.memsize);
                if let Ok(data) = pkg.read_raw_resource(entry) {
                    let len = data.len().min(64);
                    let hex: Vec<String> = data[..len].iter().map(|b| format!("{:02X}", b)).collect();
                    println!("    Sample Hex: {}", hex.join(" "));
                    let ascii: String = data[..len].iter().map(|b| {
                        if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' }
                    }).collect();
                    println!("    Sample ASCII: \"{}\"", ascii);
                }
            }
        }
    }

    if !parse_errors.is_empty() {
        println!("\nParse Error Samples (one per type):");
        for (res_type, errors) in &parse_errors {
            println!("  0x{:08X}: {}", res_type, errors[0].lines().next().unwrap_or("Unknown error"));
        }
    }

    if !unknown_types.is_empty() {
        println!("\nCandidates for Manifest (Unknown/Failed Types):");
        for res_type in unknown_types {
            println!("  0x{:08X}", res_type);
        }
    } else {
        println!("\nAll resource types are known and parsed successfully.");
    }

    Ok(())
}

fn run_extract_thumbnails(path: &Path) -> Result<()> {
    info!("Extracting thumbnails from: {:?}", path);
    let mut pkg = Package::open(path)?;

    let entries: Vec<_> = pkg.entries.iter()
        .filter(|e| e.tgi.res_type == 0x3C1AF1F2)
        .cloned()
        .collect();

    if entries.is_empty() {
        info!("No thumbnail resources (0x3C1AF1F2) found in package.");
        return Ok(());
    }

    info!("Found {} thumbnails.", entries.len());

    let output_dir = path.parent().unwrap_or(Path::new(".")).join("thumbs");
    std::fs::create_dir_all(&output_dir).context("Failed to create thumbs directory")?;

    // Try to find manifest to get original package names
    let manifest_entry = pkg.entries.iter().find(|e| e.tgi.res_type == 0x7FB6AD8A || e.tgi.res_type == 0x73E93EEB).cloned();
    let mut tgi_to_name = HashMap::new();
    if let Some(me) = manifest_entry {
        if let Ok(TypedResource::Manifest(m)) = pkg.read_resource(&me) {
            for entry in m.entries {
                for tgi in entry.resources {
                    tgi_to_name.insert(tgi, entry.name.clone());
                }
            }
        }
    }

    let package_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();

    entries.par_iter().try_for_each(|entry| -> Result<()> {
        let mut pkg_thread = Package::open(path)?;
        let data = pkg_thread.read_raw_resource(entry)?;
        
        let name_base = tgi_to_name.get(&entry.tgi).cloned().unwrap_or_else(|| package_name.clone());
        let filename = format!("{}_{:016X}.jpg", name_base, entry.tgi.instance);
        let out_path = output_dir.join(filename);
        
        std::fs::write(out_path, data)?;
        Ok(())
    })?;

    info!("Thumbnail extraction complete! Files are in: {:?}", output_dir);
    Ok(())
}

fn run_unmerge(path: &Path) -> Result<()> {
    info!("Un-merging: {:?}", path);
    let mut pkg = Package::open(path)?;
    
    let manifest_entry = pkg.entries.iter().find(|e| e.tgi.res_type == 0x7FB6AD8A || e.tgi.res_type == 0x73E93EEB)
        .cloned()
        .context("No manifest found in package. This package cannot be un-merged automatically.")?;
    
    let manifest = match pkg.read_resource(&manifest_entry)? {
        TypedResource::Manifest(m) => m,
        _ => return Err(anyhow!("Failed to parse manifest resource")),
    };

    info!("Found manifest with {} original packages.", manifest.entries.len());

    let output_dir = path.parent().unwrap_or(Path::new(".")).join("unmerged");
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    manifest.entries.par_iter().enumerate().try_for_each(|(i, entry)| -> Result<()> {
        let filename = if entry.name.to_lowercase().ends_with(".package") {
            entry.name.clone()
        } else {
            format!("{}.package", entry.name)
        };
        
        info!("[{}/{}] Extracting: {}", i + 1, manifest.entries.len(), filename);
        
        let mut sub_package_data: HashMap<TGI, (Vec<u8>, u32, u16, u16)> = HashMap::new();
        
        // We need to re-open the package in each thread because Package is not Sync (it has a File)
        let mut pkg_thread = Package::open(path)?;
        
        for tgi in &entry.resources {
            // Find the resource in the merged package
            let pkg_entry = pkg_thread.entries.iter().find(|e| e.tgi == *tgi).cloned();
            
            if let Some(entry) = pkg_entry {
                // Read RAW resource to preserve compression/metadata if possible
                let data = pkg_thread.read_raw_resource(&entry)?;
                sub_package_data.insert(*tgi, (data, entry.memsize, entry.compression, entry.committed));
            } else {
                warn!("Resource {:?} listed in manifest but not found in package!", tgi);
            }
        }

        let output_path = output_dir.join(filename);
        Package::write_merged(&output_path, &sub_package_data, false)?;
        Ok(())
    })?;

    info!("Un-merge complete! Files are in: {:?}", output_dir);
    
    Ok(())
}

fn run_merge(folder: &std::path::Path) -> Result<()> {
    let mut files_to_process = Vec::new();

    info!("Searching for .package files in: {:?}", folder);

    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "package") {
            // Avoid processing a file named "merged.package" if it already exists in a "merged" subfolder
            if !path.to_string_lossy().contains("merged/merged.package") {
                files_to_process.push(path.to_path_buf());
            }
        }
    }

    let total_files = files_to_process.len();
    if total_files == 0 {
        warn!("No .package files found to merge.");
        return Ok(());
    }

    info!("Found {} files to process.", total_files);

    let results: Vec<Result<(String, Vec<TGI>, Vec<(TGI, (Vec<u8>, u32, u16, u16))>)>> = files_to_process
        .par_iter()
        .map(|path| {
            let filename = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let mut pkg_resources = Vec::new();
            let mut pkg_data = Vec::new();
            
            let mut pkg = Package::open(path)?;
            let entries: Vec<_> = pkg.entries.iter().cloned().collect();
            
            for entry in entries {
                if entry.tgi.res_type == 0x7FB6AD8A || entry.tgi.res_type == 0x73E93EEB {
                    continue;
                }
                let data = pkg.read_raw_resource(&entry)?;
                pkg_data.push((entry.tgi, (data, entry.memsize, entry.compression, entry.committed)));
                pkg_resources.push(entry.tgi);
            }
            
            Ok((filename, pkg_resources, pkg_data))
        })
        .collect();

    let mut merged_data: HashMap<TGI, (Vec<u8>, u32, u16, u16)> = HashMap::new();
    let mut manifest_entries = Vec::new();
    let mut files_processed = 0;
    let mut files_skipped = 0;

    for res in results {
        match res {
            Ok((filename, pkg_resources, pkg_data)) => {
                files_processed += 1;
                manifest_entries.push(s4pi_reforged::package::resource::ManifestEntry {
                    name: filename,
                    resources: pkg_resources,
                });
                for (tgi, data) in pkg_data {
                    merged_data.insert(tgi, data);
                }
            }
            Err(e) => {
                error!("Error processing a file: {}. Skipping.", e);
                files_skipped += 1;
            }
        }
    }

    if merged_data.is_empty() {
        warn!("No resources found to merge.");
        return Ok(());
    }

    // Generate manifest resource
    let manifest = s4pi_reforged::package::resource::ManifestResource {
        version: 1,
        padding: 0,
        entries: manifest_entries,
    };

    use s4pi_reforged::package::resource::Resource;
    let manifest_data = manifest.to_bytes().context("Failed to serialize manifest")?;
    let manifest_tgi = TGI {
        res_type: 0x7FB6AD8A,
        res_group: 0,
        instance: 0, // Should we use a specific instance for the manifest? S4S often uses 0 or some hash.
    };
    
    // Add manifest to merged data
    // Force compression for manifest by setting compression flag to 0x5A42 and ensuring it is compressed in write_merged
    merged_data.insert(manifest_tgi, (manifest_data.clone(), manifest_data.len() as u32, 0x5A42, 1));

    let output_dir = folder.join("merged");
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
    
    let output_file = output_dir.join("merged.package");
    info!("Writing merged package to: {:?}", output_file);

    Package::write_merged(&output_file, &merged_data, true).context("Failed to write merged package")?;

    info!("Merge complete!");
    info!("Files processed: {}", files_processed);
    info!("Files skipped: {}", files_skipped);
    info!("Total resources merged: {}", merged_data.len());
    
    // Explicitly clear/drop to free memory as requested
    merged_data.clear();
    merged_data.shrink_to_fit();
    
    Ok(())
}

