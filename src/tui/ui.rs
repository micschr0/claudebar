//! Drawing: three-zone split-pane layout — left menu panel, right detail panel, bottom preview.
//! No state mutation happens here (`Cell<Rect>` excepted).

use crate::tui::app::{App, Panel, StatusKind, ThresholdField};
use crate::tui::preview;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

// ── Chrome color constants (Rgb — immune to 256-color palette remapping) ──────

const CHROME_BG: Color = Color::Rgb(16, 16, 24); // #101018 — statusline background
const CHROME_SURFACE: Color = Color::Rgb(45, 49, 72); // cursor row highlight
const CHROME_ACCENT: Color = Color::Rgb(122, 162, 247); // title, reorder mode header
const CHROME_HEADER: Color = Color::Rgb(169, 177, 214); // section headers, hint text
const CHROME_TEXT: Color = Color::Rgb(192, 202, 245); // primary list content
const CHROME_OK: Color = Color::Rgb(158, 206, 106); // enabled bullet, save success
const CHROME_WARN: Color = Color::Rgb(224, 175, 104); // dirty dot, pending banners
const CHROME_CRIT: Color = Color::Rgb(247, 118, 142); // errors, disabled call-to-action
const CHROME_DISABLED: Color = Color::Rgb(115, 122, 164); // disabled labels, section fills
const CHROME_KEY_BG: Color = Color::Rgb(65, 72, 104); // hint line [x] button bg

/// Build an active or inactive bordered panel block.
fn panel_block(title: &str, active: bool) -> Block<'_> {
    if active {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(CHROME_ACCENT))
            .title(Span::styled(
                format!(" {title} "),
                Style::default()
                    .fg(CHROME_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ))
    } else {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(CHROME_HEADER)
                    .add_modifier(Modifier::DIM),
            )
            .title(Span::styled(
                format!(" {title} "),
                Style::default()
                    .fg(CHROME_HEADER)
                    .add_modifier(Modifier::DIM),
            ))
    }
}

/// Top-level draw entrypoint — three-zone split-pane layout.
pub fn draw(f: &mut Frame, app: &App) {
    // Size guard: split-pane needs at least 80 cols and 20 rows.
    if f.area().width < 80 || f.area().height < 20 {
        f.render_widget(
            Paragraph::new("Terminal too small (min 80×20)").style(
                Style::default()
                    .fg(CHROME_CRIT)
                    .add_modifier(Modifier::BOLD),
            ),
            f.area(),
        );
        return;
    }

    // Inset by 1 row top and bottom for breathing room.
    let inner = Rect {
        x: f.area().x,
        y: f.area().y + 1,
        width: f.area().width,
        height: f.area().height.saturating_sub(2),
    };

    // Three vertical zones.
    let [panels_area, preview_area, status_area, hint_area] = Layout::vertical([
        Constraint::Min(10),   // top row: left + right panels
        Constraint::Length(3), // preview block with border
        Constraint::Length(1), // status line
        Constraint::Length(1), // hint bar
    ])
    .areas(inner);
    let [left_area, right_area] = Layout::horizontal([
        Constraint::Percentage(28), // left menu panel
        Constraint::Min(0),         // right detail panel
    ])
    .areas(panels_area);

    // Store panel areas for mouse hit-testing (Cell interior mutability).
    app.left_panel_area.set(left_area);
    app.right_panel_area.set(right_area);

    draw_left_panel(f, app, left_area);
    draw_right_panel(f, app, right_area);
    draw_preview(f, app, preview_area);
    draw_status(f, app, status_area);
    draw_hint(f, app, hint_area);

    // Help overlay spans the full panels_area (both left and right).
    if app.show_help {
        draw_help_overlay(f, panels_area);
    }
}

// ── Left panel: section menu ──────────────────────────────────────────────────

