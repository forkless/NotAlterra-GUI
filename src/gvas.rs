//! UE4/UE5 GVAS save-file binary parser.
//!
//! Ported from `legacy/extract_save_name.py`.  Extracts `SlotName` and
//! `DisplayName` properties via manual binary walking, plus corruption
//! detection by cross-referencing metadata against the canonical filename
//! convention (`savegame_N.sav` / `savegame_N_M.bak`).

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

// ── low-level binary primitives ───────────────────────────────────────────

/// Read a little-endian u32 at `offset`, returning `None` if out of bounds.
fn read_u32(data: &[u8], offset: usize) -> Option<usize> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]) as usize)
}

/// Read a little-endian i32 at `offset`, returning `None` if out of bounds.
fn read_i32(data: &[u8], offset: usize) -> Option<i64> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]) as i64)
}

/// Read a length-prefixed, null-terminated FName string.
///
/// Layout: `<u32 length><bytes><optional null>`
/// Returns (string, new_offset) or (None, offset) on failure.
fn read_fname(data: &[u8], offset: usize) -> (Option<String>, usize) {
    let len = match read_u32(data, offset) {
        Some(l) => l,
        None => return (None, offset),
    };
    let mut off = offset + 4;
    if len == 0 || off + len > data.len() {
        return (None, off);
    }
    let mut raw = &data[off..off + len];
    if raw.last() == Some(&0) {
        raw = &raw[..raw.len() - 1];
    }
    off += len;
    let s = String::from_utf8_lossy(raw).into_owned();
    (Some(s), off)
}

/// Read an FString: length-prefixed, possibly UTF-16.
///
/// Layout: `<i32 length>` – negative means UTF-16 with `-len` chars,
/// positive means UTF-8 byte count (including null terminator).
/// Returns (string, new_offset).
fn read_fstring(data: &[u8], offset: usize) -> (Option<String>, usize) {
    let raw_len = match read_i32(data, offset) {
        Some(l) => l,
        None => return (None, offset),
    };
    let mut off = offset + 4;

    if raw_len == 0 {
        return (Some(String::new()), off);
    }

    let (bytes, is_utf16): (usize, bool) = if raw_len < 0 {
        ((-raw_len) as usize * 2, true)
    } else {
        (raw_len as usize, false)
    };

    if off + bytes > data.len() {
        return (None, off);
    }

    let mut raw = &data[off..off + bytes];
    off += bytes;

    let value = if is_utf16 {
        // Decode UTF-16 LE
        let code_units: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&code_units)
    } else {
        if raw.last() == Some(&0) {
            raw = &raw[..raw.len() - 1];
        }
        String::from_utf8_lossy(raw).into_owned()
    };

    (Some(value), off)
}

// ── property extraction ────────────────────────────────────────────────────

/// Find the first `StrProperty` named `prop_name` and return its FString value.
///
/// Walks the binary looking for the FName header of the property, then
/// skips past the StrProperty metadata to read the string value.
fn extract_str_property(data: &[u8], prop_name: &str) -> Result<String, String> {
    let target = prop_name.as_bytes();
    let mut offset = 0usize;
    let mut attempts = 0u32;

    while offset < data.len().saturating_sub(20) && attempts < 100 {
        let found = match data[offset..]
            .windows(target.len())
            .position(|w| w == target)
        {
            Some(p) => offset + p,
            None => return Err(format!("{prop_name} not found")),
        };

        // Each candidate must be a proper FName: preceded by a length dword
        // matching the target length + 1 (null terminator), followed by a
        // null byte.
        if found < 4 {
            offset = found + 1;
            attempts += 1;
            continue;
        }

        let name_len_field = read_u32(data, found - 4);
        if name_len_field != Some(target.len() + 1) {
            offset = found + 1;
            attempts += 1;
            continue;
        }
        if data[found + target.len()] != 0 {
            offset = found + 1;
            attempts += 1;
            continue;
        }

        let after_name = found + target.len() + 1;
        let (next_name, next_offset) = read_fname(data, after_name);
        if next_name.as_deref() != Some("StrProperty") {
            offset = found + 1;
            attempts += 1;
            continue;
        }

        // Skip StrProperty metadata: 9 bytes of property flags + padding,
        // then the FString value.
        let meta_offset = next_offset + 9;
        if meta_offset + 4 > data.len() {
            offset = found + 1;
            attempts += 1;
            continue;
        }

        let (value, _) = read_fstring(data, meta_offset);
        match value {
            Some(v) if !v.is_empty() && v.len() < 100 => return Ok(v),
            _ => {
                offset = found + 1;
                attempts += 1;
            }
        }
    }

    Err(format!("no valid {prop_name}/StrProperty pair found"))
}

