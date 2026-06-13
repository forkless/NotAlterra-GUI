//! Path utilities and persistent app configuration.
//!
//! The save-folder path and backup root are persisted to `app.ini` under the
//! platform config directory so they survive between sessions. The disclaimer
//! acceptance is tracked via a sentinel file in the same directory.
//!
//! The backup root defaults to `~/NotAlterra` and can be changed via the
//! `Set backup location` menu item.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Custom backup root directory, set via the `Set backup location` menu.
/// When `None`, falls back to `exe_dir().join("backups")`.
static BACKUP_ROOT: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Set a custom backup root directory.  Passes ownership.
pub fn set_backup_root(path: PathBuf) {
    if let Ok(mut root) = BACKUP_ROOT.lock() {
        *root = Some(path);
    }
}

/// Return the current backup root, or the default user-profile path.
pub fn get_backup_root() -> PathBuf {
    BACKUP_ROOT
        .lock()
        .ok()
        .and_then(|r| r.clone())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join("NotAlterra"))
                .unwrap_or_else(exe_dir)
        })
}

/// Path to the disclaimer sentinel file in the config directory.
pub fn sentinel_path() -> PathBuf {
    config_base_dir().join("NOTALTERRA_LICENSE_ACCEPTED")
}

/// Return `true` if the disclaimer sentinel exists.
pub fn disclaimer_accepted() -> bool {
    sentinel_path().exists()
}

/// Create the disclaimer sentinel (0-byte file).
pub fn accept_disclaimer() -> std::io::Result<()> {
    std::fs::write(sentinel_path(), [])?;
    Ok(())
}

/// Remove the disclaimer sentinel if it exists.
pub fn reject_disclaimer() -> std::io::Result<()> {
    let p = sentinel_path();
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(())
}

/// Path to the stale `config.ini` from v0.3.0 and earlier.
pub fn stale_config_path() -> PathBuf {
    exe_dir().join("config.ini")
}

/// Remove the stale `config.ini` if it exists.  Returns `true` if removed.
pub fn cleanup_stale_config() -> bool {
    let path = stale_config_path();
    if path.exists() {
        std::fs::remove_file(&path).ok();
        true
    } else {
        false
    }
}

/// Path to the `saves/` directory under the backup root (tar.gz archives).
/// Auto-creates the directory on first call.
pub fn backups_saves_dir() -> PathBuf {
    let p = get_backup_root().join("backups").join("saves");
    std::fs::create_dir_all(&p).ok();
    p
}

/// Path to the `ue5/` subdirectory under the backup root for UE5 Config `.ini`
/// backup archives.  Auto-creates the directory on first call.
pub fn backups_config_dir() -> PathBuf {
    let p = get_backup_root().join("backups").join("ue5");
    std::fs::create_dir_all(&p).ok();
    p
}

/// Return `~/NotAlterra` as the user's data directory for save/ue5 backups.
/// Falls back to `exe_dir()` if home is not available.
fn home_notalterra_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join("NotAlterra"))
        .unwrap_or_else(exe_dir)
}

// ── persistent app config ────────────────────────────────────────────────────

/// Fixed base directory for `app.ini` and the sentinel file, under the
/// standard platform config location.  Separate from backup data so that
/// `~/NotAlterra` remains the user's visible backup data directory.
fn config_base_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|d| d.join("NotAlterra").join("config"))
        .unwrap_or_else(exe_dir)
}

/// Path to the persistent `app.ini` configuration file.
pub fn app_ini_path() -> PathBuf {
    let p = config_base_dir().join("app.ini");
    std::fs::create_dir_all(p.parent().unwrap_or(Path::new("."))).ok();
    p
}

/// Session-lifetime configuration loaded from and persisted to `app.ini`.
#[derive(Debug, Default)]
pub struct AppConfig {
    pub save_folder: Option<String>,
    pub backup_root: Option<String>,
}

/// Load `app.ini` from the fixed config directory.
/// Returns default (empty) values if the file does not exist or cannot be read.
pub fn load_app_config() -> AppConfig {
    let path = app_ini_path();
    if !path.exists() {
        return AppConfig::default();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return AppConfig::default(),
    };
    let mut cfg = AppConfig::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let val = value.trim();
            match key {
                "save_folder" => cfg.save_folder = Some(val.to_string()),
                "backup_root" => cfg.backup_root = Some(val.to_string()),
                _ => {}
            }
        }
    }
    cfg
}

/// Write the current session paths to `app.ini` in the fixed config directory.
pub fn save_app_config(save_folder: Option<&str>, backup_root: Option<&str>) {
    let path = app_ini_path();
    let mut content = String::from(
        "# NotAlterra configuration\n\
         # This file is auto-generated. Edit while the tool is not running.\n\n",
    );
    if let Some(s) = save_folder {
        content.push_str(&format!("save_folder = {s}\n"));
    }
    if let Some(r) = backup_root {
        content.push_str(&format!("backup_root = {r}\n"));
    }
    let _ = std::fs::write(&path, content);
}

/// Ensure a directory exists, creating all parents as needed.
pub fn ensure_dir(path: PathBuf) {
    let _ = std::fs::create_dir_all(&path);
}

/// Return the directory containing the running executable.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}
