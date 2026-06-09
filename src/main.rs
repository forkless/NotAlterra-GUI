#![allow(dead_code)]
//! NotAlterra — Subnautica 2 save-file manager.
//!
//! Cross-platform terminal application.  Locates save folders, recovers
//! .sav files from .bak backups, creates/restores full backups, and
//! manages Config (.ini) files.
//!
//! MIT License.  Not affiliated with Unknown Worlds Entertainment or KRAFTON.

mod config;
mod discovery;
mod guard;
mod gvas;
mod ops;
mod tui;

use anyhow::Result;
use chrono::TimeZone;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::{Path, PathBuf};

const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

/// Entry point — parses args, loads config, and starts the TUI loop.
fn main() -> Result<()> {
    for arg in std::env::args().skip(1) {
        if arg == "--help" || arg == "-h" {
            println!("notalterra {}", VERSION);
            println!("Subnautica 2 save-file manager — cross-platform terminal application.");
            println!();
            println!("Usage:  notalterra [--version | --help]");
            println!();
            println!("Run with no arguments to start the interactive terminal UI.");
            return Ok(());
        }
        if arg == "--version" || arg == "-v" {
            println!("notalterra {}", VERSION);
            return Ok(());
        }
    }

    // ── setup terminal ─────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableBracketedPaste,
        EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // ── teardown ───────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

// ── app state ──────────────────────────────────────────────────────────────

struct App {
    log_path: PathBuf,
    save_folder: Option<PathBuf>,
    tui_state: tui::AppState,
}

impl App {
    fn new() -> Result<Self> {
        let log_path = guard::log_path();
        let tui_state = tui::AppState {
            version: VERSION.to_string(),
            ..Default::default()
        };
        Ok(Self {
            log_path,
            save_folder: None,
            tui_state,
        })
    }

    /// Set the status bar message with optional style.
    fn set_status(&mut self, msg: &str, style: tui::StatusStyle) {
        self.tui_state.status_message = Some(msg.to_string());
        self.tui_state.status_style = style;
    }

    /// Reset the status bar to empty.
    fn clear_status(&mut self) {
        self.tui_state.status_message = None;
    }

    /// Show or hide the spinner indicator on the status bar.
    fn set_spinner(&mut self, active: bool) {
        self.tui_state.spinner_active = active;
        if active {
            self.tui_state.spinner_start = Some(std::time::Instant::now());
        }
    }
}

/// Return the directory containing the running executable.  Used to locate
/// the sentinel file, backups directory, and `transaction.log` alongside
/// the binary.
fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Refresh the dashboard counters (live saves, backups, ini backup status)
/// shown in the header bar.  Called after changing the save folder or after
/// any backup/restore operation.
fn refresh_stats(tui_state: &mut tui::AppState, save_folder: Option<&Path>) {
    tui_state.save_path = save_folder.map(|p| p.display().to_string());
    let (live, bak, ini) = ops::folder_stats(save_folder);
    tui_state.live_save_count = live;
    tui_state.backup_count = bak;
    tui_state.has_ini_backup = ini;
    tui_state.context_path = Some(crate::config::get_backup_root().display().to_string());
}

// ── main loop ──────────────────────────────────────────────────────────────

/// Initialize the terminal and run the main menu loop.
fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new()?;

    // Reminder — the user should close the game before using the tool
    ok_dialog(
        terminal,
        &app,
        "Before You Begin",
        "Please close Subnautica 2 before using NotAlterra.\n\
         \n\
         The game holds file locks on your save files while active.\n\
         Backing up or restoring saves with the game running can\n\
         result in incomplete, corrupt, or overwritten save files.",
    )?;

    // Scaffold backup directory structure
    crate::config::ensure_dir(crate::config::backups_saves_dir());
    crate::config::ensure_dir(crate::config::backups_config_dir());

    // Track whether any migrations occurred for the startup status message
    let mut migrated_something = false;

    // Migrate old transaction.log into logs/ directory
    if guard::migrate_old_log() {
        guard::log_action(
            "MIGRATE",
            "old transaction.log moved to logs/",
            "OK",
            &app.log_path,
        )?;
        migrated_something = true;
    }

    // Clean up stale config.ini from v0.3.0 and earlier
    if crate::config::cleanup_stale_config() {
        guard::log_action("MIGRATE", "old config.ini removed", "SAFE", &app.log_path)?;
        migrated_something = true;
    }

    // Migrate old directory-tree backups to tar.gz
    if let Ok(n) = ops::migrate_old_backups() {
        if n > 0 {
            guard::log_action("MIGRATE", &format!("{n} old backup(s) migrated to tar.gz, originals in NotAlterra_Backups/ untouched"), "OK", &app.log_path)?;
            migrated_something = true;
        }
    }

    // Notify the user about completed migrations
    if migrated_something {
        app.set_status(
            "Data migrated from previous version. Old files remain — delete manually if desired.",
            tui::StatusStyle::Info,
        );
    }

    // Quick check of common save locations (current user only, no profile scans)
    if app.save_folder.is_none() {
        if let Some(path) = discovery::quick_discover() {
            app.save_folder = Some(path);
            refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
        }
    }

    // Load persistent app config — may override quick_discover
    let app_cfg = crate::config::load_app_config();
    if let Some(sf) = app_cfg.save_folder {
        let p = PathBuf::from(&sf);
        if p.exists() {
            app.save_folder = Some(p);
            refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
        }
    }
    if let Some(br) = app_cfg.backup_root {
        let p = PathBuf::from(&br);
        if p.exists() {
            crate::config::set_backup_root(p);
        }
    }

    // Disclaimer flow
    if !crate::config::disclaimer_accepted() {
        match run_disclaimer(terminal, &mut app)? {
            Some(true) => {}    // accepted
            _ => return Ok(()), // declined or cancelled on first launch → exit
        }
    }

    // Main menu loop
    let mut menu_state = ListState::default().with_selected(Some(0));

    loop {
        // Main menu: show path based on highlighted item
        {
            let sel = menu_state.selected().unwrap_or(0);
            let br = crate::config::get_backup_root()
                .to_string_lossy()
                .to_string();
            let ini_path_str = app.save_folder.as_ref().and_then(|sf| {
                discovery::derive_ini_path(sf).map(|p| p.to_string_lossy().to_string())
            });
            app.tui_state.context_path = match sel {
                0 | 1 => None,            // save path (fallback)
                3..=5 => Some(br),        // backup root
                7 => ini_path_str,        // ini Config\Windows path
                _ => Some(String::new()), // blank / disclaimer / exit → no path
            };
        }
        terminal.draw(|f| {
            let cols = f.area().width;
            let rows = f.area().height;
            if cols < 60 || rows < 15 {
                draw_too_small(f);
                return;
            }
            tui::draw_main_menu(f, &mut menu_state, &app.tui_state);
        })?;

        const SKIP: &[usize] = &[2, 6, 8];
        let max_idx = 10usize;
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                match key.code {
                    KeyCode::Up => {
                        let mut i = menu_state.selected().unwrap_or(1);
                        loop {
                            i = i.saturating_sub(1);
                            if !SKIP.contains(&i) || i == 0 {
                                break;
                            }
                        }
                        menu_state.select(Some(i));
                    }
                    KeyCode::Down => {
                        let mut i = menu_state.selected().unwrap_or(0);
                        loop {
                            i = (i + 1).min(max_idx);
                            if !SKIP.contains(&i) || i == max_idx {
                                break;
                            }
                        }
                        menu_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        let idx = menu_state.selected().unwrap_or(0);
                        match idx {
                            0 => action_set_save_folder(terminal, &mut app)?,
                            1 => action_recover_bak(terminal, &mut app)?,
                            3 => action_set_backup_location(terminal, &mut app)?,
                            4 => action_create_backup(terminal, &mut app)?,
                            5 => action_restore_backup(terminal, &mut app)?,
                            7 => run_ini_submenu(terminal, &mut app)?,
                            9 => {
                                if let Some(false) = run_disclaimer(terminal, &mut app)? {
                                    return Ok(());
                                }
                            }
                            10 => return Ok(()), // Exit
                            _ => {}
                        }
                        app.clear_status();
                    }
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

// ── disclaimer ─────────────────────────────────────────────────────────────

/// Display the start-up disclaimer and prompt for acceptance.
fn run_disclaimer<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<Option<bool>> {
    let mut selected_yes = true;
    loop {
        terminal.draw(|f| {
            if f.area().width < 60 || f.area().height < 20 {
                draw_too_small(f);
                return;
            }
            tui::draw_disclaimer_popup(f, &app.tui_state, selected_yes);
        })?;
        if let Some(key) = poll_key(250)? {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                    selected_yes = !selected_yes
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    guard::log_action("LICENSE", "accepted", "OK", &app.log_path)?;
                    crate::config::accept_disclaimer()?;
                    return Ok(Some(true));
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    guard::log_action("LICENSE", "declined", "OK", &app.log_path)?;
                    return Ok(Some(false));
                }
                KeyCode::Esc => {
                    return Ok(None);
                }
                KeyCode::Enter => {
                    let accepted = selected_yes;
                    let detail = if accepted { "accepted" } else { "declined" };
                    guard::log_action("LICENSE", detail, "OK", &app.log_path)?;
                    if accepted {
                        crate::config::accept_disclaimer()?;
                    }
                    return Ok(Some(accepted));
                }
                _ => {}
            }
        }
    }
}

// ── menu actions ───────────────────────────────────────────────────────────

/// Open the input dialog for the user to type a save-folder path.
/// Validates the path exists and contains .sav files before accepting it.
fn action_set_save_folder<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let default = app.save_folder.as_ref().map(|p| p.to_string_lossy().to_string());
    let mut input_state =
        tui::InputDialogState::new("Enter the path to your Subnautica 2 SaveGames folder:");
    if let Some(d) = default {
        input_state.input = d;
        input_state.cursor = input_state.input.len();
    }
    let mut ok_selected = true;

    loop {
        terminal.draw(|f| {
            tui::draw_input_dialog(f, &app.tui_state, &input_state, ok_selected);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            match crossterm::event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }
                    match key.code {
                        KeyCode::Enter => {
                            if ok_selected && !input_state.input.is_empty() {
                                // Sanitize: strip control characters to prevent
                                // config.ini injection and log forgery.
                                let sanitized: String = input_state
                                    .input
                                    .chars()
                                    .filter(|c| !c.is_control())
                                    .collect();
                                let candidate = discovery::validate_custom_path(&sanitized);
                                if let Some(path) = candidate {
                                    app.save_folder = Some(path.clone());
                                    refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
                                    crate::config::save_app_config(
                                        Some(&sanitized),
                                        Some(&crate::config::get_backup_root().to_string_lossy()),
                                    );
                                    let msg = format!("Save folder set to {}", path.display());
                                    app.set_status(&msg, tui::StatusStyle::Success);
                                    input_state.confirmed = true;
                                    return Ok(());
                                } else {
                                    // Invalid path — show error and let them retry
                                    ok_dialog(
                                        terminal,
                                        app,
                                        "Invalid Path",
                                        "The path you entered does not exist or\n\
                                         does not contain any .sav save files.\n\
                                         \n\
                                         Please enter the full path to your\n\
                                         SaveGames folder (e.g.\n\
                                         /home/user/.../SaveGames).",
                                    )?;
                                    input_state.reset();
                                    ok_selected = true;
                                    continue;
                                }
                            }
                            // Cancel was selected
                            input_state.cancelled = true;
                            return Ok(());
                        }
                        KeyCode::Char(c) if ok_selected => {
                            input_state.insert(c);
                        }
                        KeyCode::Backspace if ok_selected => {
                            input_state.backspace();
                        }
                        KeyCode::Delete if ok_selected => {
                            input_state.delete();
                        }
                        KeyCode::Left if ok_selected => {
                            input_state.cursor_left();
                        }
                        KeyCode::Right if ok_selected => {
                            input_state.cursor_right();
                        }
                        KeyCode::Tab => {
                            ok_selected = !ok_selected;
                        }
                        KeyCode::Esc => {
                            input_state.cancelled = true;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                Event::Paste(s) if ok_selected => {
                    input_state.insert_str(&s);
                }
                _ => {}
            }
        }
    }
}

/// Open the input dialog for the user to set a custom backup location.
/// Pre-fills with the default `home_dir/NotAlterra` path for easy editing.
fn action_set_backup_location<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let default = dirs::home_dir()
        .map(|h| h.join("NotAlterra"))
        .unwrap_or_else(crate::config::exe_dir);
    let mut input_state = tui::InputDialogState::new("Enter the path for storing backup archives:");
    input_state.input = default.to_string_lossy().to_string();
    input_state.cursor = input_state.input.len();
    let mut ok_selected = true;

    loop {
        terminal.draw(|f| {
            tui::draw_input_dialog(f, &app.tui_state, &input_state, ok_selected);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            match crossterm::event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }
                    match key.code {
                        KeyCode::Enter => {
                            if ok_selected && !input_state.input.is_empty() {
                                let sanitized: String = input_state
                                    .input
                                    .chars()
                                    .filter(|c| !c.is_control())
                                    .collect();
                                let path = std::path::PathBuf::from(&sanitized);
                                crate::config::set_backup_root(path.clone());
                                let save_str = app
                                    .save_folder
                                    .as_ref()
                                    .map(|p| p.to_string_lossy().to_string());
                                crate::config::save_app_config(
                                    save_str.as_deref(),
                                    Some(&sanitized),
                                );
                                let msg = format!("Backup location set to {}", path.display());
                                app.set_status(&msg, tui::StatusStyle::Success);
                                input_state.confirmed = true;
                                return Ok(());
                            }
                            input_state.cancelled = true;
                            return Ok(());
                        }
                        KeyCode::Char(c) if ok_selected => {
                            input_state.insert(c);
                        }
                        KeyCode::Backspace if ok_selected => {
                            input_state.backspace();
                        }
                        KeyCode::Delete if ok_selected => {
                            input_state.delete();
                        }
                        KeyCode::Left if ok_selected => {
                            input_state.cursor_left();
                        }
                        KeyCode::Right if ok_selected => {
                            input_state.cursor_right();
                        }
                        KeyCode::Tab => {
                            ok_selected = !ok_selected;
                        }
                        KeyCode::Esc => {
                            input_state.cancelled = true;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                Event::Paste(s) if ok_selected => {
                    input_state.insert_str(&s);
                }
                _ => {}
            }
        }
    }
}

/// Recover a .sav from its .bak backup with a rollback safety net.
fn action_recover_bak<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let save_folder = ensure_save_folder(terminal, app)?;

    // Show spinner while parsing metadata
    app.set_status("Reading save metadata…", tui::StatusStyle::Info);
    app.set_spinner(true);
    terminal.draw(|f| {
        tui::draw_text_screen(
            f,
            &app.tui_state,
            &[Line::from(Span::styled(
                "Reading save file metadata…",
                Style::default().add_modifier(Modifier::BOLD),
            ))],
            "Parsing GVAS headers for slot names and display names…",
        );
    })?;

    let bak_summaries = ops::list_bak_files_with_meta(&save_folder);
    app.set_spinner(false);
    app.clear_status();

    if bak_summaries.is_empty() {
        app.set_status("No .bak files found.", tui::StatusStyle::Error);
        wait_for_key(terminal, app)?;
        return Ok(());
    }

    // Build multi-column display with slot grouping.
    // First entry in each slot gets a numbered label matching the savegame slot.
    let mut labelled: std::collections::HashSet<String> = std::collections::HashSet::new();
    let header = format!(" {:<8}  {:<24}  {}", "Slot", "Description", "Date");
    let mut items: Vec<String> = vec![header, String::new()];
    items.extend(
        bak_summaries
            .iter()
            .map(|s| {
                let num = slot_number(&s.slot);
                let first = labelled.insert(s.slot.clone());
                let label_col = if first {
                    format!("Slot {num}")
                } else {
                    String::new()
                };
                let name = s.display_name.as_deref().unwrap_or("(unnamed)");
                let name_col = if name.len() > 24 {
                    format!("{}…", &name[..23])
                } else {
                    name.to_string()
                };
                let date = s.mtime.as_deref().unwrap_or("?");
                format!(" {:<8}  {:<24}  {}", label_col, name_col, date,)
            })
            .collect::<Vec<String>>(),
    );
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

    // Filenames for the info bar, and descriptions
    let filenames: Vec<String> = bak_summaries.iter().map(|s| s.filename.clone()).collect();
    let descs: Vec<String> = bak_summaries
        .iter()
        .map(|_| "Restore this backup to its canonical .sav file".to_string())
        .collect();
    let desc_refs: Vec<&str> = descs.iter().map(|s| s.as_str()).collect();
    let mut state = ListState::default().with_selected(Some(2)); // skip header + blank

    // Lazily-loaded full metadata cache — loaded on first highlight
    let mut full_metas: Vec<Option<crate::gvas::FullMetadata>> = vec![None; bak_summaries.len()];

    /// Load metadata for index `idx` if not already cached.
    fn ensure_meta(
        idx: usize,
        bak: &[ops::BakFileSummary],
        cache: &mut [Option<crate::gvas::FullMetadata>],
    ) {
        if idx < cache.len() && cache[idx].is_none() {
            cache[idx] = crate::gvas::extract_full_metadata(&bak[idx].path).ok();
        }
    }

    /// Build right-pane display lines from cached metadata.
    fn build_meta_lines<'a>(
        meta: Option<&'a crate::gvas::FullMetadata>,
        summary: &ops::BakFileSummary,
    ) -> Vec<Line<'a>> {
        let dim = Style::default().fg(Color::Rgb(160, 160, 160));

        let Some(m) = meta else {
            return Vec::new();
        };

        let pt = m.playtime_seconds.or(summary.playtime_seconds);
        let playtime = format_playtime(pt);
        let fields: Vec<(&str, String)> = vec![
            (
                "Slot",
                m.slot_name.as_deref().unwrap_or(&summary.slot).to_string(),
            ),
            (
                "Name",
                m.display_name.as_deref().unwrap_or("(unnamed)").to_string(),
            ),
            ("Playtime", playtime),
            (
                "Game Type",
                m.game_mode.as_deref().unwrap_or("?").to_string(),
            ),
            (
                "Mode",
                if m.is_online {
                    "Multiplayer".into()
                } else {
                    "Single Player".into()
                },
            ),
            (
                "Was Multi",
                if m.was_multiplayer {
                    "Yes".into()
                } else {
                    "No".into()
                },
            ),
            (
                "Branch",
                m.build_branch.as_deref().unwrap_or("?").to_string(),
            ),
            (
                "Build",
                m.build_number.map_or("?".into(), |n| n.to_string()),
            ),
        ];
        let max_label: usize = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(6);
        fields
            .into_iter()
            .map(|(k, v)| {
                let padded_label = format!("{:<max_label$}", k);
                Line::from(vec![
                    Span::styled(padded_label, Style::default().fg(Color::White)),
                    Span::raw("  "),
                    Span::styled(v, dim),
                ])
            })
            .collect()
    }

    loop {
        let i = state.selected().unwrap_or(2).max(2);
        state.select(Some(i));
        let sel_idx = i.saturating_sub(2);
        let selected_info = filenames.get(sel_idx).map(|s| s.as_str());
        let meta_header = selected_info.map(|f| format!("Details for {}", f));

        // Lazy-load metadata on highlight
        if sel_idx < bak_summaries.len() {
            ensure_meta(sel_idx, &bak_summaries, &mut full_metas);
        }
        let meta = full_metas.get(sel_idx).and_then(|m| m.as_ref());
        let meta_lines = if let Some(m) = meta {
            build_meta_lines(Some(m), &bak_summaries[sel_idx])
        } else {
            Vec::new()
        };

        // Recover picker: browsing .bak files in the live save folder → show save path
        app.tui_state.context_path = None;
        terminal.draw(|f| {
            tui::draw_picker_split(
                f,
                &app.tui_state,
                &item_refs,
                &desc_refs,
                &mut state,
                meta_header.as_deref(),
                &meta_lines,
            );
        })?;
        if let Some(key) = poll_key(250)? {
            match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(items.len().saturating_sub(1))));
                }
                KeyCode::Enter => {
                    let sel = state.selected().unwrap_or(2);
                    if sel < 2 {
                        continue;
                    } // header + blank
                    let idx = sel.saturating_sub(2);
                    let chosen = &bak_summaries[idx];
                    let target = derive_target_sav(&chosen.filename);
                    let target_path = save_folder.join(&target);

                    // Target .sav metadata
                    let target_meta = if target_path.exists() {
                        std::fs::metadata(&target_path).ok().map(|m| {
                            let sz = m.len();
                            let mt = m.modified().ok().and_then(|t| {
                                let s = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                                chrono::Local.timestamp_opt(s as i64, 0).single()
                            });
                            let meta = crate::gvas::extract_metadata(&target_path).ok();
                            let online = meta.as_ref().map(|m| m.is_online).unwrap_or(false);
                            let disp = meta.and_then(|m| m.display_name);
                            (sz, mt, online, disp)
                        })
                    } else {
                        None
                    };

                    // Mode change warning
                    let mode_entry = target_meta.as_ref().and_then(|(_, _, live, _)| {
                        if *live != chosen.is_online {
                            let from = if *live {
                                "Multiplayer"
                            } else {
                                "Single Player"
                            };
                            let to = if chosen.is_online {
                                "Multiplayer"
                            } else {
                                "Single Player"
                            };
                            Some(("⚠  Mode change", format!("{from} → {to}")))
                        } else {
                            None
                        }
                    });
                    // Name change warning
                    let name_entry = target_meta.as_ref().and_then(|(_, _, _, live_name)| {
                        let bak_name = chosen.display_name.as_deref().unwrap_or("(unnamed)");
                        match live_name {
                            Some(live) if live != bak_name => {
                                Some(("⚠  Name change", format!("{live} → {bak_name}")))
                            }
                            _ => None,
                        }
                    });

                    // Build details
                    let src_line = format!(
                        "{}  {}  {}",
                        chosen.filename,
                        format_size(chosen.size),
                        chosen.mtime.as_deref().unwrap_or("?")
                    );
                    let mut details = vec![
                        ("Slot", chosen.slot.as_str()),
                        (
                            "Name",
                            chosen.display_name.as_deref().unwrap_or("(unnamed)"),
                        ),
                        ("Backup", src_line.as_str()),
                    ];
                    let tgt_line: String;
                    if let Some((sz, mt, _, _)) = &target_meta {
                        tgt_line = format!(
                            "{}  {}  {}",
                            target,
                            format_size(*sz),
                            mt.map(|d| d.format("%Y-%b-%d %H:%M").to_string())
                                .as_deref()
                                .unwrap_or("?")
                        );
                        details.push(("Replace", tgt_line.as_str()));
                    } else {
                        details.push(("Create", target.as_str()));
                    }
                    if let Some((k, v)) = &mode_entry {
                        details.push((&**k, &**v));
                    }
                    if let Some((k, v)) = &name_entry {
                        details.push((&**k, &**v));
                    }
                    if !has_existing_backup(app) {
                        details.push(("⚠  No backup", "create a full backup first"));
                    }
                    let accepted = confirm_modal(terminal, app, "Confirm Recovery", &details)?;

                    if accepted {
                        match ops::recover_bak_to_sav(&save_folder, &chosen.filename) {
                            Ok(result) => {
                                let msg = format!("Restored {} → {}", result.source, result.target);
                                app.set_status(&msg, tui::StatusStyle::Success);
                                guard::log_action(
                                    "RECOVER",
                                    &format!("{} → {}", result.source, result.target),
                                    "OK",
                                    &app.log_path,
                                )?;
                            }
                            Err(e) => {
                                app.set_status(&format!("Failed: {e}"), tui::StatusStyle::Error);
                                guard::log_action(
                                    "RECOVER",
                                    &chosen.filename,
                                    &format!("FAILED: {e}"),
                                    &app.log_path,
                                )?;
                            }
                        }
                    }
                    break;
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    refresh_stats(&mut app.tui_state, Some(&save_folder));
    Ok(())
}