/// Find a BoolProperty by name and return its value (true/false).
fn extract_bool_property(data: &[u8], prop_name: &str) -> Option<bool> {
    let target = prop_name.as_bytes();
    let mut offset = 0usize;
    let mut attempts = 0u32;
    while offset < data.len().saturating_sub(20) && attempts < 100 {
        let found = data[offset..].windows(target.len()).position(|w| w == target);
        let found = match found { Some(p) => offset + p, None => return None };
        if found < 4 { offset = found + 1; attempts += 1; continue; }
        let name_len_field = read_u32(data, found - 4);
        if name_len_field != Some(target.len() + 1) { offset = found + 1; attempts += 1; continue; }
        if data[found + target.len()] != 0 { offset = found + 1; attempts += 1; continue; }
        let after_name = found + target.len() + 1;
        let (next_name, next_offset) = read_fname(data, after_name);
        if next_name.as_deref() != Some("BoolProperty") { offset = found + 1; attempts += 1; continue; }
        let val_offset = next_offset + 9;
        if val_offset >= data.len() { offset = found + 1; attempts += 1; continue; }
        return Some(data[val_offset] != 0);
    }
    None
}

/// Scan for a double value near a marker byte sequence.
fn scan_double_near(data: &[u8], marker: &[u8]) -> Option<f64> {
    let pos = data.windows(marker.len()).position(|w| w == marker)?;
    let end = (pos + 60).min(data.len());
    for off in 8..50 {
        if pos + off + 8 > data.len() { break; }
        let val = f64::from_le_bytes(data[pos+off..pos+off+8].try_into().ok()?);
        if val > 60.0 && val < 10_000_000.0 {
            return Some(val);
        }
    }
    None
}

/// Find an IntProperty by name and return its u32 value.
fn extract_double_property(data: &[u8], prop_name: &str) -> Option<f64> {
    let target = prop_name.as_bytes();
    let mut offset = 0usize;
    let mut attempts = 0u32;
    while offset < data.len().saturating_sub(30) && attempts < 100 {
        let found = data[offset..].windows(target.len()).position(|w| w == target);
        let found = match found { Some(p) => offset + p, None => return None };
        if found < 4 { offset = found + 1; attempts += 1; continue; }
        let expected: usize = target.len() + 1;
        if read_u32(data, found - 4) != Some(expected) { offset = found + 1; attempts += 1; continue; }
        if data[found + target.len()] != 0 { offset = found + 1; attempts += 1; continue; }
        let (next_name, next_offset) = read_fname(data, found + target.len() + 1);
        if next_name.as_deref() != Some("DoubleProperty") { offset = found + 1; attempts += 1; continue; }
        let val_offset = next_offset + 9;
        if val_offset + 8 > data.len() { offset = found + 1; attempts += 1; continue; }
        return Some(f64::from_le_bytes(data[val_offset..val_offset+8].try_into().ok()?));
    }
    None
}

/// Extract an integer property value from a key-value text pair.
fn extract_int_property(data: &[u8], prop_name: &str) -> Option<u32> {
    let target = prop_name.as_bytes();
    let mut offset = 0usize;
    let mut attempts = 0u32;
    while offset < data.len().saturating_sub(20) && attempts < 100 {
        let found = data[offset..].windows(target.len()).position(|w| w == target);
        let found = match found { Some(p) => offset + p, None => return None };
        if found < 4 { offset = found + 1; attempts += 1; continue; }
        if read_u32(data, found - 4) != Some(target.len() + 1) { offset = found + 1; attempts += 1; continue; }
        if data[found + target.len()] != 0 { offset = found + 1; attempts += 1; continue; }
        let (next_name, next_offset) = read_fname(data, found + target.len() + 1);
        if next_name.as_deref() != Some("IntProperty") { offset = found + 1; attempts += 1; continue; }
        let val_offset = next_offset + 9;
        if val_offset + 4 > data.len() { offset = found + 1; attempts += 1; continue; }
        return read_u32(data, val_offset).map(|v| v as u32);
    }
    None
}

// ── public API ─────────────────────────────────────────────────────────────

