//! File operations: recover .bak→.sav, backup/restore full, .ini management.
//!
//! The `.bak` recovery bug from the PowerShell script is fixed here:
//! versioned backups (`savegame_0_9.bak`) recover to the canonical
//! `savegame_0.sav`, not `savegame_0_9.sav`.

use crate::gvas::{derive_slot_from_filename, extract_metadata};
use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

// ── public operation types ─────────────────────────────────────────────────

/// Result of a backup operation.
#[derive(Debug, Clone)]
pub struct BackupResult {
    pub files_copied: usize,
    pub total_size: u64,
    pub dest_dir: PathBuf,
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

/// Restore a `.sav` file from a `.bak` backup.
///
/// The target is derived from the canonical slot name (e.g.
/// `savegame_0_9.bak` → `savegame_0.sav`).  If a live `.sav` exists,
/// it is renamed to `<target>.old` as a rollback safety net before
/// the backup is copied in.
pub fn recover_bak_to_sav(
    save_folder: &Path,
    bak_filename: &str,
) -> Result<RecoveryResult> {
    let bak_path = save_folder.join(bak_filename);

    if !bak_path.exists() {
        anyhow::bail!("backup file not found: {}", bak_path.display());
    }

    // Sanity: reject tiny files (< 1 KB)
    let meta = fs::metadata(&bak_path)
        .with_context(|| format!("cannot read {}", bak_path.display()))?;
    if meta.len() < 1024 {
        anyhow::bail!(
            "backup file too small ({} bytes) — aborting restore",
            meta.len()
        );
    }

    // Derive the canonical .sav target
    let slot = derive_slot_from_filename(bak_filename)
        .ok_or_else(|| anyhow::anyhow!("cannot derive slot from filename: {bak_filename}"))?;
    let target_name = format!("{slot}.sav");
    let target_path = save_folder.join(&target_name);

    let mut old_saved_as = None;

    // Roll the existing .sav aside
    if target_path.exists() {
        let old_path = save_folder.join(format!("{target_name}.old"));
        fs::rename(&target_path, &old_path)
            .with_context(|| format!("cannot rename {} → {}", target_path.display(), old_path.display()))?;
        old_saved_as = Some(format!("{target_name}.old"));
    }

    // Copy .bak → .sav
    fs::copy(&bak_path, &target_path).with_context(|| {
        format!(
            "cannot copy {} → {}",
            bak_path.display(),
            target_path.display()
        )
    })?;

    Ok(RecoveryResult {
        source: bak_filename.to_string(),
        target: target_name,
        old_saved_as,
    })
}

// ── full backup ────────────────────────────────────────────────────────────

/// Create a full backup of the save folder to `NotAlterra_Backups/notalterra_copy_<timestamp>`.
pub fn create_full_backup(
    save_folder: &Path,
    backup_root: &Path,
) -> Result<BackupResult> {
    let ts = Local::now().format("%Y-%m-%d_%H%M%S");
    let dest = backup_root.join(format!("notalterra_copy_{ts}"));

    fs::create_dir_all(&dest)
        .with_context(|| format!("cannot create backup dir {}", dest.display()))?;

    let mut copied = 0usize;
    let mut total = 0u64;

    if let Err(e) = copy_save_files(save_folder, &dest, &mut copied, &mut total) {
        let _ = fs::remove_dir_all(&dest);
        return Err(e);
    }

    // Verify
    let verified = verify_backup(save_folder, &dest);

    Ok(BackupResult {
        files_copied: copied,
        total_size: total,
        dest_dir: dest,
        verified,
    })
}

/// Restore a full backup into the save folder.
///
/// Creates a pre-restore safety backup first.
pub fn restore_full_backup(
    backup_dir: &Path,
    save_folder: &Path,
    backup_root: &Path,
) -> Result<()> {
    // Pre-restore safety backup
    let ts = Local::now().format("%Y-%m-%d_%H%M%S");
    let pre_restore = backup_root.join(format!("pre_restore_{ts}"));
    fs::create_dir_all(&pre_restore)
        .with_context(|| format!("cannot create pre-restore dir {}", pre_restore.display()))?;
    let mut dummy = 0usize;
    let mut dummy_size = 0u64;
    if copy_save_files(save_folder, &pre_restore, &mut dummy, &mut dummy_size).is_err() {
        let _ = fs::remove_dir_all(&pre_restore);
    }

    // Overwrite save folder with backup
    copy_save_files(backup_dir, save_folder, &mut dummy, &mut dummy_size)?;

    Ok(())
}

// ── .ini management ────────────────────────────────────────────────────────

/// Back up .ini files from the Config\Windows folder.
pub fn backup_ini_files(config_path: &Path, backup_root: &Path) -> Result<BackupResult> {
    let ini_files: Vec<PathBuf> = fs::read_dir(config_path)
        .with_context(|| format!("cannot read {}", config_path.display()))?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".ini")
        })
        .map(|e| e.path())
        .collect();

    if ini_files.is_empty() {
        anyhow::bail!("no .ini files found in {}", config_path.display());
    }

    let ts = Local::now().format("%Y-%m-%d_%H%M%S");
    let dest = backup_root.join(format!("ini_backup_{ts}"));

    fs::create_dir_all(&dest)
        .with_context(|| format!("cannot create ini backup dir {}", dest.display()))?;

    let mut copied = 0usize;
    let mut total = 0u64;

    for f in &ini_files {
        let meta = fs::metadata(f)?;
        total += meta.len();
        let name = f.file_name().unwrap();
        fs::copy(f, dest.join(name))?;
        copied += 1;
    }

    let verified = verify_backup(config_path, &dest);

    Ok(BackupResult {
        files_copied: copied,
        total_size: total,
        dest_dir: dest,
        verified,
    })
}

