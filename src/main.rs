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
use config::AppConfig;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
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

fn main() -> Result<()> {
    for arg in std::env::args().skip(1) {
        if arg == "--version" || arg == "-v" {
            println!("notalterra {}", VERSION);
            return Ok(());
        }
    }

    // ── setup terminal ─────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // ── teardown ───────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

// ── app state ──────────────────────────────────────────────────────────────

struct App {
    config: AppConfig,
    config_path: PathBuf,
    log_path: PathBuf,
    save_folder: Option<PathBuf>,
    tui_state: tui::AppState,
}

impl App {
    fn new() -> Result<Self> {
        let config_path = crate::config::config_path();
        let config = crate::config::load_config(&config_path)?;
        let log_path = guard::log_path();

        let save_folder = config.last_path.as_deref().map(PathBuf::from);

        let mut tui_state = tui::AppState::default();
        tui_state.version = VERSION.to_string();
        tui_state.save_path = save_folder.as_ref().map(|p| p.display().to_string());

        // Refresh stats for the dashboard
        refresh_stats(&mut tui_state, save_folder.as_deref());

        Ok(Self {
            config,
            config_path,
            log_path,
            save_folder,
            tui_state,
        })
    }

    fn backup_root(&self) -> PathBuf {
        exe_dir().join("NotAlterra_Backups")
    }

    fn set_status(&mut self, msg: &str, style: tui::StatusStyle) {
        self.tui_state.status_message = Some(msg.to_string());
        self.tui_state.status_style = style;
    }

    fn clear_status(&mut self) {
        self.tui_state.status_message = None;
    }

    fn set_spinner(&mut self, active: bool) {
        self.tui_state.spinner_active = active;
        if active {
            self.tui_state.spinner_start = Some(std::time::Instant::now());
        }
    }
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn refresh_stats(tui_state: &mut tui::AppState, save_folder: Option<&Path>) {
    tui_state.save_path = save_folder.map(|p| p.display().to_string());
    let backup_root = exe_dir().join("NotAlterra_Backups");
    let (live, bak, ini) = ops::folder_stats(save_folder, &backup_root);
    tui_state.live_save_count = live;
    tui_state.backup_count = bak;
    tui_state.has_ini_backup = ini;
}

// ── main loop ──────────────────────────────────────────────────────────────

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new()?;

    // Guard: exit if game is running
    if guard::game_running() {
        println!("NotAlterra has detected that Subnautica 2 is currently running.\n");
println!("The game holds file locks on your save files while active.");
println!("Backing up or restoring saves while the game is running can result in");
println!("incomplete, corrupt, or overwritten save files.\n");
println!("To protect your save files, NotAlterra will now exit.\n");
println!(" → Close Subnautica 2, then relaunch NotAlterra.");
        return Ok(());
    }

    // Disclaimer flow
    if !app.config.disclaimer_accepted {
        match run_disclaimer(terminal, &mut app)? {
            Some(true) => {}  // accepted
            _ => return Ok(()), // declined or cancelled on first launch → exit
        }
    }

    // Auto-scan on startup if no cached path
    let save_found = if app.save_folder.is_some() {
        true
    } else {
        app.set_spinner(true);
        terminal.draw(|f| {
            tui::draw_text_screen(f, &app.tui_state,
                &[Line::from(Span::styled("Locating save folder…", Style::default().add_modifier(Modifier::BOLD)))],
                "",
            );
        })?;
        let folders = discovery::discover_save_folders();
        app.set_spinner(false);
        if let Some(first) = folders.first() {
            app.save_folder = Some(first.path.clone());
            app.config.last_path = Some(first.path.display().to_string());
            app.config.last_scan = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            crate::config::save_config(&app.config_path, &app.config)?;
            refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
            true
        } else {
            false
        }
    };

    // Main menu loop
    let mut menu_state = ListState::default().with_selected(Some(0));

    loop {
        terminal.draw(|f| {
            let cols = f.area().width;
            let rows = f.area().height;
            if cols < 60 || rows < 15 {
                draw_too_small(f);
                return;
            }
            tui::draw_main_menu(f, &mut menu_state, &app.tui_state, save_found);
        })?;

        let max_idx = if save_found { 6 } else { 7 };
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release { continue; }
            match key.code {
                KeyCode::Up => {
                    let i = menu_state.selected().unwrap_or(0);
                    menu_state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = menu_state.selected().unwrap_or(0);
                    menu_state.select(Some((i + 1).min(max_idx)));
                }
                KeyCode::Enter => {
                    let idx = menu_state.selected().unwrap_or(0);
                    // Remap: when locate is hidden, shift by 1
                    let remap = if save_found { idx + 1 } else { idx };
                    match remap {
                        0 => action_locate_saves(terminal, &mut app)?,
                        1 => action_recover_bak(terminal, &mut app)?,
                        2 => action_create_backup(terminal, &mut app)?,
                        3 => action_restore_backup(terminal, &mut app)?,
                        4 => action_inspect_saves(terminal, &mut app)?,
                        5 => run_ini_submenu(terminal, &mut app)?,
                        6 => {
                            match run_disclaimer(terminal, &mut app)? {
                                Some(false) => return Ok(()), // declined → exit
                                _ => {} // accepted or cancelled → stay
                            }
                        }
                        7 => return Ok(()), // Exit
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
        let key = read_key_event()?;
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => selected_yes = !selected_yes,
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                guard::log_action("LICENSE", "accepted", "OK", &app.log_path)?;
                app.config.disclaimer_accepted = true;
                crate::config::save_config(&app.config_path, &app.config)?;
                return Ok(Some(true));
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                guard::log_action("LICENSE", "declined", "OK", &app.log_path)?;
                app.config.disclaimer_accepted = false;
                crate::config::save_config(&app.config_path, &app.config)?;
                return Ok(Some(false));
            }
            KeyCode::Esc => {
                return Ok(None);
            }
            KeyCode::Enter => {
                guard::log_action("LICENSE", if selected_yes { "accepted" } else { "declined" }, "OK", &app.log_path)?;
                let accepted = selected_yes;
                app.config.disclaimer_accepted = accepted;
                crate::config::save_config(&app.config_path, &app.config)?;
                return Ok(Some(accepted));
            }
            _ => {}
        }
    }
}

// ── menu actions ───────────────────────────────────────────────────────────

fn action_locate_saves<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    // Run scan on a background thread so we can show a live elapsed timer.
    let (tx, rx) = std::sync::mpsc::channel();
    let scan_start = std::time::Instant::now();
    std::thread::spawn(move || {
        let result = discovery::discover_save_folders();
        tx.send(result).ok();
    });

    app.set_spinner(true);
    let folders = loop {
        let elapsed = scan_start.elapsed().as_secs();
        terminal.draw(|f| {
            tui::draw_text_screen(
                f,
                &app.tui_state,
                &[
                    Line::from(Span::styled(
                        "Scanning for Subnautica save files…",
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "The first scan checks every user profile and common",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(Span::styled(
                        "install location — it may take a moment.",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  ⏱  {elapsed}s elapsed"),
                        Style::default().fg(Color::Cyan),
                    )),
                ],
                "",
            );
        })?;

        match rx.try_recv() {
            Ok(f) => break f,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break Vec::new(),
            _ => {}
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    app.set_spinner(false);

    let elapsed = scan_start.elapsed();
    let elapsed_str = if elapsed.as_secs() >= 1 {
        format!("{:.1}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", elapsed.as_millis())
    };

    if folders.is_empty() {
        app.set_status("No Subnautica 2 save folders detected.", tui::StatusStyle::Error);
        wait_for_key(terminal, app)?;
        return Ok(());
    }

    // Cache the first result
    if let Some(first) = folders.first() {
        app.save_folder = Some(first.path.clone());
        app.config.last_path = Some(first.path.display().to_string());
        app.config.last_scan = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        crate::config::save_config(&app.config_path, &app.config)?;
    }

    refresh_stats(&mut app.tui_state, app.save_folder.as_deref());

    let msg = format!("Found {} save folder(s) in {elapsed_str}.", folders.len());
    app.set_status(&msg, tui::StatusStyle::Success);

    // Show found folders
    let lines: Vec<Line> = std::iter::once(Line::from(Span::styled(
        format!("Found {} save folder(s):", folders.len()),
        Style::default().fg(Color::Green),
    )))
    .chain(folders.iter().map(|f| {
        Line::from(Span::styled(
            format!("  {} — {}", f.label, f.path.display()),
            Style::default().fg(Color::Gray),
        ))
    }))
    .collect();

    let prompt = "Press any key to return to menu…";
    loop {
        terminal.draw(|f| tui::draw_text_screen(f, &app.tui_state, &lines, prompt))?;
        if let Event::Key(_) = event::read()? {
            break;
        }
    }

    Ok(())
}

fn action_recover_bak<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let save_folder = ensure_save_folder(terminal, app)?;

    // Warn if game is running
    if guard::game_running() {
        app.set_status(
            "Subnautica 2 is running — save files may be locked!",
            tui::StatusStyle::Warning,
        );
    }

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
    let items: Vec<String> = bak_summaries
        .iter()
        .map(|s| {
            let num = slot_number(&s.slot);
            let first = labelled.insert(s.slot.clone());
            let label_col = if first { format!("Slot {num}") } else { String::new() };
            let name = s.display_name.as_deref().unwrap_or("(unnamed)");
            let name_col = if name.len() > 24 {
                format!("{}…", &name[..23])
            } else {
                name.to_string()
            };
            let date = s.mtime.as_deref().unwrap_or("?");
            let save_type = if s.is_online { "Online" } else { "Local" };
            format!(
                "  {:<8}  {:<26}  {:<6}  {:>6}  {}",
                label_col,
                name_col,
                save_type,
                format_size(s.size),
                date,
            )
        })
        .collect();
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

    // Filenames for the info bar, and descriptions
    let filenames: Vec<String> = bak_summaries
        .iter()
        .map(|s| s.filename.clone())
        .collect();
    let descs: Vec<String> = bak_summaries
        .iter()
        .map(|_| "Restore this backup to its canonical .sav file".to_string())
        .collect();
    let desc_refs: Vec<&str> = descs.iter().map(|s| s.as_str()).collect();
    let mut state = ListState::default().with_selected(Some(0));

    loop {
        let selected_info = state
            .selected()
            .and_then(|i| filenames.get(i))
            .map(|s| s.as_str());

        terminal.draw(|f| {
            tui::draw_picker_with_info(
                f,
                &app.tui_state,
                &item_refs,
                &desc_refs,
                &mut state,
                selected_info,
            );
        })?;
        let key = read_key_event()?;
                match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(bak_summaries.len().saturating_sub(1))));
                }
                KeyCode::Enter => {
                    let idx = state.selected().unwrap_or(0);
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
                    } else { None };

                    // Mode change warning
                    let mode_entry = target_meta.as_ref().and_then(|(_, _, live, _)| {
                        if *live != chosen.is_online {
                            let from = if *live { "Online" } else { "Local" };
                            let to = if chosen.is_online { "Online" } else { "Local" };
                            Some(("⚠ Mode change", format!("{from} → {to}")))
                        } else { None }
                    });
                    // Name change warning
                    let name_entry = target_meta.as_ref().and_then(|(_, _, _, live_name)| {
                        let bak_name = chosen.display_name.as_deref().unwrap_or("(unnamed)");
                        match live_name {
                            Some(live) if live != bak_name => {
                                Some(("⚠ Name change", format!("{live} → {bak_name}")))
                            }
                            _ => None,
                        }
                    });

                    // Build details
                    let src_line = format!("{}  {}  {}", chosen.filename, format_size(chosen.size), chosen.mtime.as_deref().unwrap_or("?"));
                    let mut details = vec![
                        ("Slot", chosen.slot.as_str()),
                        ("Name", chosen.display_name.as_deref().unwrap_or("(unnamed)")),
                        ("Backup", src_line.as_str()),
                    ];
                    let tgt_line: String;
                    if let Some((sz, mt, _, _)) = &target_meta {
                        tgt_line = format!("{}  {}  {}", target, format_size(*sz), mt.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).as_deref().unwrap_or("?"));
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
                        details.push(("⚠ No backup", "create a full backup first"));
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

    refresh_stats(&mut app.tui_state, Some(&save_folder));
    Ok(())
}

fn action_create_backup<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let save_folder = ensure_save_folder(terminal, app)?;

    if guard::game_running() {
        app.set_status("Subnautica 2 is running — files may be locked.", tui::StatusStyle::Warning);
    }

    app.set_status("Creating backup…", tui::StatusStyle::Info);
    app.set_spinner(true);
    terminal.draw(|f| {
        tui::draw_text_screen(
            f,
            &app.tui_state,
            &[Line::from(Span::styled(
                "Creating full backup…",
                Style::default().add_modifier(Modifier::BOLD),
            ))],
            "Copying save folder contents…",
        );
    })?;

    let backup_root = app.backup_root();
    match ops::create_full_backup(&save_folder, &backup_root) {
        Ok(result) => {
            app.set_spinner(false);
            let verified = if result.verified { "verified" } else { "unverified" };
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
            guard::log_action("MANUAL_BAK", &save_folder.display().to_string(), &format!("FAILED: {e}"), &app.log_path)?;
            ok_dialog(terminal, app, "Backup Failed", &msg)?;
        }
    }
    Ok(())
}

fn action_restore_backup<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let save_folder = ensure_save_folder(terminal, app)?;
    let backup_root = app.backup_root();

    let backups = ops::list_full_backups(&backup_root);
    if backups.is_empty() {
        ok_dialog(terminal, app, "No Backups", "No full backups found in NotAlterra_Backups.\nUse 'Create full backup' from the main menu first.")?;
        return Ok(());
    }

    let items: Vec<String> = backups
        .iter()
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            format!("  {name}")
        })
        .collect();
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let descs: Vec<&str> = vec!["Restore this backup into the save folder"; backups.len()];
    let mut state = ListState::default().with_selected(Some(0));

    loop {
        terminal.draw(|f| {
            tui::draw_picker(f, &app.tui_state, &item_refs, &descs, &mut state);
        })?;
        let key = read_key_event()?;
            match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(backups.len().saturating_sub(1))));
                }
                KeyCode::Enter => {
                    let idx = state.selected().unwrap_or(0);
                    let chosen = &backups[idx];
                    let name = chosen.file_name().unwrap().to_string_lossy().to_string();

                    let mut restore_details = vec![("Backup", name.as_str())];
                    if !has_existing_backup(app) {
                        restore_details.push(("⚠ No backup", "create a full backup first"));
                    }
                    let accepted = confirm_modal(terminal, app, "Confirm Restore", &restore_details)?;

                    if accepted {
                        guard::log_action("AUTO_BAK", &format!("pre-restore → {}", save_folder.display()), "OK", &app.log_path)?;
                        match ops::restore_full_backup(chosen, &save_folder, &backup_root) {
                            Ok(()) => {
                                app.set_status("Restore complete. Previous files preserved.", tui::StatusStyle::Success);
                                guard::log_action("RESTORE", &format!("{} → {}", name, save_folder.display()), "OK", &app.log_path)?;
                            }
                            Err(e) => {
                                app.set_status(&format!("Restore failed: {e}"), tui::StatusStyle::Error);
                                guard::log_action("RESTORE", &name, &format!("FAILED: {e}"), &app.log_path)?;
                            }
                        }
                    }
                    break;
                }
                KeyCode::Esc => break,
                _ => {}
            }
    }

    refresh_stats(&mut app.tui_state, Some(&save_folder));
    Ok(())
}

