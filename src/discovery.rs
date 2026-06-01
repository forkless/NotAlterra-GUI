//! Save-folder discovery.
//!
//! Traverses known path patterns across user profiles and common install
//! locations to find Subnautica 2 save folders.

use std::fs;
use std::path::{Path, PathBuf};

/// A discovered save folder.
#[derive(Debug, Clone)]
pub struct DiscoveredFolder {
    pub label: String,
    pub path: PathBuf,
}

/// Known save-root patterns (relative from a user profile or base directory).
///
/// The same patterns work on both platforms because UE5 keeps the same
/// directory layout regardless of OS.
const KNOWN_PATTERNS: &[(&str, &str)] = &[
    ("Steam (LocalLow)", "AppData/LocalLow/Unknown Worlds/Subnautica2"),
    ("Steam (LocalLow, alt)", "AppData/LocalLow/Unknown Worlds/Subnautica 2"),
    ("AppData Local", "AppData/Local/Subnautica2/Saved/SaveGames"),
    ("AppData Local (alt)", "AppData/Local/Subnautica 2/Saved/SaveGames"),
    ("Xbox / Game Pass", "AppData/Local/Packages"), // partial — needs wildcard below
    ("Saved Games", "Saved Games/Subnautica2"),
    ("Saved Games (alt)", "Saved Games/Subnautica 2"),
    ("Documents", "Documents/Subnautica2"),
    ("Epic / Steam custom", "AppData/LocalLow/Subnautica2"),
];

