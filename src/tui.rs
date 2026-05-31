//! Modern terminal UI built on ratatui + crossterm.
//!
//! Design principles:
//! - Dashboard layout: header bar, main panel, status line
//! - Keyboard-first: arrow keys + Enter/Esc, no mouse dependency
//! - Semantic color: cyan=info, green=success, yellow=warning, red=error
//! - Adapts to terminal size; minimum 60×15

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
    /// Version string for the header
    pub version: String,
    /// Last operation result (for the status bar)
    pub status_message: Option<String>,
    pub status_style: StatusStyle,
    /// Spinner state
    pub spinner_active: bool,
    pub spinner_start: Option<Instant>,
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
            version: String::new(),
            status_message: None,
            status_style: StatusStyle::Neutral,
            spinner_active: false,
            spinner_start: None,
        }
    }
}

// ── public rendering entry points ──────────────────────────────────────────

/// Draw the main menu.  Hides the locate item when saves are already found.
pub fn draw_main_menu(f: &mut Frame, state: &mut ListState, app: &AppState, save_found: bool) {
    let items: Vec<&str> = if save_found {
        vec![
            "  Recover .sav file from .bak",
            "  Create full backup",
            "  Restore full backup",
            "  Inspect save files",
            "  Manage UE5 Config (.ini) files",
            "  View disclaimer",
            "  Exit",
        ]
    } else {
        vec![
            "  Locate Subnautica save files",
            "  Recover .sav file from .bak",
            "  Create full backup",
            "  Restore full backup",
            "  Inspect save files",
            "  Manage UE5 Config (.ini) files",
            "  View disclaimer",
            "  Exit",
        ]
    };
    let descs: Vec<&str> = if save_found {
        vec![
            "Restore a .sav file from its .bak backup",
            "Copy the savegame files to NotAlterra_Backups",
            "Restore a full backup from NotAlterra_Backups",
            "View detailed GVAS metadata for each save file",
            "Backup, restore, or delete .ini files in Config/Windows",
            "Re-read the disclaimer and terms of use",
            "Close NotAlterra",
        ]
    } else {
        vec![
            "Scan all drives for Subnautica 2 save folders",
            "Restore a .sav file from its .bak backup",
            "Copy the savegame files to NotAlterra_Backups",
            "Restore a full backup from NotAlterra_Backups",
            "View detailed GVAS metadata for each save file",
            "Backup, restore, or delete .ini files in Config/Windows",
            "Re-read the disclaimer and terms of use",
            "Close NotAlterra",
        ]
    };
    let chunks = standard_layout(f.area(), items.len());

    draw_header(f, chunks[0], app);
    draw_status_dashboard(f, chunks[1], app);

    let prompt = "↑/↓ navigate  Enter select";
    draw_select_list(f, chunks[2], &items, &descs, prompt, state);

    // Separator line
    let sep_line = "─".repeat(chunks[3].width as usize);
    f.render_widget(
        Paragraph::new(Span::styled(sep_line, Style::default().fg(Color::DarkGray))),
        chunks[3],
    );

    draw_status_bar(f, chunks[4], app);
}