/// Create a full backup of all savegame files into a timestamped folder.
fn action_create_backup<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let save_folder = ensure_save_folder(terminal, app)?;

    info_dialog(
        terminal,
        app,
        "Creating Backup",
        "NotAlterra is backing up your save files.",
    )?;

    app.set_status("Creating backup…", tui::StatusStyle::Info);
    app.set_spinner(true);

    match ops::create_full_backup(&save_folder) {
        Ok(result) => {
            app.set_spinner(false);
            let verified = if result.verified {
                "verified"
            } else {
                "unverified"
            };
            let msg = format!(
                "{} files, {} — backup {}",
                result.files_copied,
                format_size(result.total_size),
                verified,
            );
            guard::log_action("MANUAL_BAK", &msg, "OK", &app.log_path)?;
            ok_dialog(terminal, app, "Backup Complete", &msg)?;
        }
        Err(e) => {
            app.set_spinner(false);
            let msg = format!("Backup failed: {e}");
            guard::log_action(
                "MANUAL_BAK",
                &guard::sanitize_path(&save_folder.display().to_string()),
                &format!("FAILED: {e}"),
                &app.log_path,
            )?;
            ok_dialog(terminal, app, "Backup Failed", &msg)?;
        }
    }
    Ok(())
}

/// Restore a previously created full backup, overwriting the save folder.
fn action_restore_backup<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let save_folder = ensure_save_folder(terminal, app)?;
    let backups = ops::list_full_backups();
    if backups.is_empty() {
        ok_dialog(terminal, app, "No Backups", "No full backups found in NotAlterra_Backups.\nUse 'Create full backup' from the main menu first.")?;
        return Ok(());
    }

    let header = format!("    {:<38}      {:>8}", "Backup", "Size");
    let mut items: Vec<String> = vec![header, String::new()];
    items.extend(
        backups
            .iter()
            .map(|p| {
                let name = p.file_name().unwrap().to_string_lossy();
                let size = std::fs::metadata(p)
                    .map(|m| format_size(m.len()))
                    .unwrap_or("?".into());
                format!("  {:<42}  {:>8}", format_backup_label(&name), size)
            })
            .collect::<Vec<String>>(),
    );
    // Prepend empty description entries for header + blank
    let descs: Vec<String> = std::iter::once(String::new())
        .chain(std::iter::once(String::new()))
        .chain(backups.iter().map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if !ops::check_tar_gz_integrity(p) {
                "⚠  Corrupted — file does not appear to be a valid backup archive".into()
            } else if name.contains("pre_restore") {
                "Pre-restore snapshot — safety copy made before a restore".into()
            } else if name.contains("migrated") {
                "Migrated from old backup format — treated as a full backup".into()
            } else {
                "Restore save folder from this full backup".into()
            }
        }))
        .collect();
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let desc_refs: Vec<&str> = descs.iter().map(|s| s.as_str()).collect();
    let mut state = ListState::default().with_selected(Some(2)); // skip header + blank

    loop {
        // Restore picker: show backup root in header
        app.tui_state.context_path = Some(
            crate::config::get_backup_root()
                .to_string_lossy()
                .to_string(),
        );
        terminal.draw(|f| {
            tui::draw_picker(f, &app.tui_state, &item_refs, &desc_refs, &mut state, true);
        })?;
        if let Some(key) = poll_key(250)? {
            match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(items.len().saturating_sub(1))));
                }
                KeyCode::Enter => {
                    let idx = state.selected().unwrap_or(2).max(2).saturating_sub(2);
                    if idx >= backups.len() {
                        continue;
                    }
                    let chosen = &backups[idx];
                    let name = chosen.file_name().unwrap().to_string_lossy().to_string();

                    // Block restore on corrupt archives
                    if !ops::check_tar_gz_integrity(chosen) {
                        ok_dialog(
                            terminal,
                            app,
                            "Corrupt Backup",
                            "This backup archive appears to be corrupt or\n\
                             incomplete and cannot be restored.\n\
                             \n\
                             Create a new full backup to replace it.",
                        )?;
                        continue;
                    }

                    let mut restore_details = vec![("Backup", name.as_str())];
                    if !has_existing_backup(app) {
                        restore_details.push(("⚠ No backup", "create a full backup first"));
                    }
                    let accepted =
                        confirm_modal(terminal, app, "Confirm Restore", &restore_details)?;

                    if accepted {
                        guard::log_action(
                            "AUTO_BAK",
                            &format!(
                                "pre-restore → {}",
                                guard::sanitize_path(&save_folder.display().to_string())
                            ),
                            "OK",
                            &app.log_path,
                        )?;
                        match ops::restore_full_backup(chosen, &save_folder) {
                            Ok(n) => {
                                app.set_status(
                                    &format!("{n} save files restored."),
                                    tui::StatusStyle::Success,
                                );
                                guard::log_action(
                                    "RESTORE",
                                    &format!(
                                        "{} → {}",
                                        name,
                                        guard::sanitize_path(&save_folder.display().to_string())
                                    ),
                                    "OK",
                                    &app.log_path,
                                )?;
                            }
                            Err(e) => {
                                app.set_status(
                                    &format!("Restore failed: {e}"),
                                    tui::StatusStyle::Error,
                                );
                                guard::log_action(
                                    "RESTORE",
                                    &name,
                                    &format!("FAILED: {e}"),
                                    &app.log_path,
                                )?;
                            }
                        }
                    }
                    break;
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    refresh_stats(&mut app.tui_state, Some(&save_folder));
    Ok(())
}

