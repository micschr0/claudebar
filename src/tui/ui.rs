//! Drawing: six-zone single-column flat-list layout.
//! No state mutation happens here (Cell<u16> excepted).

use crate::tui::app::{App, RowItem, StatusKind, ThresholdField};
use crate::tui::preview;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

// ── Chrome color constants (Rgb — immune to 256-color palette remapping) ──────

const CHROME_BG: Color = Color::Rgb(26, 27, 38); // Tokyo Night bg
const CHROME_SURFACE: Color = Color::Rgb(36, 40, 59); // cursor row highlight
const CHROME_ACCENT: Color = Color::Rgb(122, 162, 247); // title, reorder mode header
const CHROME_HEADER: Color = Color::Rgb(169, 177, 214); // section headers, hint text
const CHROME_TEXT: Color = Color::Rgb(192, 202, 245); // primary list content
const CHROME_OK: Color = Color::Rgb(158, 206, 106); // enabled bullet, save success
const CHROME_WARN: Color = Color::Rgb(224, 175, 104); // dirty dot, pending banners
const CHROME_CRIT: Color = Color::Rgb(247, 118, 142); // errors, disabled call-to-action
const CHROME_DISABLED: Color = Color::Rgb(86, 95, 137); // disabled labels, section fills
const CHROME_KEY_BG: Color = Color::Rgb(65, 72, 104); // hint line [x] button bg

/// Top-level draw entrypoint. Returns the list viewport height for Cell update.
pub fn draw(f: &mut Frame, app: &App) {
    // ── Size guard ────────────────────────────────────────────────────────────
    if f.area().width < 60 || f.area().height < 18 {
        f.render_widget(
            Paragraph::new("Terminal too small (min 60×18)").style(
                Style::default()
                    .fg(CHROME_CRIT)
                    .add_modifier(Modifier::BOLD),
            ),
            f.area(),
        );
        app.list_viewport_height.set(10);
        return;
    }

    // ── Six vertical zones ────────────────────────────────────────────────────
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Zone 1: title bar
            Constraint::Min(8),    // Zone 2: scrollable list
            Constraint::Length(1), // Zone 3: description
            Constraint::Length(3), // Zone 4: preview
            Constraint::Length(1), // Zone 5: status
            Constraint::Length(1), // Zone 6: hint
        ])
        .split(f.area());

    let title_area = chunks[0];
    let list_area = chunks[1];
    let desc_area = chunks[2];
    let preview_area = chunks[3];
    let status_area = chunks[4];
    let hint_area = chunks[5];

    draw_title(f, app, title_area);
    draw_list(f, app, list_area);
    draw_desc(f, app, desc_area);
    draw_preview(f, app, preview_area);
    draw_status(f, app, status_area);
    draw_hint(f, app, hint_area);

    // ── Help overlay (zones 2+3 combined) ─────────────────────────────────────
    if app.show_help {
        let overlay_area = Rect::new(
            list_area.x,
            list_area.y,
            list_area.width,
            list_area.height + desc_area.height,
        );
        draw_help_overlay(f, overlay_area);
    }
}

// ── Zone 1: Title Bar ─────────────────────────────────────────────────────────

fn draw_title(f: &mut Frame, app: &App, area: Rect) {
    // Background fill.
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(CHROME_BG)),
        area,
    );

    let title_str = "  claudebar"; // 11 chars
    let sep_str = " — "; // 3 chars
    let help_str = "? help"; // 6 chars

    // Center section: path or "(no path)".
    let (center_str, center_style) = match &app.save_path {
        Some(path) => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            // Left-truncate if too long.
            let max_center = (area.width as usize).saturating_sub(11 + 3 + 2 + 6); // title+sep+dirty+help
            let s = if name.chars().count() > max_center && max_center > 3 {
                let skip = name.chars().count() - (max_center - 3);
                format!("...{}", name.chars().skip(skip).collect::<String>())
            } else {
                name
            };
            (
                s,
                Style::default()
                    .fg(CHROME_HEADER)
                    .add_modifier(Modifier::DIM),
            )
        }
        None => (
            "(no path)".to_string(),
            Style::default()
                .fg(CHROME_CRIT)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let center_display_len = center_str.chars().count();
    let pad_count = (area.width as usize).saturating_sub(11 + 3 + center_display_len + 2 + 6);
    let pad_str: String = " ".repeat(pad_count);

    // Dirty dot — always 2 chars to prevent jitter.
    let (dirty_char, dirty_style) = if app.is_dirty() {
        ("● ", Style::default().fg(CHROME_WARN))
    } else {
        ("  ", Style::default().bg(CHROME_BG))
    };

    let line = Line::from(vec![
        Span::styled(
            title_str,
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            sep_str,
            Style::default()
                .fg(CHROME_HEADER)
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(center_str, center_style),
        Span::raw(pad_str),
        Span::styled(dirty_char, dirty_style),
        Span::styled(
            help_str,
            Style::default()
                .fg(CHROME_HEADER)
                .add_modifier(Modifier::DIM),
        ),
    ]);

    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(CHROME_BG)),
        area,
    );
}