// ── .ini submenu ───────────────────────────────────────────────────────────

fn run_ini_submenu<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let config_path = get_config_path(terminal, app)?;
    let backup_root = app.backup_root();

    let items: Vec<&str> = vec![
        "Backup .ini files",
        "Restore .ini files from backup",
        "Delete .ini files (requires backup)",
    ];
    let descs: Vec<&str> = vec![
        "Copy all .ini files from Config/Windows to NotAlterra_Backups",
        "Restore .ini files from a previous backup",
        "Remove .ini files — game regenerates defaults (backup required first)",
    ];
    let mut state = ListState::default().with_selected(Some(0));

    loop {
        terminal.draw(|f| {
            tui::draw_sub_menu(f, &app.tui_state, "Config (.ini) Management", &items, &descs, &mut state);
        })?;
        let key = read_key_event()?;
                match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(2)));
                }
                KeyCode::Enter => {
                    let idx = state.selected().unwrap_or(0);
                    match idx {
                        0 => ini_backup_action(terminal, app, &config_path, &backup_root)?,
                        1 => ini_restore_action(terminal, app, &config_path, &backup_root)?,
                        2 => ini_delete_action(terminal, app, &config_path, &backup_root)?,
                        _ => {}
                    }
                }
                KeyCode::Esc => break,
                _ => {}
                }
    }

    refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
    Ok(())
}