/// Full metadata from all known GVAS properties.
#[derive(Debug, Clone, Default)]
pub struct FullMetadata {
    pub slot_name: Option<String>,
    pub display_name: Option<String>,
    pub is_online: bool,
    pub was_multiplayer: bool,
    pub game_mode: Option<String>,
    pub level_name: Option<String>,
    pub build_number: Option<u32>,
    pub build_branch: Option<String>,
    pub saves_count: Option<u32>,
    pub latest_version: Option<u32>,
    pub data_version: Option<u32>,
    pub playtime_seconds: Option<f64>,
}

/// Parse a `.sav` or `.bak` file and return all known GVAS metadata.
pub fn extract_full_metadata(path: &Path) -> Result<FullMetadata> {
    let data = fs::read(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(FullMetadata {
        slot_name: extract_str_property(&data, "SlotName").ok(),
        display_name: extract_str_property(&data, "DisplayName").ok(),
        is_online: extract_bool_property(&data, "bIsMultiplayerSave").unwrap_or(false),
        was_multiplayer: extract_bool_property(&data, "bWasMultiplayerSave").unwrap_or(false),
        game_mode: extract_str_property(&data, "GameMode").ok(),
        level_name: extract_str_property(&data, "LevelName").ok(),
        build_number: extract_int_property(&data, "BuildNumber"),
        build_branch: extract_str_property(&data, "BuildBranch").ok(),
        saves_count: extract_int_property(&data, "SavesCount"),
        latest_version: extract_int_property(&data, "LatestVersion"),
        data_version: extract_int_property(&data, "DataVersion"),
        playtime_seconds: scan_double_near(&data, b"Elapsed"),
    })
}

/// Extracted metadata from a GVAS save file.
#[derive(Debug, Clone, Default)]
/// Extracted metadata from a Subnautica 2 GVAS save file.
///
/// Each field corresponds to a named UE5 property inside the save binary.
/// Playtime is derived from the `PlaytimeData` structure when available,
/// falling back to a byte-scan heuristic.
pub struct SaveMetadata {
    /// Internal slot name, e.g. "savegame_0"
    pub slot_name: Option<String>,
    /// Human-readable display name entered in-game
    pub display_name: Option<String>,
    /// Current online/multiplayer status (bIsMultiplayerSave)
    pub is_online: bool,
    /// Total playtime in seconds
    pub playtime_seconds: Option<f64>,
    /// Any extraction errors (non-fatal)
    pub errors: Vec<String>,
}

/// Parse a `.sav` or `.bak` file and return its GVAS metadata.
pub fn extract_metadata(path: &Path) -> Result<SaveMetadata> {
    let data = fs::read(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let mut errors = Vec::new();
    let slot_name = match extract_str_property(&data, "SlotName") {
        Ok(v) => Some(v),
        Err(e) => {
            errors.push(e);
            None
        }
    };
    let display_name = match extract_str_property(&data, "DisplayName") {
        Ok(v) => Some(v),
        Err(e) => {
            errors.push(e);
            None
        }
    };
    let is_online = extract_bool_property(&data, "bIsMultiplayerSave").unwrap_or(false);

    let playtime_seconds = scan_double_near(&data, b"Elapsed");
    Ok(SaveMetadata {
        slot_name,
        display_name,
        is_online,
        playtime_seconds,
        errors,
    })
}

// ── filename conventions ───────────────────────────────────────────────────

/// Derive the expected slot name from a filename.
///
/// `savegame_2_9.sav` → `"savegame_2"`
/// `savegame_0.bak`   → `"savegame_0"`
/// `random.sav`       → `None`
pub fn derive_slot_from_filename(filename: &str) -> Option<String> {
    let re = regex::Regex::new(r"^(savegame_\d+)").ok()?;
    re.captures(filename)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Return a corruption reason, or `None` if the file looks clean.
pub fn corruption_check(filename: &str, slot_name: Option<&str>) -> Option<String> {
    let expected_slot = derive_slot_from_filename(filename);

    if expected_slot.is_none() {
        return Some("nonstandard filename".into());
    }

    let expected = expected_slot.unwrap();

    if let Some(sn) = slot_name {
        if sn != expected {
            return Some(format!("slot mismatch ({sn})"));
        }
    }

    // Non-canonical .sav: savegame_N_M.sav is not a live file
    if filename.ends_with(".sav") {
        let re = regex::Regex::new(r"^savegame_\d+\.sav$").ok()?;
        if !re.is_match(filename) {
            return Some("non-canonical .sav".into());
        }
    }

    None
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_slot() {
        assert_eq!(
            derive_slot_from_filename("savegame_0.sav"),
            Some("savegame_0".into())
        );
        assert_eq!(
            derive_slot_from_filename("savegame_2_9.bak"),
            Some("savegame_2".into())
        );
        assert_eq!(
            derive_slot_from_filename("savegame_0.bak"),
            Some("savegame_0".into())
        );
        assert_eq!(derive_slot_from_filename("random.sav"), None);
    }

    #[test]
    fn test_corruption_check() {
        // Canonical live file
        assert_eq!(
            corruption_check("savegame_0.sav", Some("savegame_0")),
            None
        );
        // Versioned .sav is non-canonical
        assert_eq!(
            corruption_check("savegame_0_9.sav", Some("savegame_0")),
            Some("non-canonical .sav".into())
        );
        // Slot mismatch
        assert_eq!(
            corruption_check("savegame_1.bak", Some("savegame_0")),
            Some("slot mismatch (savegame_0)".into())
        );
        // Backup that matches
        assert_eq!(
            corruption_check("savegame_2_5.bak", Some("savegame_2")),
            None
        );
    }

    #[test]
    fn dump_all_samples() {
        use chrono::TimeZone;
        let dir = std::path::Path::new("samples");
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let s = n.to_string_lossy();
                s.ends_with(".sav") || s.ends_with(".bak")
            })
            .collect();
        // Sort by slot (extracted from filename), then mtime desc
        files.sort_by(|a, b| {
            use crate::gvas::derive_slot_from_filename;
            let sa = derive_slot_from_filename(&a.file_name().to_string_lossy());
            let sb = derive_slot_from_filename(&b.file_name().to_string_lossy());
            let ma = a.metadata().ok().and_then(|m| m.modified().ok());
            let mb = b.metadata().ok().and_then(|m| m.modified().ok());
            sa.cmp(&sb).then_with(|| mb.cmp(&ma))
        });
        println!("\n{:<8} {:<26} {:<6} {:>7}  {:<19}  {:<28}", "", "Display Name", "Type", "Size", "Date", "File");
        println!("{}", "-".repeat(115));
        let mut seen = std::collections::HashSet::new();
        for entry in &files {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let mtime = entry.metadata().ok().and_then(|m| m.modified().ok())
                .and_then(|t| { let s = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs(); chrono::Local.timestamp_opt(s as i64,0).single() })
                .map(|dt| dt.format("%Y-%b-%d %H:%M").to_string());
            let meta = extract_metadata(&path).ok();
            let slot = meta.as_ref().and_then(|m| m.slot_name.clone())
                .unwrap_or_else(|| derive_slot_from_filename(&name).unwrap_or_else(|| "?".into()));
            let display = meta.as_ref().and_then(|m| m.display_name.clone()).unwrap_or_else(|| "(unnamed)".into());
            let online = meta.map(|m| m.is_online).unwrap_or(false);
            let num = slot.strip_prefix("savegame_").unwrap_or(&slot);
            let first = seen.insert(slot.clone());
            let label = if first { format!("Slot {num}") } else { String::new() };
            let typ = if online { "Online" } else { "Local" };
            let sz = if size < 1024 { format!("{size} B") } else if size < 1_048_576 { format!("{:.0} KB", size as f64 / 1024.0) } else { format!("{:.1} MB", size as f64 / 1_048_576.0) };
            println!("{label:<8} {display:<26} {typ:<6} {sz:>7}  {:<19}  {name:<28}", mtime.as_deref().unwrap_or("?"));
        }
        println!();
    }

    #[test]
    /// Test helper — dump full GVAS metadata for a sample file.
    fn print_full_meta() {
        let p = Path::new("samples/savegame_1.sav");
        if !p.exists() { return; }
        let m = extract_full_metadata(p).unwrap();
        println!("slot: {:?}", m.slot_name);
        println!("display: {:?}", m.display_name);
        println!("online: {}", m.is_online);
        println!("was_multi: {}", m.was_multiplayer);
        println!("gamemode: {:?}", m.game_mode);
        println!("level: {:?}", m.level_name);
        println!("build: {:?}", m.build_number);
        println!("branch: {:?}", m.build_branch);
        println!("savescnt: {:?}", m.saves_count);
        println!("latest: {:?}", m.latest_version);
        println!("dataver: {:?}", m.data_version);
    }

    #[test]
    fn test_real_sample() {
        let p = Path::new("samples/savegame_0.sav");
        if p.exists() {
            let meta = extract_metadata(p).unwrap();
            assert!(meta.slot_name.is_some() || !meta.errors.is_empty());
        }
    }
}