/// Draw the disclaimer popup with full warning text.
pub fn draw_disclaimer_popup(f: &mut Frame, _app: &AppState, selected_yes: bool) {
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
        Line::from(Span::styled("DISCLAIMER", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("This tool was created using an AI Agent. While", Style::default().fg(Color::White))),
        Line::from(Span::styled("every effort has been made to ensure it works", Style::default().fg(Color::White))),
        Line::from(Span::styled("correctly, you should review the code and test", Style::default().fg(Color::White))),
        Line::from(Span::styled("on a backup before using it on live save files.", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("NotAlterra is not affiliated with Unknown Worlds", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("Entertainment or KRAFTON. Use at your own risk.", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled("The author is NOT responsible for any data loss.", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
    ];
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), Rect { height: 11, ..inner });

    let yes_style = if selected_yes { Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Green) };
    let no_style = if !selected_yes { Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Red) };
    let buttons = Line::from(vec![Span::styled("[ Accept ]", yes_style), Span::raw("    "), Span::styled("[ Decline ]", no_style)]);
    f.render_widget(Paragraph::new(buttons).alignment(Alignment::Center), Rect { y: inner.y + 12, height: 1, ..inner });
}

/// Draw a simple confirmation popup with [ Yes ] [ No ] buttons.
pub fn draw_confirm_popup(
    f: &mut Frame,
    _app: &AppState,
    title: &str,
    details: &[(&str, &str)],
    selected_yes: bool,
) {
    let max_w = details.iter().map(|(k, v)| k.len() + v.len() + 4).max().unwrap_or(20).max(30) as u16;
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
        Paragraph::new(Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center),
        Rect { height: 1, ..inner },
    );

    // Details
    let detail_lines: Vec<Line> = details.iter().map(|(k, v)| {
        let icon = if k.starts_with('⚠') { Color::Yellow } else { Color::Gray };
        Line::from(vec![
            Span::styled(format!("{k}: "), Style::default().fg(icon)),
            Span::styled(*v, Style::default()),
        ])
    }).collect();
    f.render_widget(
        Paragraph::new(detail_lines),
        Rect { y: inner.y + 2, height: details.len() as u16, ..inner },
    );

    // Yes / No buttons
    let yes_style = if selected_yes { Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD) }
        else { Style::default().fg(Color::Green) };
    let no_style = if !selected_yes { Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD) }
        else { Style::default().fg(Color::Red) };
    let buttons = Line::from(vec![
        Span::styled("[ Yes ]", yes_style),
        Span::raw("    "),
        Span::styled("[ No ]", no_style),
    ]);
    f.render_widget(
        Paragraph::new(buttons).alignment(Alignment::Center),
        Rect { y: inner.y + inner.height.saturating_sub(1), height: 1, ..inner },
    );
}

pub fn draw_ok_dialog(f: &mut Frame, _app: &AppState, title: &str, message: &str) {
    let content_w = message.lines().map(|l| l.len()).max().unwrap_or(20).max(title.len()) as u16 + 10;
    let popup_w = content_w.max(50).min(f.area().width.saturating_sub(4));
    let popup_h = (message.lines().count() as u16 + 7).min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).border_type(BorderType::Plain).border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
    let inner = inner(area, 2, 1);
    f.render_widget(Paragraph::new(Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).alignment(Alignment::Center), Rect { height: 1, ..inner });
    let msg_h = message.lines().count() as u16;
    f.render_widget(Paragraph::new(message.to_string()).style(Style::default().fg(Color::Gray)).alignment(Alignment::Left), Rect { x: inner.x + 2, y: inner.y + 2, width: inner.width.saturating_sub(4), height: msg_h });
    let ok = Span::styled("[ OK ]", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(Paragraph::new(ok).alignment(Alignment::Center), Rect { y: inner.y + inner.height.saturating_sub(2), height: 1, ..inner });
}

pub fn draw_ok_dialog_styled(f: &mut Frame, _app: &AppState, title: &str, lines: &[Line]) {
    let content_w = lines.iter().map(|l| l.width() as u16).max().unwrap_or(20).max(title.len() as u16) + 10;
    let popup_w = content_w.max(50).min(f.area().width.saturating_sub(4));
    let popup_h = (lines.len() as u16 + 7).min(f.area().height.saturating_sub(4));
    let area = centered_rect_size(popup_w, popup_h, f.area());
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).border_type(BorderType::Plain).border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
    let inner = inner(area, 2, 1);
    f.render_widget(Paragraph::new(Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).alignment(Alignment::Center), Rect { height: 1, ..inner });
    f.render_widget(Paragraph::new(lines.to_vec()).style(Style::default()).alignment(Alignment::Left), Rect { x: inner.x + 2, y: inner.y + 2, width: inner.width.saturating_sub(4), height: lines.len() as u16 });
    let ok = Span::styled("[ OK ]", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(Paragraph::new(ok).alignment(Alignment::Center), Rect { y: inner.y + inner.height.saturating_sub(2), height: 1, ..inner });
}

