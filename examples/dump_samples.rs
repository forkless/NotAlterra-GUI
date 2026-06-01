//! Dump GVAS metadata for all .sav/.bak files in `samples/`.
//! Run: cargo run --example dump_samples

use chrono::TimeZone;
use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new("samples");
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let fname = e.file_name();
            let s = fname.to_string_lossy();
            s.ends_with(".sav") || s.ends_with(".bak")
        })
        .collect();

    entries.sort_by(|a, b| {
        let ma = a.metadata().ok().and_then(|m| m.modified().ok());
        let mb = b.metadata().ok().and_then(|m| m.modified().ok());
        mb.cmp(&ma)
            .then_with(|| a.file_name().cmp(&b.file_name()))
    });

    println!(
        "{:<8} {:<26} {:<6} {:>7}  {:<19}  {:<28}",
        "", "Display Name", "Type", "Size", "Date", "File"
    );
    println!("{}", "-".repeat(115));

    let mut seen = std::collections::HashSet::new();
    for entry in &entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                let secs = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                chrono::Local
                    .timestamp_opt(secs as i64, 0)
                    .single()
            })
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string());

        let meta = notalterra::gvas::extract_metadata(&path).ok();
        let slot = meta.as_ref().and_then(|m| m.slot_name.clone()).unwrap_or_else(|| {
            notalterra::gvas::derive_slot_from_filename(&name).unwrap_or_else(|| "?".into())
        });
        let display = meta
            .as_ref()
            .and_then(|m| m.display_name.clone())
            .unwrap_or_else(|| "(unnamed)".into());
        let is_online = meta.map(|m| m.is_online).unwrap_or(false);

        let label_num = slot.strip_prefix("savegame_").unwrap_or(&slot);
        let first = seen.insert(slot.clone());
        let label = if first { format!("Slot {label_num}") } else { String::new() };

        let typ = if is_online { "Multiplayer" } else { "Single Player" };
        let sz = if size < 1024 {
            format!("{size} B")
        } else if size < 1024 * 1024 {
            format!("{:.0} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        };

        println!(
            "{label:<8} {display:<26} {typ:<6} {sz:>7}  {:<19}  {name:<28}",
            mtime.as_deref().unwrap_or("?"),
        );
    }
}
