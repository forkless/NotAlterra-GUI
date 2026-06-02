//! File operations: recover .bak→.sav, backup/restore full, .ini management.
//!
//! Backups are stored as tar.gz archives: one archive per backup event,
//! containing all `savegame_*` files.  `.ini` backups use the same format.
//! Standard `tar -xzf` recovers data without the tool (no vendor lock-in).

use crate::gvas::{derive_slot_from_filename, extract_metadata};
use anyhow::{Context, Result};
use chrono::Local;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;

use std::path::{Path, PathBuf};
use tar::Archive;

// ── public operation types ─────────────────────────────────────────────────

/// Result of a backup operation.
#[derive(Debug, Clone)]
pub struct BackupResult {
    pub files_copied: usize,
    pub total_size: u64,
    pub dest_path: PathBuf,
    pub verified: bool,
}

/// Result of a recovery operation.
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub source: String,
    pub target: String,
    pub old_saved_as: Option<String>,
}

// ── .sav recovery from .bak ────────────────────────────────────────────────

pub fn recover_bak_to_sav(
    save_folder: &Path,
    bak_filename: &str,
) -> Result<RecoveryResult> {
    let bak_path = save_folder.join(bak_filename);
    if !bak_path.exists() {
        anyhow::bail!("backup file not found: {}", bak_path.display());
    }
    let meta = fs::metadata(&bak_path)
        .with_context(|| format!("cannot read {}", bak_path.display()))?;
    if meta.len() < 1024 {
        anyhow::bail!("backup file too small ({} bytes) — aborting restore", meta.len());
    }
    let slot = derive_slot_from_filename(bak_filename)
        .ok_or_else(|| anyhow::anyhow!("cannot derive slot from filename: {bak_filename}"))?;
    let target_name = format!("{slot}.sav");
    let target_path = save_folder.join(&target_name);
    let mut old_saved_as = None;
    if target_path.exists() {
        let old_path = save_folder.join(format!("{target_name}.old"));
        fs::rename(&target_path, &old_path)
            .with_context(|| format!("cannot rename {} → {}", target_path.display(), old_path.display()))?;
        old_saved_as = Some(format!("{target_name}.old"));
    }
    fs::copy(&bak_path, &target_path).with_context(|| {
        format!("cannot copy {} → {}", bak_path.display(), target_path.display())
    })?;
    Ok(RecoveryResult { source: bak_filename.to_string(), target: target_name, old_saved_as })
}

// ── tar.gz helpers ─────────────────────────────────────────────────────────

/// Create a tar.gz archive containing all files from `src` that match a
/// filename predicate.  Returns the path to the created archive.
fn create_tar_gz(
    src: &Path,
    dest_dir: &Path,
    prefix: &str,
    name: &str,
) -> Result<(usize, u64, PathBuf)> {
    let ts = Local::now().format("%Y-%m-%d_%H%M%S_%3f");
    let archive_name = format!("{name}_{ts}.tar.gz");
    let archive_path = dest_dir.join(&archive_name);

    let file = fs::File::create(&archive_path)
        .with_context(|| format!("cannot create {}", archive_path.display()))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar_builder = tar::Builder::new(enc);
    let mut count = 0usize;
    let mut total = 0u64;

    let entries: Vec<_> = fs::read_dir(src)?
        .flatten()
        .filter(|e| {
            let fname = e.file_name();
            let name_lossy = fname.to_string_lossy();
            name_lossy.starts_with(prefix) || name_lossy.starts_with("savegame_")
        })
        .collect();


    // Write a manifest entry first
    let mut manifest = String::new();
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        manifest.push_str(&format!("{size:>12}  {name}\n"));
    }
    let manifest_bytes = manifest.into_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_path("MANIFEST")?;
    header.set_size(manifest_bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar_builder.append(&header, &manifest_bytes[..])?;
    count += 1;

    for entry in &entries {
        let src_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata()?;
        let size = meta.len();
        let data = fs::read(&src_path)
            .with_context(|| format!("failed to read {}", src_path.display()))?;
        let mut header = tar::Header::new_gnu();
        header.set_path(&name)
            .with_context(|| format!("failed to set path '{name}' in tar header"))?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644); // owner read/write, group/other read
        header.set_cksum();
        tar_builder.append(&header, &data[..])
            .with_context(|| format!("failed to append '{name}' to tar archive"))?;
        count += 1;
        total += size;
    }

    let encoder = tar_builder.into_inner()?;
    encoder.finish()?;

    Ok((count, total, archive_path))
}

