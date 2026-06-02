//! Minimal path utilities — no persistent config.
//!
//! Save folder and Config/Windows paths are now session-only, entered
//! via the `Set save folder` menu. The disclaimer acceptance is tracked
//! via a 0-byte sentinel file (`NotAlterra_LICENSE_ACCEPTED`) next to the
//! binary instead of a config file.

use std::path::{Path, PathBuf};

/// Path to the disclaimer sentinel file alongside the binary.
pub fn sentinel_path() -> PathBuf {
    exe_dir().join("NotAlterra_LICENSE_ACCEPTED")
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

/// Path to the `backups/saves/` directory (tar.gz archives).
/// Auto-creates the directory on first call.
pub fn backups_saves_dir() -> PathBuf {
    let p = exe_dir().join("backups").join("saves");
    std::fs::create_dir_all(&p).ok();
    p
}

/// Path to the `backups/config/` directory (.ini tar.gz archives).
/// Auto-creates the directory on first call.
pub fn backups_config_dir() -> PathBuf {
    let p = exe_dir().join("backups").join("config");
    std::fs::create_dir_all(&p).ok();
    p
}

/// Return the directory containing the running executable.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}