// ── .ini submenu ───────────────────────────────────────────────────────────

/// Display the .ini management submenu with Backup, Restore, and Delete options.
fn run_ini_submenu<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let ini_path = get_ini_path(terminal, app)?;

    let items: Vec<&str> = vec![
        " Backup .ini files",
        " Restore .ini files from backup",
        " Delete .ini files (requires backup)",
        "",
        " Back",
    ];
    let descs: Vec<&str> = vec![
        "Copy all .ini files from Config/Windows to NotAlterra_Backups",
        "Restore .ini files from a previous backup",
        "Remove .ini files — game regenerates defaults (backup required first)",
        "",
        "Return to main menu",
    ];
    let mut state = ListState::default().with_selected(Some(0));
    const INI_SKIP: &[usize] = &[3];
    let ini_max = 4usize;

    let ini_path_str = app
        .save_folder
        .as_ref()
        .and_then(|sf| discovery::derive_ini_path(sf).map(|p| p.to_string_lossy().to_string()));
    loop {
        // Ini submenu: show path based on highlighted item
        {
            let sel = state.selected().unwrap_or(0);
            let br = crate::config::get_backup_root()
                .to_string_lossy()
                .to_string();
            app.tui_state.context_path = match sel {
                0 | 1 => Some(br),         // backup root (backup/restore)
                2 => ini_path_str.clone(), // Config\Windows path (delete)
                _ => Some(String::new()),  // blank / back → nothing
            };
        }
        terminal.draw(|f| {
            tui::draw_sub_menu(
                f,
                &app.tui_state,
                "Config (.ini) Management",
                &items,
                &descs,
                &mut state,
            );
        })?;
        if let Some(key) = poll_key(250)? {
            match key.code {
                KeyCode::Up => {
                    let mut i = state.selected().unwrap_or(1);
                    loop {
                        i = i.saturating_sub(1);
                        if !INI_SKIP.contains(&i) || i == 0 {
                            break;
                        }
                    }
                    state.select(Some(i));
                }
                KeyCode::Down => {
                    let mut i = state.selected().unwrap_or(0);
                    loop {
                        i = (i + 1).min(ini_max);
                        if !INI_SKIP.contains(&i) || i == ini_max {
                            break;
                        }
                    }
                    state.select(Some(i));
                }
                KeyCode::Enter => {
                    let idx = state.selected().unwrap_or(0);
                    match idx {
                        0 => ini_backup_action(terminal, app, &ini_path)?,
                        1 => ini_restore_action(terminal, app, &ini_path)?,
                        2 => ini_delete_action(terminal, app, &ini_path)?,
                        4 => break,
                        _ => {}
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
    Ok(())
}

/// Back up all `.ini` files from the Config\Windows folder into a timestamped archive.
fn ini_backup_action<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ini_path: &Path,
) -> Result<()> {
    info_dialog(
        terminal,
        app,
        "Creating .ini Backup",
        "NotAlterra is backing up your\nUE5 Config (.ini) files.",
    )?;

    match ops::backup_ini_files(ini_path) {
        Ok(result) => {
            let verified = if result.verified {
                "verified"
            } else {
                "unverified"
            };
            app.set_status(
                &format!(
                    "Config backup created: {} files ({})",
                    result.files_copied, verified
                ),
                tui::StatusStyle::Success,
            );
            guard::log_action(
                "CONFIG_BAK",
                &guard::sanitize_path(&result.dest_path.display().to_string()),
                "OK",
                &app.log_path,
            )?;
            refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
            let verified = if result.verified {
                "verified"
            } else {
                "unverified"
            };
            let msg = format!(
                "{} .ini file(s) backed up ({verified}).",
                result.files_copied
            );
            ok_dialog(terminal, app, ".ini Backup Complete", &msg)?;
        }
        Err(e) => {
            ok_dialog(terminal, app, ".ini Backup Failed", &format!("{e}"))?;
        }
    }

    Ok(())
}

/// Restore `.ini` files from a selected backup into the Config\Windows folder.
/// Creates a pre-restore safety copy of the current `.ini` files first.
fn ini_restore_action<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ini_path: &Path,
) -> Result<()> {
    let backups = ops::list_ini_backups();
    if backups.is_empty() {
        ok_dialog(
            terminal,
            app,
            "No .ini Backups",
            "No .ini backups found.\nUse 'Backup .ini files' first.",
        )?;
        return Ok(());
    }

    let header = format!("    {:<38}      {:>8}", "INI Backup", "Size");
    let mut items: Vec<String> = vec![header, String::new()];
    items.extend(backups.iter().map(|p| {
        let name = p.file_name().unwrap().to_string_lossy();
        let size = std::fs::metadata(p)
            .map(|m| format_size(m.len()))
            .unwrap_or("?".into());
        format!("  {:<42}  {:>8}", format_backup_label(&name), size)
    }));
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let ini_descs: Vec<&str> = std::iter::once("")
        .chain(std::iter::once(""))
        .chain(std::iter::repeat_n(
            "Restore .ini files from this backup",
            backups.len(),
        ))
        .collect::<Vec<_>>();
    let mut state = ListState::default().with_selected(Some(2)); // skip header + blank

    loop {
        // Restore picker: show backup root in header
        app.tui_state.context_path = Some(
            crate::config::get_backup_root()
                .to_string_lossy()
                .to_string(),
        );
        terminal.draw(|f| {
            tui::draw_picker(f, &app.tui_state, &item_refs, &ini_descs, &mut state, true);
        })?;
        if let Some(key) = poll_key(250)? {
            match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(items.len().saturating_sub(1))));
                }
                KeyCode::Enter => {
                    let idx = state.selected().unwrap_or(0);
                    let chosen = &backups[idx];

                    guard::log_action(
                        "AUTO_BAK",
                        &format!(
                            "ini pre-restore → {}",
                            guard::sanitize_path(&ini_path.display().to_string())
                        ),
                        "OK",
                        &app.log_path,
                    )?;
                    match ops::restore_ini_files(chosen, ini_path) {
                        Ok(n) => {
                            guard::log_action(
                                "CONFIG_RESTORE",
                                &guard::sanitize_path(&chosen.display().to_string()),
                                "OK",
                                &app.log_path,
                            )?;
                            let msg = format!("{n} .ini file(s) restored.");
                            ok_dialog(terminal, app, ".ini Restore Complete", &msg)?;
                        }
                        Err(e) => {
                            ok_dialog(terminal, app, ".ini Restore Failed", &format!("{e}"))?;
                        }
                    }
                    break;
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    Ok(())
}

/// Delete all `.ini` files from the Config\Windows folder.
/// Refuses to proceed unless at least one `.ini` backup exists in the backup root.
fn ini_delete_action<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ini_path: &Path,
) -> Result<()> {
    let config_dir = crate::config::backups_config_dir();
    let has_backup = config_dir.exists()
        && std::fs::read_dir(&config_dir)
            .map(|entries| {
                entries.flatten().any(|e| {
                    let file_name = e.file_name();
                    let name = file_name.to_string_lossy();
                    name.starts_with("ini_backup_")
                        && e.path().is_dir()
                        && std::fs::read_dir(e.path()).is_ok_and(|mut d| {
                            d.any(|f| {
                                f.ok().is_some_and(|f| {
                                    f.file_name().to_string_lossy().ends_with(".ini")
                                })
                            })
                        })
                })
            })
            .unwrap_or(false);

    if !has_backup {
        ok_dialog(
            terminal,
            app,
            "No Backup Found",
            "No .ini backup directory found.\n\
             \n\
             Run \"Backup .ini files\" first to create a snapshot\n\
             before deleting the live .ini files.",
        )?;
        return Ok(());
    }

    match ops::delete_ini_files(ini_path) {
        Ok(n) => {
            let msg = format!(
                "Deleted {n} .ini file(s).\nThe game will regenerate defaults on next launch."
            );
            guard::log_action(
                "CONFIG_DEL",
                &guard::sanitize_path(&ini_path.display().to_string()),
                "OK",
                &app.log_path,
            )?;
            ok_dialog(terminal, app, ".ini Delete Complete", &msg)?;
        }
        Err(e) => {
            ok_dialog(terminal, app, ".ini Delete Refused", &format!("{e}"))?;
        }
    }
    Ok(())
}

// ── inspect saves ──────────────────────────────────────────────────────────

/// Inspect GVAS metadata for any .sav or .bak file from a file picker.
fn action_inspect_saves<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let save_folder = ensure_save_folder(terminal, app)?;
    let mut files: Vec<_> = std::fs::read_dir(&save_folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            let n = e.file_name();
            let s = n.to_string_lossy();
            s.starts_with("savegame_") && (s.ends_with(".sav") || s.ends_with(".bak"))
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    let mut labelled = std::collections::HashSet::new();
    let mut items: Vec<String> = files
        .iter()
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let slot = crate::gvas::derive_slot_from_filename(&name).unwrap_or_else(|| "?".into());
            let num = slot_number(&slot);
            let first = labelled.insert(slot.clone());
            let label = if first {
                format!("Slot {num}")
            } else {
                String::new()
            };
            let sz = e.metadata().map(|m| m.len()).unwrap_or(0);
            format!("  {:<8}  {:<28}  {:>7}", label, name, format_size(sz))
        })
        .collect();
    let header = format!("  {:<8}  {:<28}  {:>7}", "Slot", "Filename", "Size");
    items.insert(0, header);
    items.insert(1, String::new());
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let filenames: Vec<String> = files
        .iter()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    let descs = vec!["Press Enter to view full GVAS metadata"; files.len()];
    let desc_refs: Vec<&str> = descs.to_vec();
    let mut state = ListState::default().with_selected(Some(2)); // skip header + blank

    loop {
        let i = state.selected().unwrap_or(2).max(2);
        state.select(Some(i));
        terminal.draw(|f| {
            tui::draw_picker_with_info(
                f,
                &app.tui_state,
                &item_refs,
                &desc_refs,
                &mut state,
                false,
            );
        })?;
        if let Some(key) = poll_key(250)? {
            match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(items.len().saturating_sub(1))));
                }
                KeyCode::Enter => {
                    let sel = state.selected().unwrap_or(2);
                    if sel < 2 {
                        continue;
                    }
                    let idx = sel.saturating_sub(2);
                    let path = files[idx].path();
                    match crate::gvas::extract_full_metadata(&path) {
                        Ok(meta) => {
                            let dim = Style::default().fg(Color::DarkGray);
                            let val = Style::default().fg(Color::White);
                            let hl = Style::default().fg(Color::Cyan);
                            let mut lines: Vec<Line> = vec![
                                Line::from(Span::styled(
                                    filenames[idx].clone(),
                                    hl.add_modifier(Modifier::BOLD),
                                )),
                                Line::from(""),
                            ];
                            let fields: Vec<(&str, &str, String)> = vec![
                                (
                                    "SlotName",
                                    ":",
                                    meta.slot_name.as_deref().unwrap_or("?").into(),
                                ),
                                (
                                    "DisplayName",
                                    ":",
                                    meta.display_name.as_deref().unwrap_or("(unnamed)").into(),
                                ),
                                (
                                    "Game Type",
                                    ":",
                                    (if meta.is_online {
                                        "Multiplayer"
                                    } else {
                                        "Single Player"
                                    })
                                    .into(),
                                ),
                                (
                                    "Was Multi",
                                    ":",
                                    (if meta.was_multiplayer { "yes" } else { "no" }).into(),
                                ),
                                (
                                    "GameMode",
                                    ":",
                                    meta.game_mode.as_deref().unwrap_or("?").into(),
                                ),
                                (
                                    "Level",
                                    ":",
                                    meta.level_name.as_deref().unwrap_or("?").into(),
                                ),
                                (
                                    "Build",
                                    ":",
                                    meta.build_number.map_or("?".into(), |n| n.to_string()),
                                ),
                                (
                                    "Branch",
                                    ":",
                                    meta.build_branch.as_deref().unwrap_or("?").into(),
                                ),
                                (
                                    "Saves",
                                    ":",
                                    meta.saves_count.map_or("?".into(), |n| n.to_string()),
                                ),
                                (
                                    "Ver",
                                    ":",
                                    meta.latest_version.map_or("?".into(), |n| n.to_string()),
                                ),
                                (
                                    "DataVer",
                                    ":",
                                    meta.data_version.map_or("?".into(), |n| n.to_string()),
                                ),
                                ("Playtime", ":", format_playtime(meta.playtime_seconds)),
                            ];
                            let max_label: usize =
                                fields.iter().map(|(k, _, _)| k.len()).max().unwrap_or(8);
                            for (key, sep, value) in fields {
                                let padded = format!("{:<max_label$}{sep} ", key);
                                lines.push(Line::from(vec![
                                    Span::styled(padded, dim),
                                    Span::styled(value, val),
                                ]));
                            }
                            ok_dialog_styled(terminal, app, "GVAS Metadata", &lines)?;
                        }
                        Err(e) => {
                            ok_dialog(terminal, app, "Parse Error", &format!("{e}"))?;
                        }
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }
    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────────

/// Poll for a single key event with a timeout in milliseconds.
/// Returns `None` if no key is pressed within the timeout.  Filters out
/// `KeyEventKind::Release` events to avoid double-firing on held keys.
fn poll_key(timeout_ms: u64) -> Result<Option<crossterm::event::KeyEvent>> {
    if event::poll(std::time::Duration::from_millis(timeout_ms))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release {
                return Ok(Some(key));
            }
        }
    }
    Ok(None)
}