fn centered_rect_size(w: u16, h: u16, r: Rect) -> Rect {
    let popup = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length((r.height.saturating_sub(h))/2), Constraint::Length(h), Constraint::Length((r.height.saturating_sub(h))/2)])
        .split(r);
    Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Length((r.width.saturating_sub(w))/2), Constraint::Length(w), Constraint::Length((r.width.saturating_sub(w))/2)])
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
        title,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(title_p, chunks[1]);

    draw_select_list(f, chunks[2], items, descs, "↑/↓ navigate  Enter select  Esc back", state);

    // Separator
    let sep_line = "─".repeat(chunks[3].width as usize);
    f.render_widget(
        Paragraph::new(Span::styled(sep_line, Style::default().fg(Color::DarkGray))),
        chunks[3],
    );

    draw_status_bar(f, chunks[4], app);
}

/// Draw a simple text screen with a "press any key" prompt.
pub fn draw_text_screen(
    f: &mut Frame,
    app: &AppState,
    lines: &[Line],
    prompt: &str,
) {
    let chunks = standard_layout(f.area(), lines.len());
    draw_header(f, chunks[0], app);

    f.render_widget(Paragraph::new(lines.to_vec()), chunks[2]);

    let prompt_p = Paragraph::new(Span::styled(
        prompt,
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Center);
    // Separator
    let sep_line = "─".repeat(chunks[3].width as usize);
    f.render_widget(
        Paragraph::new(Span::styled(sep_line, Style::default().fg(Color::DarkGray))),
        chunks[3],
    );
    f.render_widget(prompt_p, chunks[4]);
}

/// Draw a file/folder picker list.
pub fn draw_picker(
    f: &mut Frame,
    app: &AppState,
    items: &[&str],
    descs: &[&str],
    state: &mut ListState,
) {
    draw_picker_with_info(f, app, items, descs, state, None);
}

/// Draw a file/folder picker list with an extra selected-item info line
/// (e.g. showing the full filename of the highlighted .bak file).
pub fn draw_picker_with_info(
    f: &mut Frame,
    app: &AppState,
    items: &[&str],
    descs: &[&str],
    state: &mut ListState,
    selected_info: Option<&str>,
) {
    let chunks = standard_layout(f.area(), items.len());
    draw_header(f, chunks[0], app);

    let prompt = "↑/↓ navigate  Enter select  Esc cancel";
    draw_select_list_with_info(f, chunks[2], items, descs, prompt, state, selected_info);

    // Separator
    let sep_line = "─".repeat(chunks[3].width as usize);
    f.render_widget(
        Paragraph::new(Span::styled(sep_line, Style::default().fg(Color::DarkGray))),
        chunks[3],
    );

    draw_status_bar(f, chunks[4], app);
}

// ── internal drawing helpers ───────────────────────────────────────────────

fn standard_layout(area: Rect, menu_items: usize) -> Vec<Rect> {
    let menu_height = menu_items as u16 + 4; // items + gaps + prompt line
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                          // header
            Constraint::Length(2),                          // dashboard
            Constraint::Min(menu_height.min(area.height.saturating_sub(7))), // menu
            Constraint::Length(1),                          // separator
            Constraint::Length(1),                          // status bar
        ])
        .split(area).to_vec()
}