/// Restore .ini files from a backup into the Config\Windows folder.
pub fn restore_ini_files(
    backup_dir: &Path,
    config_path: &Path,
    backup_root: &Path,
) -> Result<()> {
    // Pre-restore safety
    let ts = Local::now().format("%Y-%m-%d_%H%M%S");
    let pre_restore = backup_root.join(format!("ini_pre_restore_{ts}"));
    fs::create_dir_all(&pre_restore)?;

    // Back up current .ini files
    if let Ok(entries) = fs::read_dir(config_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().ends_with(".ini") {
                fs::copy(entry.path(), pre_restore.join(&name)).ok();
            }
        }
    }

    // Copy all files from backup → config_path
    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        fs::copy(entry.path(), config_path.join(entry.file_name()))?;
    }

    Ok(())
}

/// Delete all .ini files from the Config\Windows folder.
///
/// **Guarded**: refuses to delete unless at least one `ini_backup_*` exists
/// in the backup root.
pub fn delete_ini_files(config_path: &Path, backup_root: &Path) -> Result<usize> {
    // Guard: check for existing backup
    let has_backup = backup_root.exists()
        && fs::read_dir(backup_root)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("ini_backup_")
                })
            })
            .unwrap_or(false);

    if !has_backup {
        anyhow::bail!(
            "no .ini backup found — create a backup first via 'Manage Config > Backup'"
        );
    }

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

/// List existing full backups in the backup root.
pub fn list_full_backups(backup_root: &Path) -> Vec<PathBuf> {
    list_subdirs(backup_root, "notalterra_copy_")
}

/// List existing .ini backups in the backup root.
pub fn list_ini_backups(backup_root: &Path) -> Vec<PathBuf> {
    list_subdirs(backup_root, "ini_backup_")
}

/// List .bak files in a save folder, sorted by mtime descending.
pub fn list_bak_files(save_folder: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(save_folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".bak")
        })
        .map(|e| e.path())
        .collect();

    // Sort by mtime descending
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
}