/// Ensure we have a cached save folder path that exists.
fn ensure_save_folder<B: Backend>(_terminal: &mut Terminal<B>, app: &mut App) -> Result<PathBuf> {
    if let Some(ref sf) = app.save_folder {
        if sf.exists() {
            return Ok(sf.clone());
        }
    }
    anyhow::bail!("No save folder set. Use 'Set save folder' from the main menu first.")
}

/// Derive the Config\Windows path from the save folder.
fn get_ini_path<B: Backend>(_terminal: &mut Terminal<B>, app: &mut App) -> Result<PathBuf> {
    if let Some(ref sf) = app.save_folder {
        if let Some(cp) = discovery::derive_ini_path(sf) {
            return Ok(cp);
        }
    }
    anyhow::bail!(
        "Cannot determine Config/Windows path. Set your save folder first via 'Set save folder'."
    )
}

/// Extract the numeric suffix from a slot name like "savegame_3".
fn slot_number(slot: &str) -> String {
    slot.strip_prefix("savegame_")
        .map(|s| s.to_string())
        .unwrap_or_else(|| slot.to_string())
}

/// Display a dialog with styled content lines (colors, bold).  Accepts
/// `Line` slices — use for metadata displays, help text, or any content
/// that needs inline formatting.  Press Enter or Space to dismiss.
fn ok_dialog_styled<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
    title: &str,
    lines: &[Line],
) -> Result<()> {
    loop {
        terminal.draw(|f| tui::draw_ok_dialog_styled(f, &app.tui_state, title, lines))?;
        if let Some(key) = poll_key(250)? {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                return Ok(());
            }
        }
    }
}