fn ini_backup_action<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    config_path: &Path,
    backup_root: &Path,
) -> Result<()> {
    if guard::game_running() {
        app.set_status("Subnautica 2 is running — files may be locked.", tui::StatusStyle::Warning);
    }

    match ops::backup_ini_files(config_path, backup_root) {
        Ok(result) => {
            let verified = if result.verified { "verified" } else { "unverified" };
            app.set_status(
                &format!("Config backup created: {} files ({})", result.files_copied, verified),
                tui::StatusStyle::Success,
            );
            guard::log_action("CONFIG_BAK", &result.dest_dir.display().to_string(), "OK", &app.log_path)?;
            refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
        }
        Err(e) => {
            app.set_status(&format!("Config backup failed: {e}"), tui::StatusStyle::Error);
        }
    }

    wait_for_key(terminal, app)?;
    Ok(())
}

fn ini_restore_action<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    config_path: &Path,
    backup_root: &Path,
) -> Result<()> {
    let backups = ops::list_ini_backups(backup_root);
    if backups.is_empty() {
        ok_dialog(terminal, app, "No .ini Backups", "No .ini backups found.\nUse 'Backup .ini files' first.")?;
        return Ok(());
    }

    let items: Vec<String> = backups
        .iter()
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            format!("  {name}")
        })
        .collect();
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let descs: Vec<&str> = vec!["Restore this .ini backup"; backups.len()];
    let mut state = ListState::default().with_selected(Some(0));

    loop {
        terminal.draw(|f| {
            tui::draw_picker(f, &app.tui_state, &item_refs, &descs, &mut state);
        })?;
        let key = read_key_event()?;
            match key.code {
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some((i + 1).min(backups.len().saturating_sub(1))));
                }
                KeyCode::Enter => {
                    let idx = state.selected().unwrap_or(0);
                    let chosen = &backups[idx];

                    guard::log_action("AUTO_BAK", &format!("ini pre-restore → {}", config_path.display()), "OK", &app.log_path)?;
                    match ops::restore_ini_files(chosen, config_path, backup_root) {
                        Ok(()) => {
                            guard::log_action("CONFIG_RESTORE", &chosen.display().to_string(), "OK", &app.log_path)?;
                            ok_dialog(terminal, app, ".ini Restore Complete", ".ini files restored.")?;
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

    Ok(())
}

fn ini_delete_action<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    config_path: &Path,
    backup_root: &Path,
) -> Result<()> {
    match ops::delete_ini_files(config_path, backup_root) {
        Ok(n) => {
            let msg = format!("Deleted {n} .ini file(s).\nThe game will regenerate defaults on next launch.");
            guard::log_action("CONFIG_DEL", &config_path.display().to_string(), "OK", &app.log_path)?;
            ok_dialog(terminal, app, ".ini Delete Complete", &msg)?;
        }
        Err(e) => {
            ok_dialog(terminal, app, ".ini Delete Refused", &format!("{e}"))?;
        }
    }
    Ok(())
}

// ── inspect saves ──────────────────────────────────────────────────────────

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
    let items: Vec<String> = files.iter().map(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        let slot = crate::gvas::derive_slot_from_filename(&name).unwrap_or_else(|| "?".into());
        let num = slot_number(&slot);
        let first = labelled.insert(slot.clone());
        let label = if first { format!("Slot {num}") } else { String::new() };
        let sz = e.metadata().map(|m| m.len()).unwrap_or(0);
        format!("  {:<8}  {:<30}  {:>7}", label, name, format_size(sz))
    }).collect();
    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let filenames: Vec<String> = files.iter().map(|e| e.file_name().to_string_lossy().to_string()).collect();
    let descs = vec!["Press Enter to view full GVAS metadata"; files.len()];
    let desc_refs: Vec<&str> = descs.iter().map(|s| *s).collect();
    let mut state = ListState::default().with_selected(Some(0));

    loop {
        let selected_info = state.selected().and_then(|i| filenames.get(i)).map(|s| s.as_str());
        terminal.draw(|f| {
            tui::draw_picker_with_info(f, &app.tui_state, &item_refs, &desc_refs, &mut state, selected_info);
        })?;
        let key = read_key_event()?;
        match key.code {
            KeyCode::Up => { let i = state.selected().unwrap_or(0); state.select(Some(i.saturating_sub(1))); }
            KeyCode::Down => { let i = state.selected().unwrap_or(0); state.select(Some((i+1).min(files.len().saturating_sub(1)))); }
            KeyCode::Enter => {
                let idx = state.selected().unwrap_or(0);
                let path = files[idx].path();
                match crate::gvas::extract_full_metadata(&path) {
                    Ok(meta) => {
                        let dim = Style::default().fg(Color::DarkGray);
                        let val = Style::default().fg(Color::White);
                        let hl = Style::default().fg(Color::Cyan);
                        let mut lines: Vec<Line> = vec![
                            Line::from(Span::styled(filenames[idx].clone(), hl.add_modifier(Modifier::BOLD))),
                            Line::from(""),
                        ];
                        let fields: Vec<(&str, &str, String)> = vec![
                            ("SlotName", ":", meta.slot_name.as_deref().unwrap_or("?").into()),
                            ("DisplayName", ":", meta.display_name.as_deref().unwrap_or("(unnamed)").into()),
                            ("Online", ":", (if meta.is_online { "yes" } else { "no" }).into()),
                            ("Was Multi", ":", (if meta.was_multiplayer { "yes" } else { "no" }).into()),
                            ("GameMode", ":", meta.game_mode.as_deref().unwrap_or("?").into()),
                            ("Level", ":", meta.level_name.as_deref().unwrap_or("?").into()),
                            ("Build", ":", meta.build_number.map_or("?".into(), |n| n.to_string())),
                            ("Branch", ":", meta.build_branch.as_deref().unwrap_or("?").into()),
                            ("Saves", ":", meta.saves_count.map_or("?".into(), |n| n.to_string())),
                            ("Ver", ":", meta.latest_version.map_or("?".into(), |n| n.to_string())),
                            ("DataVer", ":", meta.data_version.map_or("?".into(), |n| n.to_string())),
                        ];
                        let max_label: usize = fields.iter().map(|(k, _, _)| k.len()).max().unwrap_or(8);
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
    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────────

/// Ensure we have a save folder — prompt discovery if not.
fn ensure_save_folder<B: Backend>(_terminal: &mut Terminal<B>, app: &mut App) -> Result<PathBuf> {
    // Try cached path first
    if let Some(ref sf) = app.save_folder {
        if sf.exists() {
            return Ok(sf.clone());
        }
    }

    // Discover
    let folders = discovery::discover_save_folders();
    if let Some(first) = folders.first() {
        app.save_folder = Some(first.path.clone());
        app.config.last_path = Some(first.path.display().to_string());
        crate::config::save_config(&app.config_path, &app.config)?;
        refresh_stats(&mut app.tui_state, app.save_folder.as_deref());
        return Ok(first.path.clone());
    }

    anyhow::bail!("No save folders found. Run 'Locate Subnautica save files' from the main menu.")
}

/// Get or discover the Config\Windows path.
fn get_config_path<B: Backend>(_terminal: &mut Terminal<B>, app: &mut App) -> Result<PathBuf> {
    // Try cached config path
    if let Some(ref cp) = app.config.config_path {
        let p = PathBuf::from(cp);
        if p.exists() {
            return Ok(p);
        }
    }

    // Derive from save folder
    if let Some(ref sf) = app.save_folder {
        if let Some(cp) = discovery::derive_config_path(sf) {
            return Ok(cp);
        }
    }

    // Fall back to discovery
    let folders = discovery::discover_save_folders();
    for f in &folders {
        if let Some(cp) = discovery::derive_config_path(&f.path) {
            app.config.config_path = Some(cp.display().to_string());
            crate::config::save_config(&app.config_path, &app.config)?;
            return Ok(cp);
        }
    }

    anyhow::bail!("Cannot determine Config/Windows path. Run 'Locate Subnautica save files' first.")
}

/// Derive the target .sav name from a .bak filename.
/// Read the next keyboard event, silently skipping KeyRelease events.
/// Key-repeat and initial press are both accepted.
fn read_key_event() -> Result<crossterm::event::KeyEvent> {
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release {
                    return Ok(key);
                }
            }
        }
        // Other events (terminal changes etc) trigger a loop → redraw
    }
}

/// Determine if a save folder is cloud-backed (Xbox/Game Pass wgs path).
fn is_cloud_path(path: &Path) -> bool {
    let s = path.to_string_lossy().to_lowercase();
    s.contains("\\packages\\") || s.contains("/packages/") || s.contains("\\wgs") || s.contains("/wgs")
}

/// Extract the numeric suffix from a slot name like "savegame_3".
fn slot_number(slot: &str) -> String {
    slot.strip_prefix("savegame_")
        .map(|s| s.to_string())
        .unwrap_or_else(|| slot.to_string())
}

fn ok_dialog_styled<B: Backend>(terminal: &mut Terminal<B>, app: &App, title: &str, lines: &[Line]) -> Result<()> {
    loop {
        terminal.draw(|f| tui::draw_ok_dialog_styled(f, &app.tui_state, title, lines))?;
        let key = read_key_event()?;
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            return Ok(());
        }
    }
}

fn ok_dialog<B: Backend>(terminal: &mut Terminal<B>, app: &App, title: &str, msg: &str) -> Result<()> {
    loop {
        terminal.draw(|f| tui::draw_ok_dialog(f, &app.tui_state, title, msg))?;
        let key = read_key_event()?;
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            return Ok(());
        }
    }
}