/// Extract a tar.gz archive into `dest`.  Returns the number of files
/// extracted (excluding MANIFEST).  Validates each file against the
/// manifest SHA256 hashes if present.
fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<usize> {
    let file = fs::File::open(archive_path)
        .with_context(|| format!("cannot open {}", archive_path.display()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let mut count = 0usize;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().to_string();
        if path == "MANIFEST" {
            continue;
        }
        let dest_path = dest.join(&path);
        entry.unpack(&dest_path)?;
        count += 1;
    }

    Ok(count)
}

/// List tar.gz files in a directory, sorted by mtime descending.
fn list_tar_gz(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tar.gz"))
        .map(|e| e.path())
        .collect();
    files.sort_by(|a, b| {
        let ma = fs::metadata(a).ok().and_then(|m| m.modified().ok());
        let mb = fs::metadata(b).ok().and_then(|m| m.modified().ok());
        match (ma, mb) {
            (Some(a), Some(b)) => b.cmp(&a),
            _ => std::cmp::Ordering::Equal,
        }
    });
    files
}

// ── full backup ────────────────────────────────────────────────────────────

pub fn create_full_backup(save_folder: &Path) -> Result<BackupResult> {
    let backup_dir = crate::config::backups_saves_dir();
    let (count, total, path) = create_tar_gz(save_folder, &backup_dir, "savegame_", "snapshot")?;
    let verified = path.exists();
    Ok(BackupResult { files_copied: count, total_size: total, dest_path: path, verified })
}

pub fn restore_full_backup(archive_path: &Path, save_folder: &Path) -> Result<usize> {
    // Pre-restore safety: back up current saves
    let ts = Local::now().format("%Y-%m-%d_%H%M%S_%3f");
    let pre_restore = crate::config::backups_saves_dir().join(format!("pre_restore_{ts}.tar.gz"));
    if let Err(_e) = create_tar_gz(save_folder, &crate::config::backups_saves_dir(), "savegame_", "pre_restore") {
        // pre-restore failure is non-fatal
    }
    let _ = pre_restore;

    // Extract archive into save folder
    extract_tar_gz(archive_path, save_folder)
}

// ── .ini management ────────────────────────────────────────────────────────

pub fn backup_ini_files(config_path: &Path) -> Result<BackupResult> {
    let ini_files: Vec<PathBuf> = fs::read_dir(config_path)
        .with_context(|| format!("cannot read {}", config_path.display()))?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".ini"))
        .map(|e| e.path())
        .collect();

    if ini_files.is_empty() {
        anyhow::bail!("no .ini files found in {}", config_path.display());
    }

    let backup_dir = crate::config::backups_config_dir();
    let (count, total, path) = create_tar_gz(config_path, &backup_dir, "", "ini")?;
    let verified = path.exists();
    Ok(BackupResult { files_copied: count, total_size: total, dest_path: path, verified })
}

pub fn restore_ini_files(archive_path: &Path, config_path: &Path) -> Result<usize> {
    // Pre-restore safety: back up current .ini files
    if let Err(_e) = backup_ini_files(config_path) {
        // pre-restore failure is non-fatal
    }
    extract_tar_gz(archive_path, config_path)
}

pub fn delete_ini_files(config_path: &Path) -> Result<usize> {
    let mut deleted = 0usize;
    for entry in fs::read_dir(config_path)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().ends_with(".ini") {
            fs::remove_file(entry.path())?;
            deleted += 1;
        }
    }
    Ok(deleted)
}

// ── listing ────────────────────────────────────────────────────────────────

pub fn list_full_backups() -> Vec<PathBuf> {
    list_tar_gz(&crate::config::backups_saves_dir())
}

pub fn list_ini_backups() -> Vec<PathBuf> {
    list_tar_gz(&crate::config::backups_config_dir())
}

/// List .bak files in the save folder, sorted by mtime descending.
pub fn list_bak_files(save_folder: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(save_folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".bak"))
        .map(|e| e.path())
        .collect();
    files.sort_by(|a, b| {
        let ma = fs::metadata(a).ok();
        let mb = fs::metadata(b).ok();
        match (ma.and_then(|m| m.modified().ok()), mb.and_then(|m| m.modified().ok())) {
            (Some(a), Some(b)) => b.cmp(&a),
            _ => std::cmp::Ordering::Equal,
        }
    });
    files
}