/// Display a non-interactive info dialog — no buttons, renders once.
/// The dialog stays on screen until the next `terminal.draw()` call
/// replaces it.  Use for brief status messages before a blocking operation.
fn info_dialog<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
    title: &str,
    msg: &str,
) -> Result<()> {
    terminal.draw(|f| tui::draw_info_dialog(f, &app.tui_state, title, msg))?;
    Ok(())
}

/// Display a plain-text informational dialog with a single OK button.
/// `msg` supports newlines for multi-line messages.  Press Enter or
/// Space to dismiss.  For styled content, use `ok_dialog_styled`.
fn ok_dialog<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
    title: &str,
    msg: &str,
) -> Result<()> {
    loop {
        terminal.draw(|f| tui::draw_ok_dialog(f, &app.tui_state, title, msg))?;
        if let Some(key) = poll_key(250)? {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                return Ok(());
            }
        }
    }
}

/// Check whether at least one backup directory exists in the backup root.
/// Used to gate destructive operations behind a backup requirement.
fn has_existing_backup(_app: &App) -> bool {
    let root = crate::config::backups_saves_dir();
    root.exists()
        && std::fs::read_dir(&root).is_ok_and(|mut d| {
            d.any(|e| e.is_ok_and(|e| e.file_name().to_string_lossy().ends_with(".tar.gz")))
        })
}