// ── Zone 2: Scrollable List ───────────────────────────────────────────────────

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    // Store viewport height via Cell (interior mutability — no &mut App needed).
    app.list_viewport_height.set(area.height);

    let cursor_dr = app.selectable_indices.get(app.flat_cursor).copied();
    let scroll_offset = app.scroll_offset;
    let viewport_h = area.height as usize;

    let mut lines: Vec<Line> = Vec::new();

    for dr in scroll_offset..(scroll_offset + viewport_h) {
        let row_item = match app.list_rows.get(dr) {
            Some(r) => r,
            None => break,
        };
        let is_cursor = cursor_dr == Some(dr);
        let line = render_row(row_item, app, is_cursor, area.width);
        lines.push(line);
    }

    let para = Paragraph::new(Text::from(lines));
    f.render_widget(para, area);
}

fn render_row(row: &RowItem, app: &App, is_cursor: bool, width: u16) -> Line<'static> {
    let cursor_bg = if is_cursor {
        Style::default().bg(CHROME_SURFACE)
    } else {
        Style::default()
    };

    match row {
        RowItem::SectionHeader(section_idx) => render_section_header(*section_idx, app, width),
        RowItem::Divider => render_divider(width),
        RowItem::SegmentRow(kind) => render_segment_row(*kind, app, is_cursor, cursor_bg, width),
        RowItem::ThemeRow(name) => render_theme_row(name, app, is_cursor, cursor_bg),
        RowItem::StyleRow(name) => render_style_row(name, app, is_cursor, cursor_bg),
        RowItem::ThresholdRow(field) => render_threshold_row(*field, app, is_cursor, cursor_bg),
    }
}

fn render_section_header(section_idx: usize, app: &App, width: u16) -> Line<'static> {
    let dim_italic = Style::default()
        .fg(CHROME_DISABLED)
        .add_modifier(Modifier::DIM | Modifier::ITALIC);

    let (title, badge, title_style) = match section_idx {
        0 => {
            let n = app.config.segments.len();
            let total = crate::model::SegmentKind::ALL.len();
            let badge = format!("({n}/{total} enabled)");
            if app.reorder_mode {
                (
                    "Segments — REORDER ".to_string(),
                    badge,
                    Style::default()
                        .fg(CHROME_ACCENT)
                        .add_modifier(Modifier::DIM | Modifier::ITALIC),
                )
            } else {
                (
                    "Segments ".to_string(),
                    badge,
                    Style::default()
                        .fg(CHROME_HEADER)
                        .add_modifier(Modifier::DIM | Modifier::ITALIC),
                )
            }
        }
        1 => {
            let count = crate::themes::NAMES.len();
            ("Theme ".to_string(), format!("({count})"), dim_italic)
        }
        2 => {
            let count = crate::styles::NAMES.len();
            ("Style ".to_string(), format!("({count})"), dim_italic)
        }
        _ => ("Thresholds ".to_string(), String::new(), dim_italic),
    };

    let prefix = "── ";
    let fill_count = (width as usize)
        .saturating_sub(prefix.len() + title.chars().count() + badge.chars().count() + 1);
    let fill: String = std::iter::repeat('\u{2500}').take(fill_count).collect();

    Line::from(vec![
        Span::styled(prefix, dim_italic),
        Span::styled(title, title_style),
        Span::styled(badge, Style::default().fg(CHROME_TEXT)),
        Span::raw(" "),
        Span::styled(fill, dim_italic),
    ])
}