fn has_existing_backup(app: &App) -> bool {
    let root = app.backup_root();
    root.exists() && std::fs::read_dir(&root).map_or(false, |mut d| d.any(|e| e.map_or(false, |e| e.path().is_dir())))
}

fn require_backup<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    if has_existing_backup(app) {
        return Ok(true);
    }
    app.set_status("No backup found — create a full backup before destructive actions.", tui::StatusStyle::Error);
    wait_for_key(terminal, app)?;
    Ok(false)
}

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
        let key = read_key_event()?;
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
            let p = Paragraph::new(prompt_span)
                .alignment(Alignment::Center);
            f.render_widget(p, centered_bottom(f.area()));
            // Also show the status message if set
            if let Some(ref msg) = app.tui_state.status_message {
                let color = match app.tui_state.status_style {
                    tui::StatusStyle::Success => Color::Green,
                    tui::StatusStyle::Warning => Color::Yellow,
                    tui::StatusStyle::Error => Color::Red,
                    tui::StatusStyle::Info => Color::Cyan,
                    tui::StatusStyle::Neutral => Color::Gray,
                };
                let line = Line::from(Span::styled(format!(" [{msg}]"), Style::default().fg(color)));
                f.render_widget(
                    Paragraph::new(line),
                    Rect {
                        x: f.area().x,
                        y: f.area().height.saturating_sub(2),
                        width: f.area().width,
                        height: 1,
                    },
                );
            }
        })?;
        if let Event::Key(_) = event::read()? {
            break;
        }
    }
    Ok(())
}

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