/// Gate: warn the user if no full backup exists yet.  Returns `false` if
/// no backup is found (caller should abort the destructive operation).
fn require_backup<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    if has_existing_backup(app) {
        return Ok(true);
    }
    app.set_status(
        "No backup found — create a full backup before destructive actions.",
        tui::StatusStyle::Error,
    );
    wait_for_key(terminal, app)?;
    Ok(false)
}

/// Derive the canonical `.sav` target filename from a `.bak` filename.
/// E.g. `savegame_0_9.bak` → `savegame_0.sav`.  Falls back to replacing
/// `.bak` with `.sav` if the slot pattern is not recognized.
fn derive_target_sav(bak_name: &str) -> String {
    crate::gvas::derive_slot_from_filename(bak_name)
        .map(|s| format!("{s}.sav"))
        .unwrap_or_else(|| bak_name.replace(".bak", ".sav"))
}

/// Show a confirmation popup with [ Yes ] [ No ] buttons.
fn confirm_modal<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    title: &str,
    details: &[(&str, &str)],
) -> Result<bool> {
    let mut selected_yes = true;
    loop {
        terminal.draw(|f| {
            if f.area().width < 60 || f.area().height < 15 {
                draw_too_small(f);
                return;
            }
            tui::draw_confirm_popup(f, &app.tui_state, title, details, selected_yes);
        })?;
        if let Some(key) = poll_key(250)? {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                    selected_yes = !selected_yes;
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                KeyCode::Enter => return Ok(selected_yes),
                _ => {}
            }
        }
    }
}

