//! config.ini read/write.
//!
//! The file sits next to the binary and stores cached paths plus the
//! disclaimer-acceptance flag.  Format is a flat `[alterra]` section:
//!
//! ```ini
//! [alterra]
//! save_path = C:\Users\...\Subnautica2\Saved\SaveGames
//! save_scan = 2026-05-26 19:45:22
//! disclaimer_accepted = true
//! ini_path = C:\Users\...\Subnautica2\Saved\Config\Windows
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// In-memory representation of the config.ini keys.
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    /// Last-known save folder (SaveGames)
    pub save_path: Option<String>,
    /// Last-known Config\Windows folder
    pub ini_path: Option<String>,
    /// Timestamp of last successful scan
    pub save_scan: Option<String>,
    /// Whether disclaimer was accepted
    pub disclaimer_accepted: bool,
}

/// Path to config.ini alongside the binary.
pub fn ini_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config.ini")
}

/// Read and parse config.ini.
pub fn load_config(path: &Path) -> Result<AppConfig> {
    let mut cfg = AppConfig::default();

    if !path.exists() {
        return Ok(cfg);
    }

    let f = fs::File::open(path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('[') {
            continue;
        }

        if let Some(val) = trimmed.strip_prefix("save_path").and_then(strip_eq) {
            cfg.save_path = Some(val.to_string());
        } else if let Some(val) = trimmed.strip_prefix("ini_path").and_then(strip_eq) {
            cfg.ini_path = Some(val.to_string());
        } else if let Some(val) = trimmed.strip_prefix("save_scan").and_then(strip_eq) {
            cfg.save_scan = Some(val.to_string());
        } else if let Some(val) = trimmed.strip_prefix("disclaimer_accepted").and_then(strip_eq) {
            cfg.disclaimer_accepted = val.trim() == "true";
        }
    }

    Ok(cfg)
}

/// Strip `=` and surrounding whitespace.  Returns `Some(value)`.
fn strip_eq(s: &str) -> Option<&str> {
    Some(s.trim().strip_prefix('=')?.trim())
}

/// Write config back to disk, preserving non-managed keys.
pub fn save_config(path: &Path, cfg: &AppConfig) -> Result<()> {
    // Preserve unknown keys by reading the old file first.
    let preserved = preserved_keys(path).unwrap_or_default();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let mut f = fs::File::create(path)
        .with_context(|| format!("cannot write {}", path.display()))?;

    writeln!(f, "[alterra]")?;

    if let Some(ref lp) = cfg.save_path {
        writeln!(f, "save_path = {lp}")?;
    }
    if let Some(ref cp) = cfg.ini_path {
        writeln!(f, "ini_path = {cp}")?;
    }
    if let Some(ref ls) = cfg.save_scan {
        writeln!(f, "save_scan = {ls}")?;
    }
    writeln!(
        f,
        "disclaimer_accepted = {}",
        cfg.disclaimer_accepted
    )?;

    // Append any extra keys we didn't touch
    for (k, v) in &preserved {
        writeln!(f, "{k} = {v}")?;
    }

    Ok(())
}

/// Collect key/value pairs from the old config that this tool doesn't own.
fn preserved_keys(path: &Path) -> Result<Vec<(String, String)>> {
    let known = &["save_path", "ini_path", "save_scan", "disclaimer_accepted"];
    let mut out = Vec::new();

    let f = fs::File::open(path)?;
    for line in BufReader::new(f).lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('[') {
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let key = trimmed[..eq].trim();
            if !known.contains(&key) {
                let val = trimmed[eq + 1..].trim();
                out.push((key.to_string(), val.to_string()));
            }
        }
    }
    Ok(out)
}