fn draw_left_panel(f: &mut Frame, app: &App, area: Rect) {
    let active = app.focused_panel == Panel::Left;
    let title = if app.is_dirty() { "● Menu" } else { "Menu" };
    let block = panel_block(title, active);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = [
        (
            "Segments",
            format!(
                "  {}/{}",
                app.config.segments.len(),
                crate::model::SegmentKind::ALL.len()
            ),
        ),
        ("Theme", format!("  {}", app.config.theme)),
        ("Style", format!("  {}", app.config.style)),
        ("Thresholds", String::new()),
    ];

    let lines: Vec<Line> = sections
        .iter()
        .enumerate()
        .map(|(idx, (label, badge))| {
            let selected = idx == app.menu_cursor;
            let marker = if selected {
                Span::styled("● ", Style::default().fg(CHROME_OK))
            } else {
                Span::styled("○ ", Style::default().fg(CHROME_DISABLED))
            };
            let label_style = if selected {
                Style::default()
                    .fg(CHROME_TEXT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(CHROME_HEADER)
            };
            let badge_style = Style::default()
                .fg(CHROME_DISABLED)
                .add_modifier(Modifier::DIM);

            let mut line = Line::from(vec![
                Span::raw(" "),
                marker,
                Span::styled(label.to_string(), label_style),
                Span::styled(badge.clone(), badge_style),
            ]);
            if selected {
                line = line.style(Style::default().bg(CHROME_SURFACE));
            }
            line
        })
        .collect();

    f.render_widget(Paragraph::new(Text::from(lines)), inner);
}

// ── Right panel: section detail ───────────────────────────────────────────────

fn draw_right_panel(f: &mut Frame, app: &App, area: Rect) {
    let section_title = match app.menu_cursor {
        0 => {
            if app.reorder_mode {
                "Segments — REORDER".to_string()
            } else {
                "Segments".to_string()
            }
        }
        1 => "Theme".to_string(),
        2 => "Style".to_string(),
        _ => "Thresholds".to_string(),
    };

    let active = app.focused_panel == Panel::Right;
    let block = panel_block(&section_title, active);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Reserve 2 rows at the bottom for the config-values footer.
    let item_height = inner.height.saturating_sub(2) as usize;

    let item_lines: Vec<Line> = match app.menu_cursor {
        0 => build_segment_lines(app, inner.width),
        1 => build_theme_lines(app),
        2 => build_style_lines(app),
        _ => build_threshold_lines(app),
    };

    // Virtual scroll: keep detail_cursor visible.
    let scroll_start = app
        .detail_cursor
        .saturating_sub(item_height.saturating_sub(1));
    let total = item_lines.len();
    let has_more_above = scroll_start > 0;
    let has_more_below = total > scroll_start + item_height;

    let mut visible: Vec<Line> = item_lines
        .into_iter()
        .skip(scroll_start)
        .take(item_height)
        .collect();

    // Scroll indicators: show '…' when list extends beyond visible area.
    if has_more_below && !visible.is_empty() {
        visible.pop();
        visible.push(Line::from(Span::styled(
            "  \u{2026}",
            Style::default()
                .fg(CHROME_DISABLED)
                .add_modifier(Modifier::DIM),
        )));
    }
    if has_more_above && !visible.is_empty() {
        visible.remove(0);
        visible.insert(
            0,
            Line::from(Span::styled(
                "  \u{2026}",
                Style::default()
                    .fg(CHROME_DISABLED)
                    .add_modifier(Modifier::DIM),
            )),
        );
    }

    let item_area = Rect::new(inner.x, inner.y, inner.width, item_height as u16);
    f.render_widget(Paragraph::new(Text::from(visible)), item_area);

    // Config-values footer (D-07): one row from the bottom of inner.
    let footer_y = inner.y + inner.height.saturating_sub(1);
    let footer_area = Rect::new(inner.x, footer_y, inner.width, 1);
    let footer_text = format!(
        " Theme: {}  Style: {}  Segs: {}/{}  Warn: {}%  Crit: {}%",
        app.config.theme,
        app.config.style,
        app.config.segments.len(),
        crate::model::SegmentKind::ALL.len(),
        app.config.thresholds.warn,
        app.config.thresholds.crit,
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default()
                .fg(CHROME_DISABLED)
                .add_modifier(Modifier::DIM),
        ))),
        footer_area,
    );
}

fn build_segment_lines(app: &App, width: u16) -> Vec<Line<'static>> {
    // Display order: enabled in config.segments order, then disabled in ALL order.
    let display_order: Vec<crate::model::SegmentKind> = {
        let mut order: Vec<crate::model::SegmentKind> = app.config.segments.clone();
        for &kind in &crate::model::SegmentKind::ALL {
            if !app.config.segments.contains(&kind) {
                order.push(kind);
            }
        }
        order
    };

    display_order
        .into_iter()
        .enumerate()
        .map(|(idx, kind)| {
            let is_cursor = idx == app.detail_cursor;
            let cursor_bg = if is_cursor {
                Style::default().bg(CHROME_SURFACE)
            } else {
                Style::default()
            };
            render_segment_row(kind, app, is_cursor, cursor_bg, width)
        })
        .collect()
}

fn build_theme_lines(app: &App) -> Vec<Line<'static>> {
    crate::themes::NAMES
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let is_cursor = idx == app.detail_cursor;
            let cursor_bg = if is_cursor {
                Style::default().bg(CHROME_SURFACE)
            } else {
                Style::default()
            };
            render_theme_row(name, app, is_cursor, cursor_bg)
        })
        .collect()
}

