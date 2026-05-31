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
#[cfg(target_os = "windows")]
pub fn game_running() -> bool {
    let out = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq Subnautica2.exe", "/NH"])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("Subnautica2.exe"),
        Err(_) => false,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn game_running() -> bool {
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

/// Path to transaction.log next to the binary.
pub fn log_path() -> PathBuf {
    exe_dir().join("transaction.log")
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Maximum lines before rotation.
const MAX_LOG_LINES: usize = 10_000;

/// Append a timestamped log entry.  Auto-rotates if the log exceeds 10k lines.
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

/// Check whether a path looks like a network/UNC path (for warning purposes).
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