fn render_divider(width: u16) -> Line<'static> {
    let prefix = "  ─── disabled ";
    let dash_count = (width as usize).saturating_sub(prefix.len());
    let dashes: String = std::iter::repeat('─').take(dash_count).collect();
    let full = format!("{prefix}{dashes}");
    Line::from(Span::styled(
        full,
        Style::default()
            .fg(CHROME_DISABLED)
            .add_modifier(Modifier::DIM),
    ))
}

fn render_segment_row(
    kind: crate::model::SegmentKind,
    app: &App,
    is_cursor: bool,
    cursor_bg: Style,
    _width: u16,
) -> Line<'static> {
    let enabled = app.config.segments.contains(&kind);

    // Badge: position number or spaces (3 chars).
    let badge_span = if app.reorder_mode && is_cursor {
        Span::styled("≡  ", Style::default().fg(CHROME_ACCENT))
    } else if enabled {
        let pos = app
            .config
            .segments
            .iter()
            .position(|s| *s == kind)
            .unwrap_or(0)
            + 1;
        Span::styled(format!("{pos}. "), Style::default().fg(CHROME_HEADER))
    } else {
        Span::raw("   ")
    };

    // Marker: filled or empty circle.
    let marker_span = if enabled {
        Span::styled("● ", Style::default().fg(CHROME_OK))
    } else {
        Span::styled("○ ", Style::default().fg(CHROME_DISABLED))
    };

    // Label.
    let label_span = if enabled {
        Span::styled(
            kind.label(),
            Style::default()
                .fg(CHROME_TEXT)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(kind.label(), Style::default().fg(CHROME_DISABLED))
    };

    let mut spans = vec![Span::raw("  "), badge_span, marker_span, label_span];

    // Apply cursor background to the whole line via a background-colored trailing space.
    if is_cursor {
        // We set the line style below.
        let _ = cursor_bg; // used via line.style()
    }

    let mut line = Line::from(spans.drain(..).collect::<Vec<_>>());
    if is_cursor {
        line = line.style(cursor_bg);
    }
    line
}

fn render_theme_row(
    name: &&'static str,
    app: &App,
    is_cursor: bool,
    cursor_bg: Style,
) -> Line<'static> {
    let name: &'static str = name;
    let is_active = name == app.config.theme;

    let marker_span = if is_active {
        Span::styled("● ", Style::default().fg(CHROME_OK))
    } else {
        Span::raw("  ")
    };

    let name_span = if is_active {
        Span::styled(
            format!("{name:<14}"),
            Style::default()
                .fg(CHROME_TEXT)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(format!("{name:<14}"), Style::default().fg(CHROME_HEADER))
    };

    // Swatch spans.
    let theme_idx = crate::themes::NAMES
        .iter()
        .position(|n| *n == name)
        .unwrap_or(0);
    let swatches = &app.swatch_cache[theme_idx];
    let mut spans: Vec<Span> = vec![Span::raw("  "), marker_span, name_span, Span::raw(" ")];
    for &slot in swatches.iter() {
        spans.push(Span::styled(
            "\u{2588} ",
            Style::default().fg(Color::Indexed(slot)),
        ));
    }

    let mut line = Line::from(spans);
    if is_cursor {
        line = line.style(cursor_bg);
    }
    line
}

fn render_style_row(
    name: &&'static str,
    app: &App,
    is_cursor: bool,
    cursor_bg: Style,
) -> Line<'static> {
    let name: &'static str = name;
    let is_active = name == app.config.style;

    let marker_span = if is_active {
        Span::styled("● ", Style::default().fg(CHROME_OK))
    } else {
        Span::raw("  ")
    };

    let name_span = if is_active {
        Span::styled(
            format!("{name:<14}"),
            Style::default()
                .fg(CHROME_TEXT)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(format!("{name:<14}"), Style::default().fg(CHROME_HEADER))
    };

    let style = crate::styles::get(name);

    // Separator preview.
    let sep_span = Span::raw(style.separator);
    // 2-space gap.
    let gap_span = Span::raw("  ");
    // Bar fill preview (2 cells).
    let fill2 = format!("{0}{0}", style.bar_fill);
    let empty2 = format!("{0}{0}", style.bar_empty);
    let fill_span = Span::styled(fill2, Style::default().fg(CHROME_OK));
    let empty_span = Span::styled(empty2, Style::default().fg(CHROME_DISABLED));

    let mut line = Line::from(vec![
        Span::raw("  "),
        marker_span,
        name_span,
        Span::raw(" "),
        sep_span,
        gap_span,
        fill_span,
        empty_span,
    ]);
    if is_cursor {
        line = line.style(cursor_bg);
    }
    line
}