fn build_style_lines(app: &App) -> Vec<Line<'static>> {
    crate::styles::NAMES
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let is_cursor = idx == app.detail_cursor;
            let cursor_bg = if is_cursor {
                Style::default().bg(CHROME_SURFACE)
            } else {
                Style::default()
            };
            render_style_row(name, app, is_cursor, cursor_bg)
        })
        .collect()
}

fn build_threshold_lines(app: &App) -> Vec<Line<'static>> {
    [
        ThresholdField::Warn,
        ThresholdField::Crit,
        ThresholdField::WeeklyShowAt,
        ThresholdField::BarWidth,
        ThresholdField::ClockMode,
        ThresholdField::Layout,
    ]
    .iter()
    .enumerate()
    .map(|(idx, &field)| {
        let is_cursor = idx == app.detail_cursor;
        let cursor_bg = if is_cursor {
            Style::default().bg(CHROME_SURFACE)
        } else {
            Style::default()
        };
        render_threshold_row(field, app, is_cursor, cursor_bg)
    })
    .collect()
}

// ── Row renderers (unchanged from old flat-list ui) ───────────────────────────

fn render_segment_row(
    kind: crate::model::SegmentKind,
    app: &App,
    is_cursor: bool,
    cursor_bg: Style,
    _width: u16,
) -> Line<'static> {
    let enabled = app.config.segments.contains(&kind);

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

    let marker_span = if enabled {
        Span::styled("● ", Style::default().fg(CHROME_OK))
    } else {
        Span::styled("○ ", Style::default().fg(CHROME_DISABLED))
    };

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

    let spans = vec![Span::raw("  "), badge_span, marker_span, label_span];
    let mut line = Line::from(spans);
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
    let sep_span = Span::raw(style.separator);
    let gap_a = Span::raw("  ");
    let gap_b = Span::raw("  ");
    let fill2 = format!("{0}{0}", style.bar_fill);
    let empty2 = format!("{0}{0}", style.bar_empty);
    let fill_span = Span::styled(fill2, Style::default().fg(CHROME_OK));
    let empty_span = Span::styled(empty2, Style::default().fg(CHROME_DISABLED));
    let project_span = Span::raw(style.glyphs.project);
    let duration_span = Span::raw(style.glyphs.duration);

    let mut line = Line::from(vec![
        Span::raw("  "),
        marker_span,
        name_span,
        Span::raw(" "),
        sep_span,
        gap_a,
        fill_span,
        empty_span,
        gap_b,
        project_span,
        Span::raw(" "),
        duration_span,
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

    let (label, value_str, unit_or_options) = match field {
        ThresholdField::Warn => ("warn", format!("{:>3}", t.warn), "%".to_string()),
        ThresholdField::Crit => ("crit", format!("{:>3}", t.crit), "%".to_string()),
        ThresholdField::WeeklyShowAt => (
            "weekly show at",
            format!("{:>3}", t.weekly_show_at),
            "%".to_string(),
        ),
        ThresholdField::BarWidth => ("bar width", format!("{:>3}", t.bar_width), " ".to_string()),
        ThresholdField::ClockMode => {
            let options = "auto | 12h | 24h | off";
            (
                if is_cursor { options } else { "clock" },
                app.config.thresholds.clock_mode.clone(),
                String::new(),
            )
        }
        ThresholdField::Layout => {
            let options = "fixed | auto";
            (
                if is_cursor { options } else { "layout" },
                app.config.thresholds.layout.clone(),
                String::new(),
            )
        }
    };

    let mut line = if matches!(field, ThresholdField::ClockMode | ThresholdField::Layout) {
        // Enum-cycling fields show options when focused, current value otherwise.
        Line::from(vec![
            Span::styled("  ~ ", Style::default().fg(CHROME_WARN)),
            Span::styled(format!("{label:<14} "), Style::default().fg(CHROME_HEADER)),
            Span::styled(
                value_str,
                Style::default()
                    .fg(CHROME_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            if !unit_or_options.is_empty() {
                Span::raw(format!("  ({unit_or_options})"))
            } else {
                Span::raw("")
            },
        ])
    } else {
        // Numeric nudge fields — show value and range.
        let (lo, hi) = match field {
            ThresholdField::Warn => (1i32, i32::from(t.crit) - 1),
            ThresholdField::Crit => (i32::from(t.warn) + 1, 99i32),
            ThresholdField::WeeklyShowAt => (1i32, 99i32),
            ThresholdField::BarWidth => (2i32, 20i32),
            _ => (0i32, 0i32),
        };
        let range_str = format!("[{lo}\u{2013}{hi}]");
        Line::from(vec![
            Span::styled("  ~ ", Style::default().fg(CHROME_WARN)),
            Span::styled(format!("{label:<14} "), Style::default().fg(CHROME_HEADER)),
            Span::styled(
                value_str,
                Style::default()
                    .fg(CHROME_WARN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{unit_or_options:<2}"),
                Style::default().fg(CHROME_HEADER),
            ),
            Span::raw("  "),
            Span::styled(
                range_str,
                Style::default()
                    .fg(CHROME_DISABLED)
                    .add_modifier(Modifier::DIM),
            ),
        ])
    };
    if is_cursor {
        line = line.style(cursor_bg);
    }
    line
}

// ── Preview block ─────────────────────────────────────────────────────────────

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

// ── Status line ───────────────────────────────────────────────────────────────

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
    } else if let Some(help) = app.context_help() {
        Paragraph::new(Line::from(vec![Span::styled(
            format!("  {help}"),
            Style::default()
                .fg(CHROME_HEADER)
                .add_modifier(Modifier::DIM),
        )]))
    } else if app.is_dirty() {
        Paragraph::new(Line::from(vec![Span::styled(
            "  unsaved changes — press [s] to save",
            Style::default()
                .fg(CHROME_DISABLED)
                .add_modifier(Modifier::DIM),
        )]))
    } else {
        Paragraph::new(Line::from(Span::raw(" ")))
    };

    f.render_widget(para, area);
}

// ── Hint bar ──────────────────────────────────────────────────────────────────

fn draw_hint(f: &mut Frame, app: &App, area: Rect) {
    if app.pending_reset || app.pending_quit {
        f.render_widget(Paragraph::new(Line::from(Span::raw(" "))), area);
        return;
    }

    let line = if app.reorder_mode {
        hint_line(&[
            ("[j/k]", " move  "),
            ("[m/Enter/Esc]", " done  "),
            ("[1\u{2013}4]", " jump"),
        ])
    } else if app.config.segments.is_empty() {
        hint_line_with_crit_space(&[
            ("[Space]", " enable  ", true),
            ("[j/k]", " move  ", false),
            ("[?]", " help  ", false),
            ("[s]", " save  ", false),
            ("[q]", " quit", false),
        ])
    } else if app.focused_panel == Panel::Right && app.menu_cursor == 0 {
        hint_line(&[
            ("[Space]", " toggle  "),
            ("[m]", " reorder  "),
            ("[←→]", " panel  "),
            ("[↑↓]", " move  "),
            ("[s]", " save  "),
            ("[?]", " help  "),
            ("[q]", " quit"),
        ])
    } else if app.focused_panel == Panel::Right && (app.menu_cursor == 1 || app.menu_cursor == 2) {
        hint_line(&[
            ("[↑↓]", " browse  "),
            ("[s]", " save  "),
            ("[p]", " sample  "),
            ("[?]", " help  "),
            ("[q]", " quit"),
        ])
    } else if app.focused_panel == Panel::Right && app.menu_cursor == 3 {
        hint_line(&[
            ("[-/=]", " \u{b1}1  "),
            ("[_/+]", " \u{b1}5  "),
            ("[Spc/Ent]", " cycle  "),
            ("[\u{2191}\u{2193}]", " move  "),
            ("[s]", " save  "),
            ("[?]", " help  "),
            ("[q]", " quit"),
        ])
    } else {
        // Left panel focused or default.
        hint_line(&[
            ("[←→]", " panel  "),
            ("[↑↓]", " section  "),
            ("[1\u{2013}4]", " jump  "),
            ("[s]", " save  "),
            ("[?]", " help  "),
            ("[q]", " quit"),
        ])
    };

    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(CHROME_BG)),
        area,
    );
}