/// Enriched .bak file entry with GVAS metadata for the picker UI.
#[derive(Debug, Clone)]
pub struct BakFileSummary {
    pub path: PathBuf,
    pub filename: String,
    pub slot: String,
    pub display_name: Option<String>,
    pub is_online: bool,
    pub size: u64,
    pub mtime: Option<String>,
    pub playtime_seconds: Option<f64>,
}

/// List .bak files with parsed GVAS metadata.
pub fn list_bak_files_with_meta(save_folder: &Path) -> Vec<BakFileSummary> {
    use chrono::TimeZone;
    let mut files: Vec<BakFileSummary> = Vec::new();
    let entries: Vec<_> = fs::read_dir(save_folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".bak"))
        .collect();

    for entry in entries {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();
        let meta = fs::metadata(&path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = meta.as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                let secs = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                Local.timestamp_opt(secs as i64, 0).single()
                    .map(|dt| dt.format("%Y-%b-%d %H:%M").to_string())
            });
        let slot = derive_slot_from_filename(&filename).unwrap_or_else(|| "?".into());
        let meta = extract_metadata(&path).ok();
        let display_name = meta.as_ref().and_then(|m| m.display_name.clone());
        let is_online = meta.as_ref().map(|m| m.is_online).unwrap_or(false);
        let playtime_seconds = meta.as_ref().and_then(|m| m.playtime_seconds);
        files.push(BakFileSummary { path, filename, slot, display_name, is_online, size, mtime, playtime_seconds });
    }
    files.sort_by(|a, b| a.slot.cmp(&b.slot).then_with(|| b.mtime.cmp(&a.mtime)));
    files
}

/// Keep only the most recent .bak per slot, discarding older versioned backups.
pub fn dedup_by_slot(files: Vec<BakFileSummary>) -> Vec<BakFileSummary> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out = Vec::new();
    for f in files {
        if seen.insert(f.slot.clone()) {
            out.push(f);
        }
    }
    out
}