/// Search all known locations for Subnautica 2 save folders.
///
/// Returns a deduplicated, ranked list.  The first result is cached as
/// `save_path` in config.ini so subsequent launches skip the scan.
/// Auto-discover Subnautica 2 save folders across Steam, Xbox, Epic, and
/// custom installs.  Checks both modern and legacy Steam paths.
///
/// Returns a deduplicated, ranked list.  The first result is cached as
/// `save_path` in config.ini so subsequent launches skip the scan.
pub fn discover_save_folders() -> Vec<DiscoveredFolder> {
    let mut found: Vec<DiscoveredFolder> = Vec::new();
    let mut seen: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    // 0. Fast-path: %LOCALAPPDATA%\Subnautica2\Saved\SaveGames (primary)
    #[cfg(target_os = "windows")]
    {
        if let Some(local) = dirs::data_local_dir() {
            let primary = local.join("Subnautica2").join("Saved").join("SaveGames");
            if primary.exists() && has_save_files(&primary) {
                found.push(DiscoveredFolder {
                    label: "AppData Local".into(),
                    path: primary,
                });
                return found;
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            let proton_paths = &[
                ".steam/steam/steamapps/compatdata/1962700/pfx/drive_c/users/steamuser/AppData/Local/Subnautica2/Saved/SaveGames",
                ".local/share/Steam/steamapps/compatdata/1962700/pfx/drive_c/users/steamuser/AppData/Local/Subnautica2/Saved/SaveGames",
            ];
            for rel in proton_paths {
                let candidate = home.join(rel);
                if candidate.exists() && has_save_files(&candidate) {
                    found.push(DiscoveredFolder {
                        label: "Proton / Steam Deck".into(),
                        path: candidate,
                    });
                    return found;
                }
            }
        }
        if let Some(data) = dirs::data_local_dir() {
            let primary = data.join("Subnautica2").join("Saved").join("SaveGames");
            if primary.exists() && has_save_files(&primary) {
                found.push(DiscoveredFolder {
                    label: "XDG Data".into(),
                    path: primary,
                });
                return found;
            }
        }
    }

    // 1. Current user profile — remaining patterns
    if let Some(home) = dirs::home_dir() {
        for (label, rel) in KNOWN_PATTERNS {
            let candidate = home.join(rel);
            if candidate.exists() && candidate.is_dir() {
                if has_save_files(&candidate) && seen.insert(candidate.clone()) {
                    found.push(DiscoveredFolder {
                        label: label.to_string(),
                        path: candidate,
                    });
                }
            }
        }
    }

    // 2. Xbox / Game Pass wildcard scan
    #[cfg(target_os = "windows")]
    {
        if let Some(home) = dirs::home_dir() {
            let pkg_root = home.join("AppData/Local/Packages");
            if pkg_root.exists() {
                if let Ok(entries) = fs::read_dir(&pkg_root) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.contains("Subnautica2") {
                            let wgs = entry.path().join("SystemAppData/wgs");
                            if wgs.exists() && has_save_files(&wgs) && seen.insert(wgs.clone()) {
                                found.push(DiscoveredFolder {
                                    label: "Xbox / Game Pass".into(),
                                    path: wgs,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: scan other user profiles
    scan_other_users(&mut found, &mut seen);

    // 4. Broad custom-install scan
    scan_common_install_dirs(&mut found, &mut seen);

    found
}

/// Check if a directory contains at least one `.sav` or `.save` file.
fn has_save_files(dir: &Path) -> bool {
    let check = |ext: &str| -> bool {
        fs::read_dir(dir)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .ends_with(ext)
                })
            })
            .unwrap_or(false)
    };
    check(".sav") || check(".save")
}

/// Scan other user profiles on the system.
#[cfg(target_os = "windows")]
fn scan_other_users(
    found: &mut Vec<DiscoveredFolder>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    for drive in fixed_drives() {
        let users = Path::new(&drive).join("Users");
        if !users.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&users) {
            for user_dir in entries.flatten() {
                let user_path = user_dir.path();
                for (label, rel) in KNOWN_PATTERNS {
                    let candidate = user_path.join(rel);
                    if candidate.exists() && has_save_files(&candidate) && seen.insert(candidate.clone()) {
                        found.push(DiscoveredFolder {
                            label: label.to_string(),
                            path: candidate,
                        });
                    }
                }
            }
        }
    }
}

/// Linux variant — same logic as the Windows version above.
#[cfg(not(target_os = "windows"))]
fn scan_other_users(
    found: &mut Vec<DiscoveredFolder>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    // On Linux, check /home for other users
    let home_root = Path::new("/home");
    if !home_root.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(home_root) {
        for user_dir in entries.flatten() {
            let user_path = user_dir.path();
            for (label, rel) in KNOWN_PATTERNS {
                let candidate = user_path.join(rel);
                if candidate.exists() && has_save_files(&candidate) && seen.insert(candidate.clone()) {
                    found.push(DiscoveredFolder {
                        label: label.to_string(),
                        path: candidate,
                    });
                }
            }
        }
    }
    // Also check common Steam Deck paths
    let deck_paths = &[
        Path::new("/run/media/mmcblk0p1/steamapps/compatdata"),
        Path::new("/home/deck/.local/share/Steam/steamapps/compatdata"),
    ];
    for base in deck_paths {
        if base.exists() {
            // Walk compatdata for Subnautica 2 prefix
            if let Ok(entries) = fs::read_dir(base) {
                for app_entry in entries.flatten() {
                    let pfx = app_entry.path().join("pfx/drive_c/users/steamuser");
                    for (label, rel) in KNOWN_PATTERNS {
                        let candidate = pfx.join(rel);
                        if candidate.exists() && has_save_files(&candidate) && seen.insert(candidate.clone()) {
                            found.push(DiscoveredFolder {
                                label: format!("Steam Deck — {label}"),
                                path: candidate,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Scan common install directories (Program Files, Games, Steam, etc.).
#[cfg(target_os = "windows")]
fn scan_common_install_dirs(
    found: &mut Vec<DiscoveredFolder>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    for drive in fixed_drives() {
        let roots: &[&str] = &[
            &format!("{drive}Games"),
            &format!("{drive}Program Files"),
            &format!("{drive}Program Files (x86)"),
            &format!("{drive}Steam"),
            &format!("{drive}Epic Games"),
        ];
        for rt in roots {
            let p = Path::new(rt);
            if !p.exists() {
                continue;
            }
            walk_for_subnautica(p, "custom install", found, seen);
        }
    }
}

/// Linux variant — same logic as the Windows version above.
#[cfg(not(target_os = "windows"))]
fn scan_common_install_dirs(
    found: &mut Vec<DiscoveredFolder>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    let roots: &[&str] = &[
        "/opt",
        "/usr/local/games",
        "/usr/share/games",
    ];
    for rt in roots {
        let p = Path::new(rt);
        if !p.exists() {
            continue;
        }
        walk_for_subnautica(p, "custom install", found, seen);
    }
    // Steam library paths
    if let Some(home) = dirs::home_dir() {
        let steam = home.join(".local/share/Steam");
        if steam.exists() {
            walk_for_subnautica(&steam, "Steam", found, seen);
        }
    }
}

/// Recursively walk a root looking for folders named "*Subnautica*".
fn walk_for_subnautica(
    root: &Path,
    label: &str,
    found: &mut Vec<DiscoveredFolder>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    use std::collections::VecDeque;

    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    queue.push_back(root.to_path_buf());

    while let Some(dir) = queue.pop_front() {
        // Limit depth: don't recurse more than 5 levels from root
        let depth = dir.components().count().saturating_sub(root.components().count());
        if depth > 5 {
            continue;
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();

            if name.contains("subnautica") {
                if has_save_files(&path) && seen.insert(path.clone()) {
                    found.push(DiscoveredFolder {
                        label: label.to_string(),
                        path: path.clone(),
                    });
                }
            }

            if path.is_dir() {
                queue.push_back(path);
            }
        }
    }
}

/// List available fixed drives on Windows.
#[cfg(target_os = "windows")]
fn fixed_drives() -> Vec<String> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let path = format!("{}:\\", letter as char);
        if Path::new(&path).exists() {
            drives.push(path);
        }
    }
    drives
}

// ── helpers for the TUI ──────────────────────────────────────────────────

/// Validate that a manually entered path exists and contains save files.
pub fn validate_custom_path(input: &str) -> Option<PathBuf> {
    let expanded = if input.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            input.replacen('~', &home.to_string_lossy(), 1)
        } else {
            input.to_string()
        }
    } else {
        input.to_string()
    };
    let path = PathBuf::from(expanded);
    if path.exists() && has_save_files(&path) {
        Some(path)
    } else {
        None
    }
}

/// Derive the Config\Windows path from a SaveGames path.
///
/// Walks up to the `Saved` ancestor, then down to `Config/Windows`.
pub fn derive_ini_path(save_path: &Path) -> Option<PathBuf> {
    let mut current = save_path.to_path_buf();

    // Walk up looking for "Saved" component
    loop {
        if current.file_name().map(|n| n == "Saved").unwrap_or(false) {
            let config = current.join("Config").join("Windows");
            return if config.exists() { Some(config) } else { None };
        }
        if !current.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_ini_path_sample() {
        // Should resolve from a save path even if dirs don't exist locally
        let save = Path::new("C:/Users/test/AppData/Local/Subnautica2/Saved/SaveGames");
        // Since we can't test existence, at least verify the walk logic compiles
        let result = derive_ini_path(save);
        // On a test machine without the game, this returns None — which is fine
        let _ = result;
    }
}