fn render_threshold_row(
    field: ThresholdField,
    app: &App,
    is_cursor: bool,
    cursor_bg: Style,
) -> Line<'static> {
    let t = &app.config.thresholds;

    let (label, value, unit, lo, hi) = match field {
        ThresholdField::Warn => ("warn", t.warn as i32, "%", 1i32, (t.crit as i32) - 1),
        ThresholdField::Crit => ("crit", t.crit as i32, "%", (t.warn as i32) + 1, 99i32),
        ThresholdField::WeeklyShowAt => {
            ("weekly show at", t.weekly_show_at as i32, "%", 1i32, 99i32)
        }
        ThresholdField::BarWidth => ("bar width", t.bar_width as i32, " ", 2i32, 20i32),
    };

    let range_str = format!("[{lo}–{hi}]");

    let mut line = Line::from(vec![
        Span::styled("  ~ ", Style::default().fg(CHROME_WARN)),
        Span::styled(format!("{label:<14} "), Style::default().fg(CHROME_HEADER)),
        Span::styled(
            format!("{value:>3}"),
            Style::default()
                .fg(CHROME_WARN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{unit:<2}"), Style::default().fg(CHROME_HEADER)),
        Span::raw("  "),
        Span::styled(
            range_str,
            Style::default()
                .fg(CHROME_DISABLED)
                .add_modifier(Modifier::DIM),
        ),
    ]);
    if is_cursor {
        line = line.style(cursor_bg);
    }
    line
}

// ── Zone 3: Description ───────────────────────────────────────────────────────

fn draw_desc(f: &mut Frame, app: &App, area: Rect) {
    let line = build_desc_line(app);
    f.render_widget(Paragraph::new(line), area);
}

fn build_desc_line(app: &App) -> Line<'static> {
    let dim_italic = Style::default()
        .fg(CHROME_HEADER)
        .add_modifier(Modifier::DIM | Modifier::ITALIC);

    let cursor_row = match app.cursor_row() {
        Some(r) => r.clone(),
        None => return Line::from(Span::raw(" ")),
    };

    match cursor_row {
        RowItem::SegmentRow(kind) => {
            if app.config.segments.contains(&kind) {
                let desc: &'static str = match kind {
                    crate::model::SegmentKind::Directory => {
                        "Current working directory, fish-style abbreviated"
                    }
                    crate::model::SegmentKind::Git => {
                        "Branch name, ahead/behind counts, modified and untracked files"
                    }
                    crate::model::SegmentKind::Context => {
                        "Token usage bar and count for the current session"
                    }
                    crate::model::SegmentKind::RateLimits => {
                        "5-hour and weekly API usage windows with live reset countdown"
                    }
                    crate::model::SegmentKind::Model => "Active Claude model display name",
                };
                Line::from(Span::styled(desc, dim_italic))
            } else {
                let label = kind.label().to_string();
                Line::from(vec![
                    Span::styled(format!("{label} — disabled · "), dim_italic),
                    Span::styled(
                        "[Space]",
                        Style::default()
                            .fg(CHROME_CRIT)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to enable", dim_italic),
                ])
            }
        }
        RowItem::ThemeRow(name) => {
            if name == app.saved_config.theme {
                Line::from(Span::styled(format!("Using {name}"), dim_italic))
            } else {
                let saved = app.saved_config.theme.clone();
                Line::from(Span::styled(
                    format!(
                        "Previewing {name} — originally: {saved} · [s] save  [j/k] browse  [1/Tab] jump"
                    ),
                    Style::default()
                        .fg(CHROME_WARN)
                        .add_modifier(Modifier::DIM | Modifier::ITALIC),
                ))
            }
        }
        RowItem::StyleRow(name) => {
            let font_note: &str = if name == "ascii" {
                " · Works in all terminals"
            } else {
                " · Requires Nerd Font"
            };
            if name == app.saved_config.style {
                Line::from(Span::styled(format!("Using {name}{font_note}"), dim_italic))
            } else {
                Line::from(Span::styled(
                    format!("Previewing {name}{font_note} — [s] save  [j/k] browse"),
                    Style::default()
                        .fg(CHROME_WARN)
                        .add_modifier(Modifier::DIM | Modifier::ITALIC),
                ))
            }
        }
        RowItem::ThresholdRow(field) => {
            let t = &app.config.thresholds;
            let desc: String = match field {
                ThresholdField::Warn => format!(
                    "Warn threshold: color bar yellow at/above. Range constrained to [1–{}%].",
                    t.crit - 1
                ),
                ThresholdField::Crit => format!(
                    "Crit threshold: color bar red at/above. Range constrained to [{}–99%].",
                    t.warn + 1
                ),
                ThresholdField::WeeklyShowAt => {
                    "Show weekly rate-limit window once usage exceeds this percent. Range: [1–99%]."
                        .to_string()
                }
                ThresholdField::BarWidth => {
                    "Progress bar width in terminal cells. Range: [2–20].".to_string()
                }
            };
            Line::from(Span::styled(desc, dim_italic))
        }
        _ => Line::from(Span::raw(" ")),
    }
}