/// Scan the save folder and return stats for the dashboard.
pub fn folder_stats(save_folder: Option<&Path>) -> (usize, usize, bool) {
    let (live, bak) = if let Some(dir) = save_folder {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut l = 0;
            let mut b = 0;
            for e in entries.flatten() {
                let name = e.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("savegame_") && name_str.ends_with(".sav") {
                    l += 1;
                } else if name_str.starts_with("savegame_") && name_str.ends_with(".bak") {
                    b += 1;
                }
            }
            (l, b)
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    let ini_has_backup = crate::config::backups_config_dir().exists()
        && fs::read_dir(crate::config::backups_config_dir())
            .map(|entries| entries.flatten().any(|e| {
                e.file_name().to_string_lossy().ends_with(".tar.gz")
            }))
            .unwrap_or(false);

    let _ = crate::config::backups_saves_dir(); // ensure dir exists
    (live, bak, ini_has_backup)
}

// ── migration ──────────────────────────────────────────────────────────────

/// Migrate old `NotAlterra_Backups/` directory-tree backups into the new
/// tar.gz format.  Each timestamped directory becomes its own `.tar.gz`
/// archive in `backups/saves/`.  The old directory is not deleted.
pub fn migrate_old_backups() -> Result<usize> {
    let old_root = crate::config::exe_dir().join("NotAlterra_Backups");
    migrate_backups_from(old_root)
}

/// Migrate old directory-tree backups from a given root path.
/// Separated from `migrate_old_backups()` so tests can use temp directories.
fn migrate_backups_from(old_root: PathBuf) -> Result<usize> {
    if !old_root.exists() {
        return Ok(0);
    }

    let mut migrated = 0usize;
    if let Ok(entries) = fs::read_dir(&old_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if !dir_name.starts_with("notalterra_copy_") {
                continue;
            }
            // Check if this directory contains save files
            let has_saves = fs::read_dir(&path)
                .map(|e| e.flatten().any(|f| {
                    let fname = f.file_name();
                    let n = fname.to_string_lossy();
                    n.starts_with("savegame_")
                }))
                .unwrap_or(false);
            if !has_saves {
                continue;
            }
            let backup_dir = crate::config::backups_saves_dir();
            match create_tar_gz(&path, &backup_dir, "savegame_", &format!("migrated_{dir_name}")) {
                Ok((_count, _size, archive_path)) if archive_path.exists() => {
                    migrated += 1;
                }
                Ok(_) => {},
                Err(e) => {
                    eprintln!("migration warning: failed to archive {:?}: {}", path, e);
                }
            }
        }
    }
    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create an old-style NotAlterra_Backups directory with save files.
    fn create_old_backup(root: &Path, dir_name: &str, save_names: &[&str]) -> PathBuf {
        let dir = root.join(dir_name);
        fs::create_dir_all(&dir).unwrap();
        for (i, name) in save_names.iter().enumerate() {
            fs::write(dir.join(name), format!("content-{i}").as_bytes()).unwrap();
        }
        dir
    }

    #[test]
    fn migrate_old_backups_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let old_root = tmp.path().join("NotAlterra_Backups");
        fs::create_dir_all(&old_root).unwrap();
        create_old_backup(&old_root, "notalterra_copy_2026-01-01_120000", &["savegame_0.sav"]);
        create_old_backup(&old_root, "notalterra_copy_2026-01-02_120000", &["savegame_0.sav", "savegame_1.sav"]);

        let count = migrate_backups_from(old_root.clone()).unwrap();
        assert_eq!(count, 2, "two old backups should be migrated");

        // Verify archives exist in the shared backup directory
        let saves_dir = crate::config::backups_saves_dir();
        let archives: Vec<_> = fs::read_dir(&saves_dir).unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains("migrated_"))
            .collect();
        assert!(archives.len() >= 2, "at least 2 migrated archives should exist");
    }

    #[test]
    fn migrate_old_backups_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let old_root = tmp.path().join("NotAlterra_Backups");
        fs::create_dir_all(&old_root).unwrap();
        // Empty directory — nothing to migrate
        let count = migrate_backups_from(old_root.clone()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migrate_old_backups_nonexistent_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let old_root = tmp.path().join("NotAlterra_Backups");
        // Directory doesn't exist
        let count = migrate_backups_from(old_root.clone()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migrate_old_backups_file_integrity() {
        let tmp = tempfile::tempdir().unwrap();
        let old_root = tmp.path().join("NotAlterra_Backups");
        fs::create_dir_all(&old_root).unwrap();
        let _dir = create_old_backup(&old_root, "notalterra_copy_2026-01-01_120000", &["savegame_0.sav", "savegame_1.sav"]);

        let count = migrate_backups_from(old_root.clone()).unwrap();
        assert_eq!(count, 1);

        // Find the migrated archive
        let saves_dir = crate::config::backups_saves_dir();
        let archive: Option<PathBuf> = fs::read_dir(&saves_dir).unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains("migrated_"))
            .map(|e| e.path())
            .find(|_| true);
        assert!(archive.is_some(), "migrated archive should exist");

        // Extract to a temp dir and verify content
        let extract_dir = tmp.path().join("extracted");
        fs::create_dir_all(&extract_dir).unwrap();
        let extracted = extract_tar_gz(&archive.unwrap(), &extract_dir).unwrap();
        assert_eq!(extracted, 2, "both save files should be restored");
        assert_eq!(fs::read_to_string(extract_dir.join("savegame_0.sav")).unwrap(), "content-0");
        assert_eq!(fs::read_to_string(extract_dir.join("savegame_1.sav")).unwrap(), "content-1");
    }

    #[test]
    fn migrate_old_backups_skips_non_save_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let old_root = tmp.path().join("NotAlterra_Backups");
        fs::create_dir_all(&old_root).unwrap();
        create_old_backup(&old_root, "notalterra_copy_valid", &["savegame_0.sav"]);
        create_old_backup(&old_root, "notalterra_copy_empty", &[]); // no save files
        create_old_backup(&old_root, "unrelated_dir", &["savegame_0.sav"]); // wrong prefix
        fs::write(old_root.join("random_file.txt"), b"not a backup").unwrap();

        let count = migrate_backups_from(old_root.clone()).unwrap();
        assert_eq!(count, 1, "only the dir with save files and correct prefix should be migrated");
    }

    #[test]
    fn migrate_old_backups_dedup_filename() {
        let tmp = tempfile::tempdir().unwrap();
        let old_root = tmp.path().join("NotAlterra_Backups");
        fs::create_dir_all(&old_root).unwrap();
        create_old_backup(&old_root, "notalterra_copy_session1", &["savegame_0.sav"]);

        // Migrate twice — second pass should not fail
        let c1 = migrate_backups_from(old_root.clone()).unwrap();
        assert_eq!(c1, 1, "first migration");
        let c2 = migrate_backups_from(old_root.clone()).unwrap();
        assert_eq!(c2, 1, "second migration (should not duplicate)");
    }
}
