//! Modern terminal UI built on ratatui + crossterm.
//!
//! Design principles:
//! - Dashboard layout: header bar, main panel, status line
//! - Keyboard-first: arrow keys + Enter/Esc, no mouse dependency
//!
//! Terminal UI rendering for NotAlterra.
//!
//! Uses ratatui + crossterm to draw menu screens, picker lists, dialogs,
//! metadata inspectors, and the animated whale separator.  All rendering
//! is stateless — callers pass in an [`AppState`] snapshot.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::time::Instant;

// ── app state ──────────────────────────────────────────────────────────────

/// Global application state passed through every frame.
pub struct AppState {
    /// Terminal dimensions (updated on Resize events)
    pub cols: u16,
    pub rows: u16,
    /// Current save-folder path (for the header bar)
    pub save_path: Option<String>,
    /// Number of live .sav files in the current folder
    pub live_save_count: usize,
    /// Number of .bak backup files
    pub backup_count: usize,
    /// Whether a .ini backup exists
    pub has_ini_backup: bool,
    /// Context-specific path shown on the right side of the header bar.
    /// When `None`, falls back to `save_path`.
    pub context_path: Option<String>,
    /// Version string for the header
    pub version: String,
    /// Last operation result (for the status bar)
    pub status_message: Option<String>,
    pub status_style: StatusStyle,
    /// Spinner state
    pub spinner_active: bool,
    pub spinner_start: Option<Instant>,
    pub whale_start: Instant,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StatusStyle {
    Info,
    Success,
    Warning,
    Error,
    Neutral,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            save_path: None,
            live_save_count: 0,
            backup_count: 0,
            has_ini_backup: false,
            context_path: None,
            version: String::new(),
            status_message: None,
            status_style: StatusStyle::Neutral,
            spinner_active: false,
            spinner_start: None,
            whale_start: Instant::now(),
        }
    }
}

// ── public rendering entry points ──────────────────────────────────────────

/// Draw the main menu.
pub fn draw_main_menu(f: &mut Frame, state: &mut ListState, app: &AppState) {
    let items: Vec<&str> = vec![
        " Set Subnautica 2 location",
        " Recover save file",
        "",
        " Set backup location",
        " Create full backup",
        " Restore full backup",
        "",
        " Manage UE5 Config (.ini) files",
        "",
        " View disclaimer",
        " Exit",
    ];
    let descs: Vec<&str> = vec![
        "Enter your Subnautica 2 save folder path (paste supported)",
        "Restore a save file from a backup",
        "",
        "Choose where backup archives are stored (default: next to the binary)",
        "Copy the savegame files to NotAlterra_Backups",
        "Restore a full backup from NotAlterra_Backups",
        "",
        "Backup, restore, or delete .ini files in Config/Windows",
        "",
        "Re-read the disclaimer and terms of use",
        "Close NotAlterra",
    ];
    let chunks = standard_layout(f.area(), items.len());

    draw_header(f, chunks[0], app);
    draw_status_dashboard(f, chunks[1], app);

    let prompt = "↑/↓ navigate  Enter select";
    draw_select_list(f, chunks[2], &items, &descs, prompt, state);

    draw_status_bar(f, chunks[4], app);
}