// ── Zone 4: Preview Block ─────────────────────────────────────────────────────

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let sample = app.current_sample();
    let title = format!(" Preview — {} ", sample.name);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(CHROME_HEADER)
                .add_modifier(Modifier::DIM),
        )
        .title(Span::styled(
            title,
            Style::default()
                .fg(CHROME_HEADER)
                .add_modifier(Modifier::DIM),
        ));

    if app.config.segments.is_empty() {
        let msg = Paragraph::new(Line::from(vec![
            Span::raw("No segments enabled — "),
            Span::styled(
                "[Space]",
                Style::default()
                    .fg(CHROME_CRIT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to enable one"),
        ]))
        .block(block);
        f.render_widget(msg, area);
        return;
    }

    let text = preview::render(&app.config, sample);
    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

// ── Zone 5: Status Line ───────────────────────────────────────────────────────

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let para = if app.pending_reset {
        Paragraph::new(Line::from(vec![Span::styled(
            "  Reset to defaults? — [r] confirm   [any other key] cancel  ",
            Style::default().fg(CHROME_WARN),
        )]))
    } else if app.pending_quit {
        Paragraph::new(Line::from(vec![Span::styled(
            "  Unsaved changes — [s] save & quit   [q] discard   [any other key] cancel  ",
            Style::default().fg(CHROME_WARN),
        )]))
    } else if let Some((kind, msg)) = &app.status {
        let color = match kind {
            StatusKind::Success => CHROME_OK,
            StatusKind::Warning => CHROME_WARN,
            StatusKind::Error => CHROME_CRIT,
        };
        let msg = msg.clone();
        Paragraph::new(Line::from(vec![Span::styled(
            format!("  {msg}"),
            Style::default().fg(color),
        )]))
    } else {
        Paragraph::new(Line::from(Span::raw(" ")))
    };

    f.render_widget(para, area);
}

// ── Zone 6: Hint Line ─────────────────────────────────────────────────────────

fn draw_hint(f: &mut Frame, app: &App, area: Rect) {
    if app.pending_reset || app.pending_quit {
        f.render_widget(Paragraph::new(Line::from(Span::raw(" "))), area);
        return;
    }

    let cursor_row = app.cursor_row().cloned();
    let line = if app.reorder_mode {
        // Variant 1: reorder mode.
        hint_line(
            &[
                ("[j/k]", " move  "),
                ("[m/Enter/Esc]", " done  "),
                ("[1–4]", " jump"),
            ],
            false,
        )
    } else if app.config.segments.is_empty() {
        // Variant 2: all segments disabled — [Space] uses CHROME_CRIT bg.
        hint_line_with_crit_space(&[
            ("[Space]", " enable  ", true),
            ("[j/k]", " move  ", false),
            ("[?]", " help  ", false),
            ("[s]", " save  ", false),
            ("[q]", " quit", false),
        ])
    } else {
        match &cursor_row {
            Some(RowItem::SegmentRow(kind)) if app.config.segments.contains(kind) => {
                // Variant 3: enabled segment.
                hint_line(
                    &[
                        ("[Space]", " toggle  "),
                        ("[m]", " reorder  "),
                        ("[j/k]", " move  "),
                        ("[1–4]", " section  "),
                        ("[s]", " save  "),
                        ("[?]", " help  "),
                        ("[q]", " quit"),
                    ],
                    false,
                )
            }
            Some(RowItem::SegmentRow(_)) => {
                // Variant 4: disabled segment.
                hint_line(
                    &[
                        ("[Space]", " enable  "),
                        ("[j/k]", " move  "),
                        ("[1–4]", " section  "),
                        ("[s]", " save  "),
                        ("[?]", " help  "),
                        ("[q]", " quit"),
                    ],
                    false,
                )
            }
            Some(RowItem::ThemeRow(_)) | Some(RowItem::StyleRow(_)) => {
                // Variant 5: theme or style.
                hint_line(
                    &[
                        ("[j/k]", " browse  "),
                        ("[s]", " save  "),
                        ("[1–4]", " section  "),
                        ("[p]", " sample  "),
                        ("[?]", " help  "),
                        ("[q]", " quit"),
                    ],
                    false,
                )
            }
            Some(RowItem::ThresholdRow(_)) => {
                // Variant 6: threshold.
                hint_line(
                    &[
                        ("[h/l]", " ±1  "),
                        ("[H/L]", " ±5  "),
                        ("[j/k]", " move  "),
                        ("[s]", " save  "),
                        ("[?]", " help  "),
                        ("[q]", " quit"),
                    ],
                    false,
                )
            }
            _ => {
                // Variant 7: default.
                hint_line(
                    &[
                        ("[j/k]", " move  "),
                        ("[1–4]", " section  "),
                        ("[s]", " save  "),
                        ("[?]", " help  "),
                        ("[q]", " quit"),
                    ],
                    false,
                )
            }
        }
    };

    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(CHROME_BG)),
        area,
    );
}