fn hint_line(pairs: &[(&str, &str)]) -> Line<'static> {
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

// ── Help overlay ──────────────────────────────────────────────────────────────

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
                "  \u{2190} / h  ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Switch focus to left panel",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  \u{2192} / l  ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Switch focus to right panel",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  j / \u{2193} ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Move cursor down within focused panel",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  k / \u{2191} ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Move cursor up within focused panel",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  g       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Jump to top of right panel list",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  G       ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Jump to bottom of right panel list",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Tab     ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Toggle focus between left and right panel",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  1\u{2013}4    ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "   Jump to section: 1=Segments  2=Theme  3=Style  4=Thresholds",
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
                "  j / \u{2193}   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Move segment down", Style::default().fg(CHROME_TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "  k / \u{2191}   ",
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
                "  - / =   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Nudge \u{b1}1 (numeric fields only)",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  _ / +   ",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Nudge \u{b1}5 (numeric fields only)",
                Style::default().fg(CHROME_TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Space / Enter",
                Style::default()
                    .fg(CHROME_KEY_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Cycle value (clock_mode, layout)",
                Style::default().fg(CHROME_TEXT),
            ),
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
                "  s       ",
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
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " ? Keybindings ",
            Style::default()
                .fg(CHROME_ACCENT)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(CHROME_ACCENT));

    f.render_widget(Paragraph::new(content).block(block), overlay_area);
}