/// Draw the disclaimer popup with full warning text.
pub fn draw_disclaimer_popup(f: &mut Frame, app: &AppState, selected_yes: bool) {
    // Whale at bottom row
    let bar = Rect {
        x: 0,
        y: f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };
    draw_whale_separator(f, bar, app);
    let popup_w = 60.min(f.area().width.saturating_sub(4));
    let popup_h = 18.min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, area);

    let inner = inner(area, 2, 1);

    let lines = vec![
        Line::from(Span::styled(
            "DISCLAIMER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This tool was created using an AI Agent. While",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "every effort has been made to ensure it works",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "correctly, you should review the code and test",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "on a backup before using it on live save files.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "NotAlterra is not affiliated with Unknown Worlds",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Entertainment or KRAFTON. Use at your own risk.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "The author is NOT responsible for any data loss.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        Rect {
            height: 11,
            ..inner
        },
    );

    let yes_style = if selected_yes {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let no_style = if !selected_yes {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    let buttons = Line::from(vec![
        Span::styled("[ Accept ]", yes_style),
        Span::raw("    "),
        Span::styled("[ Decline ]", no_style),
    ]);
    f.render_widget(
        Paragraph::new(buttons).alignment(Alignment::Center),
        Rect {
            y: inner.y + 12,
            height: 1,
            ..inner
        },
    );
}

/// Draw a simple confirmation popup with \[ Yes \] \[ No \] buttons.
pub fn draw_confirm_popup(
    f: &mut Frame,
    app: &AppState,
    title: &str,
    details: &[(&str, &str)],
    selected_yes: bool,
) {
    let max_w = details
        .iter()
        .map(|(k, v)| k.len() + v.len() + 4)
        .max()
        .unwrap_or(20)
        .max(30) as u16;
    let popup_w = (max_w + 4).min(f.area().width.saturating_sub(4));
    let popup_h = (details.len() as u16 + 6).min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, area);

    let inner = inner(area, 2, 1);

    // Title
    f.render_widget(
        Paragraph::new(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Rect { height: 1, ..inner },
    );

    // Details
    let detail_lines: Vec<Line> = details
        .iter()
        .map(|(k, v)| {
            let icon = if k.starts_with('⚠') {
                Color::Yellow
            } else {
                Color::Gray
            };
            Line::from(vec![
                Span::styled(format!("{k}: "), Style::default().fg(icon)),
                Span::styled(*v, Style::default()),
            ])
        })
        .collect();
    f.render_widget(
        Paragraph::new(detail_lines),
        Rect {
            y: inner.y + 2,
            height: details.len() as u16,
            ..inner
        },
    );

    // Yes / No buttons
    let yes_style = if selected_yes {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let no_style = if !selected_yes {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    let buttons = Line::from(vec![
        Span::styled("[ Yes ]", yes_style),
        Span::raw("    "),
        Span::styled("[ No ]", no_style),
    ]);
    f.render_widget(
        Paragraph::new(buttons).alignment(Alignment::Center),
        Rect {
            y: inner.y + inner.height.saturating_sub(1),
            height: 1,
            ..inner
        },
    );

    // Whale
    let bar = Rect {
        x: 0,
        y: f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };
    draw_whale_separator(f, bar, app);
}

/// Render an informational dialog with a plain-text message and OK button.
/// Auto-sizes to fit content.  Title is displayed in cyan, message in gray,
/// whale separator at the bottom.  Press Enter or Space to dismiss.
pub fn draw_ok_dialog(f: &mut Frame, app: &AppState, title: &str, message: &str) {
    let content_w = message
        .lines()
        .map(|l| l.len())
        .max()
        .unwrap_or(20)
        .max(title.len()) as u16
        + 10;
    let popup_w = content_w.max(50).min(f.area().width.saturating_sub(4));
    let popup_h = (message.lines().count() as u16 + 7).min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
    let inner = inner(area, 2, 1);
    f.render_widget(
        Paragraph::new(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Rect { height: 1, ..inner },
    );
    let msg_h = message.lines().count() as u16;
    f.render_widget(
        Paragraph::new(message.to_string())
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left),
        Rect {
            x: inner.x + 2,
            y: inner.y + 2,
            width: inner.width.saturating_sub(4),
            height: msg_h,
        },
    );
    let ok = Span::styled(
        "[ OK ]",
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(
        Paragraph::new(ok).alignment(Alignment::Center),
        Rect {
            y: inner.y + inner.height.saturating_sub(2),
            height: 1,
            ..inner
        },
    );

    // Whale
    let bar = Rect {
        x: 0,
        y: f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };
    draw_whale_separator(f, bar, app);
}

/// Render a non-interactive info dialog — no buttons, renders once, caller
/// is expected to return to the event loop (the dialog stays visible until
/// the next `terminal.draw()` replaces it).
pub fn draw_info_dialog(f: &mut Frame, app: &AppState, title: &str, message: &str) {
    let content_w = message
        .lines()
        .map(|l| l.len())
        .max()
        .unwrap_or(20)
        .max(title.len()) as u16
        + 10;
    let popup_w = content_w.max(50).min(f.area().width.saturating_sub(4));
    let popup_h = (message.lines().count() as u16 + 6).min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
    let inner = inner(area, 2, 1);
    f.render_widget(
        Paragraph::new(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Rect { height: 1, ..inner },
    );
    let msg_h = message.lines().count() as u16;
    f.render_widget(
        Paragraph::new(message.to_string())
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left),
        Rect {
            x: inner.x + 2,
            y: inner.y + 2,
            width: inner.width.saturating_sub(4),
            height: msg_h,
        },
    );
    // No button — this is informational only

    // Whale
    let bar = Rect {
        x: 0,
        y: f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };
    draw_whale_separator(f, bar, app);
}

/// Render a dialog with styled content lines.  Supports inline formatting
/// (colors, bold) via [`Line`] slices.  Use for metadata displays, help
/// text, or any content that needs per-span styling.
pub fn draw_ok_dialog_styled(f: &mut Frame, app: &AppState, title: &str, lines: &[Line]) {
    let content_w = lines
        .iter()
        .map(|l| l.width() as u16)
        .max()
        .unwrap_or(20)
        .max(title.len() as u16)
        + 10;
    let popup_w = content_w.max(50).min(f.area().width.saturating_sub(4));
    let popup_h = (lines.len() as u16 + 7).min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
    let inner = inner(area, 2, 1);
    f.render_widget(
        Paragraph::new(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Rect { height: 1, ..inner },
    );
    f.render_widget(
        Paragraph::new(lines.to_vec())
            .style(Style::default())
            .alignment(Alignment::Left),
        Rect {
            x: inner.x + 2,
            y: inner.y + 2,
            width: inner.width.saturating_sub(4),
            height: lines.len() as u16,
        },
    );
    let ok = Span::styled(
        "[ OK ]",
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(
        Paragraph::new(ok).alignment(Alignment::Center),
        Rect {
            y: inner.y + inner.height.saturating_sub(2),
            height: 1,
            ..inner
        },
    );

    // Whale
    let bar = Rect {
        x: 0,
        y: f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };
    draw_whale_separator(f, bar, app);
}

/// Return a rectangle centered in `r` by the given width and height percentages.
/// Shrink a rectangle to the given absolute width and height, centered.
/// Return a rectangle centered in `r` by the given width and height percentages.
fn centered_rect_size(w: u16, h: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(h)) / 2),
            Constraint::Length(h),
            Constraint::Length((r.height.saturating_sub(h)) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(w)) / 2),
            Constraint::Length(w),
            Constraint::Length((r.width.saturating_sub(w)) / 2),
        ])
        .split(popup[1])[1]
}

/// Draw a sub-menu (e.g. Config management).
pub fn draw_sub_menu(
    f: &mut Frame,
    app: &AppState,
    title: &str,
    items: &[&str],
    descs: &[&str],
    state: &mut ListState,
) {
    let chunks = standard_layout(f.area(), items.len());

    draw_header(f, chunks[0], app);

    let title_p = Paragraph::new(Span::styled(
        format!("   {title}"),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(title_p, chunks[1]);

    draw_select_list(
        f,
        chunks[2],
        items,
        descs,
        "↑/↓ navigate  Enter select  Esc back",
        state,
    );
    draw_status_bar(f, chunks[4], app);
}

/// Draw a full-screen text display with a "press any key" prompt at the
/// bottom.  Used for status messages during long operations (scanning,
/// backing up) and for displaying scan results.
pub fn draw_text_screen(f: &mut Frame, app: &AppState, lines: &[Line], prompt: &str) {
    let chunks = standard_layout(f.area(), lines.len());
    draw_header(f, chunks[0], app);

    f.render_widget(Paragraph::new(lines.to_vec()), chunks[2]);

    let prompt_p = Paragraph::new(Span::styled(prompt, Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Center);
    f.render_widget(prompt_p, chunks[4]);
}

/// Draw a file/folder picker list.
/// `pinned_header` renders items\[0\] as a fixed header above the scrollable list.
pub fn draw_picker(
    f: &mut Frame,
    app: &AppState,
    items: &[&str],
    descs: &[&str],
    state: &mut ListState,
    pinned_header: bool,
) {
    draw_picker_with_info(f, app, items, descs, state, pinned_header);
}

/// Draw a file/folder picker list with an extra selected-item info line
/// (e.g. showing the full filename of the highlighted .bak file).
/// `pinned_header` renders items\[0\] as a fixed header above the scrollable list.
pub fn draw_picker_with_info(
    f: &mut Frame,
    app: &AppState,
    items: &[&str],
    descs: &[&str],
    state: &mut ListState,
    pinned_header: bool,
) {
    let chunks = standard_layout(f.area(), items.len());
    draw_header(f, chunks[0], app);

    let prompt = "↑/↓ navigate | Enter select | Esc cancel";
    draw_select_list_with_info(f, chunks[2], items, descs, prompt, state, pinned_header);
    draw_status_bar(f, chunks[4], app);
}

// ── internal drawing helpers ───────────────────────────────────────────────

fn standard_layout(area: Rect, _menu_items: usize) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(2), // dashboard
            Constraint::Min(1),    // menu (fills remaining)
            Constraint::Length(1), // spacer
            Constraint::Length(1), // status bar
        ])
        .split(area)
        .to_vec()
}

/// Render the title bar with version information.
fn draw_header(f: &mut Frame, area: Rect, app: &AppState) {
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Cyan));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(inner(area, 1, 0));

    let title_line = Line::from(vec![
        Span::styled(
            "NotAlterra",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(app.version.clone(), Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // Header path priority:
    //   1. context_path = Some("path")  → show that path
    //   2. context_path = Some("")       → show nothing (blank/disclaimer/exit)
    //   3. context_path = None           → fall back to save_path
    let max_w = chunks[1].width.saturating_sub(2) as usize;
    let path_line = match &app.context_path {
        Some(p) if p.is_empty() => {
            // Show nothing — blank line, disclaimer, or exit
            Paragraph::new(Span::raw(""))
        }
        Some(p) => {
            let display = truncate_path_tail(p, max_w);
            Paragraph::new(Span::styled(display, Style::default().fg(Color::Gray)))
                .alignment(Alignment::Right)
        }
        None => {
            // Fall back to save_path
            if let Some(ref save) = app.save_path {
                let display = truncate_path_tail(save, max_w);
                Paragraph::new(Span::styled(display, Style::default().fg(Color::Gray)))
                    .alignment(Alignment::Right)
            } else {
                Paragraph::new(Span::styled(
                    "no save folder selected",
                    Style::default().fg(Color::DarkGray),
                ))
                .alignment(Alignment::Right)
            }
        }
    };
    f.render_widget(path_line, chunks[1]);
    f.render_widget(header_block, area);
}

/// Render the status dashboard beneath the header.
fn draw_status_dashboard(f: &mut Frame, area: Rect, app: &AppState) {
    let live = Span::styled(
        format!(
            " Save{}: {} ",
            if app.live_save_count == 1 { "" } else { "s" },
            if app.save_path.is_some() {
                app.live_save_count.to_string()
            } else {
                "—".into()
            }
        ),
        Style::default().fg(Color::Green),
    );
    let bak = Span::styled(
        format!(
            " Backup{}: {} ",
            if app.backup_count == 1 { "" } else { "s" },
            app.backup_count
        ),
        Style::default().fg(Color::Yellow),
    );
    let ini = Span::styled(
        format!(
            " .ini backup: {} ",
            if app.has_ini_backup { "yes" } else { "no" }
        ),
        Style::default().fg(if app.has_ini_backup {
            Color::Green
        } else {
            Color::DarkGray
        }),
    );

    let line = Line::from(vec![
        Span::raw("  "),
        live,
        Span::raw("  "),
        bak,
        Span::raw("  "),
        ini,
    ]);

    f.render_widget(Paragraph::new(line), area);
}

/// Render a scrollable picker list with description and prompt.
#[allow(unused)]
fn draw_select_list(
    f: &mut Frame,
    area: Rect,
    items: &[&str],
    descs: &[&str],
    prompt: &str,
    state: &mut ListState,
) {
    let list_area = Rect {
        height: area.height.saturating_sub(1),
        ..area
    };

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(Span::raw(*item)).style(Style::default()))
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ")
        .repeat_highlight_symbol(true);

    f.render_stateful_widget(list, list_area, state);

    // Description line for the highlighted item
    let desc_idx = state
        .selected()
        .unwrap_or(0)
        .min(descs.len().saturating_sub(1));
    let desc = descs.get(desc_idx).copied().unwrap_or("");
    let desc_line = Paragraph::new(Span::styled(
        format!("  {desc}"),
        Style::default().fg(Color::DarkGray),
    ));

    f.render_widget(
        desc_line,
        Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        },
    );

    // Prompt at bottom-right
    let prompt_len = prompt.len() as u16;
    if area.width > prompt_len + 2 {
        let prompt_p = Paragraph::new(Span::styled(prompt, Style::default().fg(Color::DarkGray)))
            .alignment(Alignment::Right);
        f.render_widget(
            prompt_p,
            Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(1),
                width: area.width.saturating_sub(2),
                height: 1,
            },
        );
    }
}

/// Render a picker list with description and prompt.
fn draw_select_list_with_info(
    f: &mut Frame,
    area: Rect,
    items: &[&str],
    descs: &[&str],
    prompt: &str,
    state: &mut ListState,
    pinned_header: bool,
) {
    // If pinned_header is set, items[0] (header) and items[1] (blank spacer)
    // render as fixed rows above the scrollable list. items[2..] form the list.
    let (list_start_y, list_items_slice): (u16, &[&str]) = if pinned_header && !items.is_empty() {
        let header_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        f.render_widget(
            Paragraph::new(Span::styled(items[0], header_style)),
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            },
        );
        // Leave items[1] (blank spacer) as visual gap — rendered as empty row
        (area.y + 2, &items[2..])
    } else {
        (area.y, items)
    };

    let list_area = Rect {
        y: list_start_y,
        height: area.height.saturating_sub(2 + (list_start_y - area.y)),
        ..area
    };

    // Offset: when pinned_header is true, the list widget only sees
    // items[2..], but state.selected() is absolute to the full items array.
    // Adjust the state so the list widget highlights the correct entry.
    let header_offset = if pinned_header && !items.is_empty() {
        2u16
    } else {
        0u16
    };
    let orig_selected = state.selected();
    if header_offset > 0 {
        if let Some(s) = orig_selected {
            state.select(Some(s.saturating_sub(header_offset as usize)));
        }
    }

    let list_items: Vec<ListItem> = list_items_slice
        .iter()
        .map(|item| ListItem::new(Span::raw(*item)).style(Style::default()))
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ")
        .repeat_highlight_symbol(true);

    f.render_stateful_widget(list, list_area, state);

    // Restore state to absolute indexing for the caller
    if header_offset > 0 {
        let s = state.selected().unwrap_or(0);
        state.select(Some(s + header_offset as usize));
    }

    // Description line — use original (absolute) selection index
    let base_y = area.y + area.height.saturating_sub(1);
    let desc_idx = orig_selected
        .unwrap_or(0)
        .min(descs.len().saturating_sub(1));
    let desc = descs.get(desc_idx).copied().unwrap_or("");
    let desc_line = Paragraph::new(Span::styled(
        format!("  {desc}"),
        Style::default().fg(Color::DarkGray),
    ));

    f.render_widget(
        desc_line,
        Rect {
            x: area.x,
            y: base_y,
            width: area.width,
            height: 1,
        },
    );

    let prompt_len = prompt.len() as u16;
    if area.width > prompt_len + 2 {
        let prompt_p = Paragraph::new(Span::styled(prompt, Style::default().fg(Color::DarkGray)))
            .alignment(Alignment::Right);
        f.render_widget(
            prompt_p,
            Rect {
                x: area.x,
                y: base_y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
        );
    }
}

// ── pip-list renderer (for split-layout file picker) ─────────────────────

/// Render a compact pip-list without description line or prompt.
/// The pip (►) replaces the full-row background highlight.
fn draw_select_list_pip(f: &mut Frame, area: Rect, items: &[&str], state: &mut ListState) {
    if area.height < 2 || area.width < 10 {
        return;
    }

    let dim_val = Style::default().fg(Color::Rgb(160, 160, 160));

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == 0 {
                // Header row — match right pane header color
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if i >= 2 {
                // Data rows — match right pane value color
                dim_val
            } else {
                Style::default()
            };
            ListItem::new(Span::raw(*item)).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ")
        .repeat_highlight_symbol(true);

    f.render_stateful_widget(list, area, state);
}

// ── right-pane metadata panel ────────────────────────────────────────────

/// Render the right-hand metadata pane in the split file picker.
/// Shows the filename header, a dim separator, then the provided content lines.
/// When `meta_lines` is empty, shows a placeholder message.
fn draw_right_pane(f: &mut Frame, area: Rect, filename: &str, meta_lines: &[Line]) {
    if area.height < 3 || area.width < 10 {
        return;
    }

    let dim = Style::default().fg(Color::Rgb(160, 160, 160));

    // Filename header
    let mut y = area.y;
    let fname = if filename.len() as u16 > area.width.saturating_sub(2) {
        format!(
            "{}…",
            &filename[..area.width.saturating_sub(3).max(1) as usize]
        )
    } else {
        filename.to_string()
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            &fname,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Rect {
            x: area.x + 1,
            y,
            width: area.width.saturating_sub(2),
            height: 1,
        },
    );
    y += 2;

    if meta_lines.is_empty() {
        // Placeholder
        let msg = if filename.is_empty() {
            Span::styled("select a file", dim)
        } else {
            Span::styled("select to load metadata", dim)
        };
        f.render_widget(
            Paragraph::new(msg),
            Rect {
                x: area.x + 1,
                y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
        );
        return;
    }

    // Content lines
    let max_lines = area.height.saturating_sub(2) as usize;
    for (i, line) in meta_lines.iter().enumerate().take(max_lines) {
        f.render_widget(
            Paragraph::new(line.clone()),
            Rect {
                x: area.x + 1,
                y: y + i as u16,
                width: area.width.saturating_sub(2),
                height: 1,
            },
        );
    }
}

// ── split-layout picker entry point ──────────────────────────────────────

/// Draw the file picker with a horizontal split: pip-style file list on the
/// left, live metadata preview on the right.  Used by the .bak recover flow.
pub fn draw_picker_split(
    f: &mut Frame,
    app: &AppState,
    items: &[&str],
    _descs: &[&str],
    state: &mut ListState,
    selected_info: Option<&str>,
    meta_lines: &[Line],
) {
    let chunks = standard_layout(f.area(), items.len());
    draw_header(f, chunks[0], app);

    let menu_area = chunks[2];
    let prompt = "↑/↓ navigate | Enter select | Esc cancel";

    // Reserve bottom row of menu area for the prompt
    let prompt_y = menu_area.y + menu_area.height.saturating_sub(1);
    let content_area = Rect {
        height: menu_area.height.saturating_sub(1),
        ..menu_area
    };

    // Split content into left (60%) and right (40%)
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(content_area);

    // Left: pip list (full height of content area)
    draw_select_list_pip(f, halves[0], items, state);

    // Right: metadata pane
    draw_right_pane(f, halves[1], selected_info.unwrap_or(""), meta_lines);

    // Prompt at bottom-right of the full menu area
    let prompt_len = prompt.len() as u16;
    if menu_area.width > prompt_len + 2 {
        f.render_widget(
            Paragraph::new(Span::styled(prompt, Style::default().fg(Color::DarkGray)))
                .alignment(Alignment::Right),
            Rect {
                x: menu_area.x,
                y: prompt_y,
                width: menu_area.width.saturating_sub(2),
                height: 1,
            },
        );
    }

    draw_status_bar(f, chunks[4], app);
}

/// Render the status bar at the bottom of the screen.
fn draw_status_bar(f: &mut Frame, area: Rect, app: &AppState) {
    draw_whale_separator(f, area, app);
}

/// Draw the bottom status bar with an animated whale patrolling right-to-left.
/// The whale moves one position every 180ms.  After reaching the left edge,
/// it disappears for ~5.4s (30 cooldown ticks) before reappearing on the
/// right.  Two variants alternate every 400ms.
pub fn draw_whale_separator(f: &mut Frame, area: Rect, app: &AppState) {
    if area.width < 4 {
        return;
    }
    let elapsed = app.whale_start.elapsed().as_millis() as u64;
    let bar_w = area.width as u64;
    let speed_ms: u64 = 180;
    let cooldown_ticks: u64 = 30;
    let total = bar_w + cooldown_ticks;
    let t = (elapsed / speed_ms) % total;
    if t < bar_w {
        let x = bar_w - t - 1;
        let switch = ((elapsed / 400) * 7 + (elapsed / 600) * 13) % 2;
        let variants: &[&str] = &["🐋", "🐳"];
        let whale = variants[(switch % variants.len() as u64) as usize];
        f.render_widget(
            Paragraph::new(Span::styled(whale, Style::default().fg(Color::Cyan))),
            Rect {
                x: area.x + (x as u16).min(area.width.saturating_sub(4)),
                y: area.y,
                width: 4,
                height: 1,
            },
        );
    }
}

// ── input dialog ─────────────────────────────────────────────────────────────

/// State for the text-input dialog used to enter a save-folder path.
pub struct InputDialogState {
    pub input: String,
    pub cursor: usize,
    pub prompt: String,
    pub confirmed: bool,
    pub cancelled: bool,
}

impl InputDialogState {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            prompt: prompt.into(),
            confirmed: false,
            cancelled: false,
        }
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.confirmed = false;
        self.cancelled = false;
    }

    /// Insert a character at the cursor position.
    pub fn insert(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Insert a string at the cursor position (for paste).
    pub fn insert_str(&mut self, s: &str) {
        self.input.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    /// Delete the character before the cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev = self.cursor.saturating_sub(1);
            self.input.remove(prev);
            self.cursor = prev;
        }
    }

    /// Delete the character at the cursor (Delete).
    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    /// Move cursor left by one grapheme boundary.
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.input[..self.cursor]
                .char_indices()
                .nth_back(0)
                .map(|(i, _c)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right by one grapheme boundary.
    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor = self.input[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _c)| self.cursor + i)
                .unwrap_or(self.input.len());
        }
    }
}

