// Render module - TUI layout and widgets

use crate::app::{App, AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Render the application UI
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Status bar
            Constraint::Min(10),   // List
            Constraint::Length(3), // Summary
            Constraint::Length(2), // Keybinds
        ])
        .split(frame.area());

    render_header(frame, chunks[0]);
    render_status(frame, chunks[1], app);
    render_list(frame, chunks[2], app);
    render_summary(frame, chunks[3], app);
    render_keybinds(frame, chunks[4]);

    // Overlay help if shown
    if app.show_help {
        render_help_overlay(frame);
    }

    // Overlay confirm dialog
    if app.state == AppState::Confirming {
        render_confirm_dialog(frame, app);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "claudekill ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(concat!("v", env!("CARGO_PKG_VERSION"))),
        Span::raw("                                        "),
        Span::styled("[?] Help  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q] Quit", Style::default().fg(Color::DarkGray)),
    ])])
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let status_text = match app.state {
        AppState::Scanning => {
            let path = app
                .scan_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let truncated = if path.len() > 40 {
                format!("...{}", &path[path.len() - 37..])
            } else {
                path
            };
            format!(
                "Scanning: {:40}           Found: {}",
                truncated,
                app.folders.len()
            )
        }
        AppState::Browsing => {
            if let Some(msg) = &app.message {
                msg.clone()
            } else {
                format!(
                    "Scan complete                                     Found: {}",
                    app.folders.len()
                )
            }
        }
        AppState::Confirming | AppState::Deleting | AppState::Done => {
            app.message.clone().unwrap_or_default()
        }
    };

    let color = match app.state {
        AppState::Scanning => Color::Yellow,
        AppState::Browsing => Color::Green,
        AppState::Confirming => Color::Magenta,
        AppState::Deleting => Color::Red,
        AppState::Done => Color::Green,
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(color))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}

fn render_list(frame: &mut Frame, area: Rect, app: &App) {
    let home = dirs::home_dir();

    let items: Vec<ListItem> = app
        .folders
        .iter()
        .enumerate()
        .map(|(i, folder)| {
            // Check if this is the global ~/.claude folder
            let is_global = home
                .as_ref()
                .map(|h| folder.path == h.join(".claude"))
                .unwrap_or(false);

            let selected_marker = if folder.selected { "●" } else { " " };
            let size = format!("{:>10}", folder.size_display());

            // Truncate path to fit
            let path = folder.path.display().to_string();
            let max_path_len = if is_global { 38 } else { 45 };
            let display_path = if path.len() > max_path_len {
                format!("...{}", &path[path.len() - max_path_len + 3..])
            } else {
                path
            };

            // Add warning for global folder
            let project_type = if is_global {
                format!("{} ⚠GLOBAL", folder.project_type)
            } else {
                folder.project_type.clone()
            };

            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else if is_global {
                Style::default().fg(Color::Red)
            } else if folder.selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            let content = format!(
                "{} {} {:48} {:10}",
                selected_marker, size, display_path, project_type
            );

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" .claude folders "),
    );

    frame.render_widget(list, area);
}

fn render_summary(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app.selected_count();
    let selected_size = crate::utils::format_size(app.selected_size());
    let total_size = crate::utils::format_size(app.total_size());

    let summary = Paragraph::new(format!(
        "Selected: {} ({})                               Total: {}",
        selected, selected_size, total_size
    ))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(summary, area);
}

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let keybinds = Paragraph::new(
        "[Space] Toggle   [a] All   [n] None   [d] Delete   [↑↓/jk] Navigate   [q] Quit",
    )
    .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(keybinds, area);
}

fn render_help_overlay(frame: &mut Frame) {
    let area = centered_rect(60, 60, frame.area());

    let help_text = vec![
        "",
        "  Keyboard Shortcuts",
        "  ──────────────────",
        "",
        "  ↑/k        Move selection up",
        "  ↓/j        Move selection down",
        "  PgUp/g     Page up / Go to top",
        "  PgDn/G     Page down / Go to bottom",
        "  Home/End   Jump to first/last",
        "  Space      Toggle folder selection",
        "  a          Select all folders",
        "  n          Deselect all folders",
        "  d          Delete selected folders",
        "  ?          Toggle this help",
        "  q/Esc      Quit",
        "",
        "  Press any key to close",
        "",
    ];

    let help = Paragraph::new(help_text.join("\n")).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn render_confirm_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, frame.area());

    let count = app.selected_count();
    let size = crate::utils::format_size(app.selected_size());

    let (method, warning) = if app.permanent_delete {
        ("PERMANENTLY DELETE", "⚠ This cannot be undone!")
    } else {
        ("Move to Trash", "You can restore from Trash later.")
    };

    // Build folder list preview (show first 5)
    let selected_folders: Vec<String> = app
        .get_selected_folders()
        .iter()
        .take(5)
        .map(|f| {
            let path = f.path.display().to_string();
            if path.len() > 50 {
                format!("  • ...{}", &path[path.len() - 47..])
            } else {
                format!("  • {}", path)
            }
        })
        .collect();

    let mut text = vec![
        String::new(),
        format!("  {} {} folder(s) ({})", method, count, size),
        String::new(),
    ];

    text.extend(selected_folders);

    if count > 5 {
        text.push(format!("  ... and {} more", count - 5));
    }

    text.push(String::new());
    text.push(format!("  {}", warning));
    text.push(String::new());
    text.push("  Confirm? [y/N]".to_string());
    text.push(String::new());

    let color = if app.permanent_delete {
        Color::Red
    } else {
        Color::Yellow
    };

    let dialog = Paragraph::new(text.join("\n")).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm Deletion ")
            .style(Style::default().bg(Color::Black).fg(color)),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(dialog, area);
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