/// Format a backup archive filename into a human-readable label.
/// `snapshot_2026-06-09_125430_001.tar.gz` → `Full Backup — 2026-Jun-09 12:54`
fn format_backup_label(filename: &str) -> String {
    let stripped = filename.strip_suffix(".tar.gz").unwrap_or(filename);
    let (prefix, rest) = match stripped.split_once('_') {
        Some((p, r)) => (p, r),
        None => return filename.to_string(),
    };
    let label = match prefix {
        "snapshot" => "Full Backup",
        "pre" if stripped.contains("pre_restore") => "Pre-restore",
        "ini" => "INI Backup",
        "migrated" => "Migrated",
        _ => prefix,
    };
    // rest looks like: "2026-06-09_125430_001" or "notalterra_copy_..."
    // Try to parse the timestamp portion (first YYYY-MM-DD_HHMMSS segment)
    let date_str = if let Some(pos) = rest.find(|c: char| c.is_ascii_digit()) {
        let slice = &rest[pos..];
        if slice.len() >= 17 {
            let date_part = &slice[..10]; // 2026-06-09
            let time_part = &slice[11..17]; // 125430
            let h = &time_part[..2];
            let m = &time_part[2..4];
            format!("{date_part} {h}:{m}")
        } else {
            rest.to_string()
        }
    } else {
        rest.to_string()
    };
    format!("{label} — {date_str}")
}