/// Draw the text input dialog for entering a custom save-folder path.
///
/// Layout: title bar, prompt, input line with cursor indicator,
/// instruction line, and [ OK ] [ Cancel ] buttons.
pub fn draw_input_dialog(
    f: &mut Frame,
    _app: &AppState,
    state: &InputDialogState,
    ok_selected: bool,
) {
    let prompt_w = state.prompt.len() as u16 + 4;
    let input_display = &state.input;
    let display_w = input_display.len() + 4; // rough, but good enough for sizing
    let popup_w =
        (prompt_w.max(display_w as u16).max(40) + 4).min(f.area().width.saturating_sub(4));
    let popup_h = 10u16.min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, area);

    let inner = inner(area, 2, 1);

    // Title
    f.render_widget(
        Paragraph::new(Span::styled(
            "Set Save Folder",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Rect { height: 1, ..inner },
    );

    // Prompt text
    f.render_widget(
        Paragraph::new(Span::styled(
            &state.prompt,
            Style::default().fg(Color::White),
        )),
        Rect {
            y: inner.y + 2,
            height: 1,
            width: inner.width,
            x: inner.x,
        },
    );

    // Input line with cursor
    let cursor_visible = (std::time::Instant::now().elapsed().as_millis() / 500).is_multiple_of(2);
    let mut input_spans = vec![Span::styled("  ", Style::default())];
    // Show the text up to cursor
    let before = &state.input[..state.cursor.min(state.input.len())];
    let after = if state.cursor < state.input.len() {
        Some(&state.input[state.cursor..])
    } else {
        None
    };
    input_spans.push(Span::styled(before, Style::default().fg(Color::White)));
    if cursor_visible && !state.confirmed && !state.cancelled {
        input_spans.push(Span::styled("█", Style::default().fg(Color::Cyan)));
    }
    if let Some(a) = after {
        input_spans.push(Span::styled(a, Style::default().fg(Color::White)));
    }
    f.render_widget(
        Paragraph::new(Line::from(input_spans)),
        Rect {
            y: inner.y + 3,
            height: 1,
            width: inner.width,
            x: inner.x,
        },
    );

    // Instruction line
    f.render_widget(
        Paragraph::new(Span::styled(
            "Type a path, then Tab to buttons  Enter to confirm  Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
        Rect {
            y: inner.y + 5,
            height: 1,
            width: inner.width,
            x: inner.x,
        },
    );

    // OK / Cancel buttons
    let ok_style = if ok_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let cancel_style = if !ok_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    let buttons = Line::from(vec![
        Span::styled(" [ OK ] ", ok_style),
        Span::raw("  "),
        Span::styled(" [ Cancel ] ", cancel_style),
    ]);
    f.render_widget(
        Paragraph::new(buttons).alignment(Alignment::Center),
        Rect {
            y: inner.y + 6,
            height: 1,
            width: inner.width,
            x: inner.x,
        },
    );

    // Whale
    let bar = Rect {
        x: 0,
        y: f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };
    draw_whale_separator(f, bar, _app);
}

/// Truncate a filesystem path to show only the most specific directories.
/// Walks forward to the first path separator after the truncation point to
/// avoid splitting mid-component.
///
/// Examples:
/// - `C:\Users\user\AppData\...\SaveGames` → `…\AppData\...\SaveGames`
/// - A short path that fits `max_width` is returned unchanged.
fn truncate_path_tail(path: &str, max_width: usize) -> String {
    if path.len() <= max_width {
        return path.to_string();
    }
    let keep = max_width.saturating_sub(3);
    if keep == 0 {
        return "…".to_string();
    }
    let tail = &path[path.len().saturating_sub(keep)..];
    // Walk forward to a path separator so we don't split mid-component
    if let Some(sep_pos) = tail.find(&['\\', '/'][..]) {
        let start = path.len().saturating_sub(keep) + sep_pos;
        format!("…{}", &path[start..])
    } else {
        format!("…{}", tail)
    }
}

/// Create a centered rectangle for modal overlays.
/// Return a rectangle centered in `r` by the given width and height percentages.
/// Return a rectangle centered in `r` by the given width and height percentages.
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

/// Shrink a rect by a margin on all sides.
fn inner(rect: Rect, margin_x: u16, margin_y: u16) -> Rect {
    Rect {
        x: rect.x + margin_x,
        y: rect.y + margin_y,
        width: rect.width.saturating_sub(margin_x * 2),
        height: rect.height.saturating_sub(margin_y * 2),
    }
}