/// Build a hint line from (key, desc) pairs. All key buttons use CHROME_KEY_BG.
fn hint_line(pairs: &[(&str, &str)], _crit_first: bool) -> Line<'static> {
    let mut spans = vec![Span::raw("  ")];
    for &(key, desc) in pairs {
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(CHROME_BG).bg(CHROME_KEY_BG),
        ));
        spans.push(Span::styled(
            desc.to_string(),
            Style::default().fg(CHROME_HEADER),
        ));
    }
    Line::from(spans)
}

/// Build a hint line where entries marked true use CHROME_CRIT bg for [key].
fn hint_line_with_crit_space(pairs: &[(&str, &str, bool)]) -> Line<'static> {
    let mut spans = vec![Span::raw("  ")];
    for &(key, desc, crit) in pairs {
        let key_bg = if crit { CHROME_CRIT } else { CHROME_KEY_BG };
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(CHROME_BG).bg(key_bg),
        ));
        spans.push(Span::styled(
            desc.to_string(),
            Style::default().fg(CHROME_HEADER),
        ));
    }
    Line::from(spans)
}

// ── Help Overlay ──────────────────────────────────────────────────────────────

fn draw_help_overlay(f: &mut Frame, overlay_area: Rect) {
    f.render_widget(Clear, overlay_area);

    let content = vec![
        Line::from(Span::styled(
            " Movement",
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  j / ↓ ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Move cursor down one row",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  k / ↑ ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Move cursor up one row", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  g       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Jump to top of list", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  G       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Jump to bottom of list", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  Tab     ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Jump to next section", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  Shift-Tab",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Jump to previous section",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  1–4    ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "   Jump to section 1=Segments 2=Theme 3=Style 4=Thresholds",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " Segments",
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  Space   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Toggle segment enabled/disabled",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  m       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Enter reorder mode (enabled segments only)",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " Reorder mode",
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  j / ↓   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Move segment down", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  k / ↑   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Move segment up", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  m / Enter",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Exit reorder mode (commit)",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Esc     ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Exit reorder mode (commit + flash)",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " Thresholds",
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  h / ←   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Decrement by 1", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  l / →   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Increment by 1", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  H       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Decrement by 5", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  L       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Increment by 5", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            " Global",
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  s / Ctrl-S",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Save config", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  r       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Reset to defaults (confirm prompt)",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  p       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Cycle preview sample forward",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  P       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Cycle preview sample backward",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  ?       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Close this help overlay",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  q / Esc ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Quit (confirm if unsaved)",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
    ];

    let block = Block::bordered()
        .title(Span::styled(
            " ? Keybindings ",
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(CHROME_ACCENT));

    f.render_widget(Paragraph::new(content).block(block), overlay_area);
}