fn draw_header(f: &mut Frame, area: Rect, app: &AppState) {
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Cyan));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(0),
        ])
        .split(inner(area, 1, 0));

    let title_line = Line::from(vec![
        Span::styled("NOTALTERRA", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(app.version.clone(), Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    let path_line = if let Some(ref path) = app.save_path {
        let max_w = chunks[1].width.saturating_sub(2) as usize;
        let display = truncate_path_tail(path, max_w);
        Paragraph::new(Span::styled(display, Style::default().fg(Color::Gray)))
            .alignment(Alignment::Right)
    } else {
        Paragraph::new(Span::styled(
            "no save folder selected",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Right)
    };
    f.render_widget(path_line, chunks[1]);
    f.render_widget(header_block, area);
}

fn draw_status_dashboard(f: &mut Frame, area: Rect, app: &AppState) {
    let live = Span::styled(
        format!(" Save: {} ", if app.save_path.is_some() { app.live_save_count.to_string() } else { "—".into() }),
        Style::default().fg(Color::Green),
    );
    let bak = Span::styled(
        format!(" Backups: {} ", app.backup_count),
        Style::default().fg(Color::Yellow),
    );
    let ini = Span::styled(
        format!(" .ini backup: {} ", if app.has_ini_backup { "yes" } else { "no" }),
        Style::default().fg(if app.has_ini_backup { Color::Green } else { Color::DarkGray }),
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
        .map(|item| {
            ListItem::new(Span::raw(*item))
                .style(Style::default())
        })
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ")
        .repeat_highlight_symbol(true);

    f.render_stateful_widget(list, list_area, state);

    // Description line for the highlighted item
    let desc_idx = state.selected().unwrap_or(0).min(descs.len().saturating_sub(1));
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
        let prompt_p = Paragraph::new(Span::styled(
            prompt,
            Style::default().fg(Color::DarkGray),
        ))
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

fn draw_select_list_with_info(
    f: &mut Frame,
    area: Rect,
    items: &[&str],
    descs: &[&str],
    prompt: &str,
    state: &mut ListState,
    selected_info: Option<&str>,
) {
    let extra = if selected_info.is_some() { 1u16 } else { 0u16 };
    let list_area = Rect {
        height: area.height.saturating_sub(1 + extra),
        ..area
    };

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| {
            ListItem::new(Span::raw(*item))
                .style(Style::default())
        })
        .collect();

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ")
        .repeat_highlight_symbol(true);

    f.render_stateful_widget(list, list_area, state);

    // Description line for the highlighted item
    let base_y = area.y + area.height.saturating_sub(1 + extra);
    let desc_idx = state.selected().unwrap_or(0).min(descs.len().saturating_sub(1));
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

    // Selected-item info line (e.g. filename)
    if let Some(info) = selected_info {
        let info_line = Paragraph::new(Span::styled(
            format!("  → {info}"),
            Style::default().fg(Color::White),
        ));
        f.render_widget(
            info_line,
            Rect {
                x: area.x,
                y: base_y + 1,
                width: area.width,
                height: 1,
            },
        );
    }

    // Prompt at bottom-right
    let prompt_len = prompt.len() as u16;
    if area.width > prompt_len + 2 {
        let prompt_p = Paragraph::new(Span::styled(
            prompt,
            Style::default().fg(Color::DarkGray),
        ))
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

fn draw_status_bar(f: &mut Frame, area: Rect, app: &AppState) {
    if let Some(ref msg) = app.status_message {
        let color = match app.status_style {
            StatusStyle::Success => Color::Green,
            StatusStyle::Warning => Color::Yellow,
            StatusStyle::Error => Color::Red,
            StatusStyle::Info => Color::Cyan,
            StatusStyle::Neutral => Color::Gray,
        };

        let icon = match app.status_style {
            StatusStyle::Success => "√",
            StatusStyle::Warning => "!",
            StatusStyle::Error => "×",
            StatusStyle::Info => "i",
            StatusStyle::Neutral => " ",
        };

        let line = Line::from(vec![
            Span::styled(
                format!(" [{icon}] {msg}"),
                Style::default().fg(color),
            ),
        ]);
        f.render_widget(Paragraph::new(line), area);
    }

}

/// Truncate a path to show the tail (most specific directories).
/// e.g. `C:\Users\...\Subnautica2\Saved\SaveGames` → `…\Subnautica2\Saved\SaveGames`
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