/// Create a rectangle centered in the parent area by percentage.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Wait for any key press, showing the current state.
fn wait_for_key<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let prompt_span = Span::styled(
                "Press any key to return to menu…",
                Style::default().fg(Color::DarkGray),
            );
            let p = Paragraph::new(prompt_span).alignment(Alignment::Center);
            f.render_widget(p, centered_bottom(f.area()));
            // Whale at bottom
            let bar = Rect {
                x: 0,
                y: f.area().height.saturating_sub(1),
                width: f.area().width,
                height: 1,
            };
            tui::draw_whale_separator(f, bar, &app.tui_state);
        })?;
        if let Event::Key(_) = event::read()? {
            break;
        }
    }
    Ok(())
}
/// Create a rectangle anchored to the bottom of the parent area.
fn centered_bottom(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    }
}

/// Draw "terminal too small" message.
fn draw_too_small(f: &mut Frame) {
    let msg = "Terminal too small (min 60×15)";
    let p = Paragraph::new(Span::styled(
        msg,
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));
    let area = centered_rect(50, 20, f.area());
    f.render_widget(p, area);
}

/// Read file metadata.
fn fs_meta(path: &Path) -> Result<std::fs::Metadata, ()> {
    std::fs::metadata(path).map_err(|_| ())
}

/// Format a playtime value in seconds to a human-readable string
/// (`HHh MMm` or `MMm`, zero-padded to 2 digits).  Returns `—` for
/// `None` or sub-minute values.
fn format_playtime(seconds: Option<f64>) -> String {
    match seconds {
        Some(s) if s >= 3600.0 => {
            let h = (s / 3600.0) as u32;
            let m = ((s % 3600.0) / 60.0) as u32;
            format!("{h:02}h {m:02}m")
        }
        Some(s) if s >= 60.0 => {
            let m = (s / 60.0) as u32;
            format!("    {m:02}m")
        }
        Some(_) => String::from("—"),
        None => String::from("—"),
    }
}

/// Format a byte size human-readably.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
