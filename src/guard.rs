//! Game-running guard and transaction logging.
//!
//! Before launch and before each destructive operation, check whether
//! Subnautica 2 is running — the game holds file locks on `.sav` files
//! while active.

use anyhow::Result;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

// ── process detection ──────────────────────────────────────────────────────

/// Return `true` if Subnautica 2 appears to be running.
///
/// Always returns `false` — process detection (`tasklist` / `pgrep`) is
/// intentionally disabled to avoid Windows Defender false positives
/// (Trojan:Win32/Wacatac.C!ml).  Users are reminded to close the game
/// manually before using the tool.
#[cfg(target_os = "windows")]
pub fn game_running() -> bool {
    // Process detection disabled — avoid AV false-positives.
    // Close Subnautica 2 manually before using NotAlterra.
    false
}
#[allow(dead_code)]
/// Check if Subnautica 2 is running via tasklist (Windows). Dormant.
fn _game_running_windows() -> bool {
    let out = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq Subnautica2.exe", "/NH"])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("Subnautica2.exe"),
        Err(_) => false,
    }
}

/// Check if Subnautica 2 is currently running (Linux).
///
/// Always returns `false` — process detection via `pgrep` is disabled to
/// avoid false positives. Users are reminded to close the game manually.
#[cfg(not(target_os = "windows"))]
pub fn game_running() -> bool {
    false
}
#[allow(dead_code)]
/// Check if Subnautica 2 is running via pgrep (Linux). Dormant.
fn _game_running_linux() -> bool {
    let patterns = &["Subnautica2", "Subnautica2-Win64-Shipping"];
    for pat in patterns {
        if let Ok(out) = std::process::Command::new("pgrep")
            .args(["-ci", pat])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Ok(n) = s.trim().parse::<u32>() {
                if n > 0 {
                    return true;
                }
            }
        }
    }
    false
}

// ── transaction logging ────────────────────────────────────────────────────

/// Migrate the old `transaction.log` (next to the binary) into the new
/// `logs/` directory.  Appends old entries to the new file, then renames
/// the old file to `.migrated` so it won't be migrated again.
/// Returns `true` if an old log was found and handled.
pub fn migrate_old_log() -> bool {
    let old = exe_dir().join("transaction.log");
    if !old.exists() {
        return false;
    }
    let new = log_path();
    // Read old content
    if let Ok(content) = std::fs::read_to_string(&old) {
        if !content.trim().is_empty() {
            // Append to new log with a migration header
            let header = format!(
                "───── migrated from old location [{ts}] ─────\n",
                ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new)
            {
                use std::io::Write;
                let _ = write!(f, "{header}{content}");
            }
        }
    }
    // Rename old file so it won't be processed again
    let migrated = old.with_extension("log.migrated");
    let _ = std::fs::rename(&old, &migrated);
    true
}

/// Path to `transaction.log` inside the `logs/` directory.  All timestamped
/// actions are appended here for audit trail purposes.
pub fn log_path() -> PathBuf {
    let p = exe_dir().join("logs").join("transaction.log");
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    p
}

/// Return the directory containing the running executable.
fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Maximum lines before rotation.
const MAX_LOG_LINES: usize = 10_000;

/// Append a timestamped log entry to `transaction.log`.
///
/// Format: `YYYY-MM-DD HH:MM:SS | ACTION   | detail | result`
/// Auto-rotates if the log exceeds 10,000 lines — the oldest lines are
/// discarded, keeping only the most recent 10,000.
pub fn log_action(
    action: &str,
    detail: &str,
    result: &str,
    log_path: &Path,
) -> Result<()> {
    let stamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("{stamp} | {action:<8} | {detail} | {result}\n");

    // Rotate if needed
    if log_path.exists() {
        if let Ok(content) = fs::read_to_string(log_path) {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() > MAX_LOG_LINES {
                let keep: String = lines[lines.len() - MAX_LOG_LINES..].join("\n");
                fs::write(log_path, keep + "\n").ok();
            }
        }
    }

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    f.write_all(line.as_bytes())?;
    Ok(())
}
/// Truncate a filesystem path to start at `Subnautica2/` or `Subnautica2\`,
/// stripping the user-specific prefix for privacy-safe logging.
///
/// Returns the original path unchanged if `Subnautica2` is not found in the
/// input (e.g. custom paths entered via `Set save folder`).
pub fn sanitize_path(p: &str) -> String {
    let needle = "Subnautica2";
    let sep = if p.contains('\\') { "\\" } else { "/" };
    if let Some(pos) = p.find(needle) {
        format!("...{sep}{}", &p[pos..])
    } else {
        p.to_string()
    }
}

/// Check whether a path looks like a network/UNC path — for warning purposes.
/// Matches paths starting with `\\` (Windows) or `//` (Linux).
pub fn is_network_path(p: &str) -> bool {
    p.starts_with("\\\\") || p.starts_with("//")
}

/// Estimate free space on the volume containing `path` in bytes.
/// Returns `None` on platforms or filesystems where we can't determine this.
pub fn available_space(_path: &Path) -> Option<u64> {
    // Advisory only — the PowerShell original had a try/catch fallback.
    // For a cross-platform build without platform-specific FFI, we
    // return None and skip the disk-space warning.
    None
}