/// List .bak files with parsed GVAS metadata.
///
/// Each file is read to extract `SlotName` and `DisplayName`.  Files that
/// fail to parse still appear — the metadata fields are simply empty.
pub fn list_bak_files_with_meta(save_folder: &Path) -> Vec<BakFileSummary> {
    use chrono::TimeZone;

    let mut files: Vec<BakFileSummary> = Vec::new();

    let entries: Vec<_> = fs::read_dir(save_folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".bak")
        })
        .collect();

    for entry in entries {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();

        let meta = fs::metadata(&path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                let secs = t
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                Local
                    .timestamp_opt(secs as i64, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            });

        // Use filename-derived slot for grouping (authoritative).
        // Internal SlotName is advisory — files may have been moved.
        let slot = derive_slot_from_filename(&filename).unwrap_or_else(|| "?".into());
        let meta = extract_metadata(&path).ok();
        let display_name = meta.as_ref().and_then(|m| m.display_name.clone());
        let is_online = meta.as_ref().map(|m| m.is_online).unwrap_or(false);

        files.push(BakFileSummary {
            path,
            filename,
            slot,
            display_name,
            is_online,
            size,
            mtime,
        });
    }

    // Sort by mtime descending, then by slot
    files.sort_by(|a, b| {
        a.slot
            .cmp(&b.slot)
            .then_with(|| b.mtime.cmp(&a.mtime))
    });

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
pub fn folder_stats(
    save_folder: Option<&Path>,
    backup_root: &Path,
) -> (usize, usize, bool) {
    let (live, bak) = if let Some(dir) = save_folder {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut l = 0;
            let mut b = 0;
            for e in entries.flatten() {
                let name = e.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".sav") {
                    l += 1;
                } else if name_str.ends_with(".bak") {
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

    let ini = backup_root.exists()
        && fs::read_dir(backup_root)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("ini_backup_")
                })
            })
            .unwrap_or(false);

    (live, bak, ini)
}

// ── internal helpers ───────────────────────────────────────────────────────

/// Copy only files matching `savegame_*` prefix from `src` to `dest`.
fn copy_save_files(
    src: &Path,
    dest: &Path,
    count: &mut usize,
    total_size: &mut u64,
) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let meta = entry.metadata()?;
        if meta.is_dir() {
            continue;
        }
        if meta.is_file() && name_str.starts_with("savegame_") {
            let dest_path = dest.join(&name);
            fs::copy(&entry.path(), &dest_path)?;
            *count += 1;
            *total_size += meta.len();
        }
    }
    Ok(())
}

fn copy_recursive(
    src: &Path,
    dest: &Path,
    count: &mut usize,
    total_size: &mut u64,
) -> Result<()> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let name = entry.file_name();
        let dest_path = dest.join(&name);

        let meta = entry.metadata()?;
        if meta.is_dir() {
            copy_recursive(&src_path, &dest_path, count, total_size)?;
        } else if meta.is_file() {
            fs::copy(&src_path, &dest_path)?;
            *count += 1;
            *total_size += meta.len();
        }
    }
    Ok(())
}

/// Verify that files in `src` match those in `dest` (by relative path + size).
fn verify_backup(src: &Path, dest: &Path) -> bool {
    let Ok(src_entries) = fs::read_dir(src) else { return false };
    for entry in src_entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("savegame_") {
            continue;
        }
        let expected = dest.join(&name);
        if !expected.exists() {
            return false;
        }
        if let (Ok(sm), Ok(dm)) = (entry.metadata(), fs::metadata(&expected)) {
            if sm.len() != dm.len() {
                return false;
            }
        }
    }
    true
}

fn list_subdirs(root: &Path, prefix: &str) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(prefix)
                && e.path().is_dir()
        })
        .map(|e| e.path())
        .collect();
    dirs.sort_by(|a, b| {
        let ma = fs::metadata(a).map(|m| m.modified().ok()).ok().flatten();
        let mb = fs::metadata(b).map(|m| m.modified().ok()).ok().flatten();
        match (ma, mb) {
            (Some(a), Some(b)) => b.cmp(&a),
            _ => std::cmp::Ordering::Equal,
        }
    });
    dirs
}
