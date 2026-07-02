//! TUI configurator: split-pane design with left menu panel, right detail panel, and bottom preview.
//! Save writes the edited [`crate::model::Config`] back to the config path as TOML.

mod app;
mod preview;
mod sample;
mod ui;

use app::{
    App, Dir, Panel, RowItem, StatusKind, ThresholdField, build_list, detail_len, move_segment,
};
use crossterm::ExecutableCommand;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

/// RAII guard: the terminal is restored when this drops, including on panic or
/// any early return from the event loop.
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn enter() -> Result<TerminalGuard, String> {
        enable_raw_mode().map_err(|e| format!("enable raw mode: {e}"))?;
        let mut out = io::stdout();
        out.execute(EnterAlternateScreen)
            .map_err(|e| format!("enter alternate screen: {e}"))?;
        out.execute(EnableMouseCapture)
            .map_err(|e| format!("enable mouse: {e}"))?;
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend).map_err(|e| format!("init terminal: {e}"))?;
        Ok(TerminalGuard { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(DisableMouseCapture);
        let _ = io::stdout().execute(LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Run the interactive configurator, persisting to `config_path` on save.
pub fn run(config_path: Option<PathBuf>) -> Result<(), String> {
    let config = crate::model::Config::load_or_default(config_path.as_deref());
    let save_path = config_path.or_else(crate::model::Config::default_path);
    let mut app = App::new(config, save_path);

    let mut guard = TerminalGuard::enter()?;
    let res = event_loop(&mut guard.terminal, &mut app);
    drop(guard);
    res
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<(), String> {
    loop {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|e| format!("draw: {e}"))?;

        if event::poll(Duration::from_millis(200)).map_err(|e| format!("poll: {e}"))? {
            match event::read().map_err(|e| format!("read: {e}"))? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key(app, key);
                }
                Event::Mouse(mouse) => {
                    handle_mouse(app, mouse);
                }
                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    // ── Priority 1: Help overlay consumes all input ───────────────────────────
    if app.show_help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                app.show_help = false;
            }
            _ => {}
        }
        return;
    }

    // ── Priority 2: Pending reset ─────────────────────────────────────────────
    if app.pending_reset {
        handle_pending_reset(app, key);
        return;
    }

    // ── Priority 3: Pending quit ──────────────────────────────────────────────
    if app.pending_quit {
        handle_pending_quit(app, key);
        return;
    }

    // ── Priority 4: Reorder mode ──────────────────────────────────────────────
    if app.reorder_mode {
        handle_reorder(app, key);
        return;
    }

    // ── Priority 5: Clear transient status ───────────────────────────────────
    app.status = None;

    // ── Priority 6: Normal dispatch ───────────────────────────────────────────
    handle_normal(app, key);
}

fn handle_pending_reset(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') => {
            app.reset();
            app.pending_reset = false;
        }
        KeyCode::Char('s') => {
            // Cancel reset, save directly.
            app.pending_reset = false;
            app.save();
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            // Cancel pending_reset, then re-dispatch to normal handler.
            app.pending_reset = false;
            app.status = None;
            // Re-dispatch: this may arm pending_quit if dirty.
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                if app.is_dirty() {
                    app.request_quit();
                } else {
                    app.should_quit = true;
                }
            }
        }
        // Navigation keys: silent no-op (preserve banner).
        KeyCode::Char('j')
        | KeyCode::Char('k')
        | KeyCode::Up
        | KeyCode::Down
        | KeyCode::Char('h')
        | KeyCode::Char('l')
        | KeyCode::Char('H')
        | KeyCode::Char('L')
        | KeyCode::Char('1')
        | KeyCode::Char('2')
        | KeyCode::Char('3')
        | KeyCode::Char('4')
        | KeyCode::Tab
        | KeyCode::BackTab => {}
        // Any other key: silent cancel.
        _ => {
            app.pending_reset = false;
        }
    }
}

fn handle_pending_quit(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('s') => {
            if app.save_path.is_none() {
                app.pending_quit = false;
                app.status = Some((
                    StatusKind::Error,
                    "no save path (set $HOME or --config)".into(),
                ));
            } else {
                app.save();
                app.should_quit = true;
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        // Navigation keys: silent no-op (preserve banner).
        KeyCode::Char('j')
        | KeyCode::Char('k')
        | KeyCode::Up
        | KeyCode::Down
        | KeyCode::Char('h')
        | KeyCode::Char('l')
        | KeyCode::Char('H')
        | KeyCode::Char('L')
        | KeyCode::Char('1')
        | KeyCode::Char('2')
        | KeyCode::Char('3')
        | KeyCode::Char('4')
        | KeyCode::Tab
        | KeyCode::BackTab => {}
        // Any other key: silent cancel.
        _ => {
            app.pending_quit = false;
        }
    }
}

fn handle_reorder(app: &mut App, key: KeyEvent) {
    // In reorder mode, detail_cursor is the index into the enabled segment list.
    let moved_kind = match app.config.segments.get(app.detail_cursor) {
        Some(&k) => k,
        None => {
            app.reorder_mode = false;
            return;
        }
    };
    let seg_idx = app.detail_cursor;

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if seg_idx + 1 >= app.config.segments.len() {
                return;
            }
            move_segment(&mut app.config.segments, seg_idx, Dir::Down);
            rebuild_and_follow(app, moved_kind);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if seg_idx == 0 {
                return;
            }
            move_segment(&mut app.config.segments, seg_idx, Dir::Up);
            rebuild_and_follow(app, moved_kind);
        }
        KeyCode::Char('m') | KeyCode::Enter => {
            app.reorder_mode = false;
        }
        KeyCode::Esc => {
            app.reorder_mode = false;
            app.status = Some((
                StatusKind::Success,
                "Reorder committed — reorder again to undo, or [r] to reset all".into(),
            ));
        }
        _ => {}
    }
}

fn rebuild_and_follow(app: &mut App, moved_kind: crate::model::SegmentKind) {
    let (list_rows, selectable_indices, section_starts) = build_list(&app.config);
    app.list_rows = list_rows;
    app.selectable_indices = selectable_indices;
    app.section_starts = section_starts;

    // In reorder mode, the moved segment is always enabled, so its index in
    // config.segments equals its position in the right panel's item list.
    if let Some(idx) = app.config.segments.iter().position(|&k| k == moved_kind) {
        app.detail_cursor = idx;
    }

    // Also update flat_cursor for backward compat with unit tests.
    if let Some(new_si) = app
        .selectable_indices
        .iter()
        .enumerate()
        .find_map(|(si, &dr)| {
            if let RowItem::SegmentRow(k) = &app.list_rows[dr]
                && *k == moved_kind
            {
                return Some(si);
            }
            None
        })
    {
        app.flat_cursor = new_si;
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) {
    let _ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        // ── Panel switching and within-panel navigation ───────────────────────
        KeyCode::Left | KeyCode::Char('h') => {
            app.focused_panel = Panel::Left;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.focused_panel = Panel::Right;
        }
        KeyCode::Up | KeyCode::Char('k') => match app.focused_panel {
            Panel::Left => {
                if app.menu_cursor > 0 {
                    app.menu_cursor -= 1;
                    app.detail_cursor = app.detail_cursor.min(detail_len(app).saturating_sub(1));
                }
            }
            Panel::Right => {
                if app.detail_cursor > 0 {
                    app.detail_cursor -= 1;
                    app.apply_move_is_select();
                }
            }
        },
        KeyCode::Down | KeyCode::Char('j') => match app.focused_panel {
            Panel::Left => {
                if app.menu_cursor < 3 {
                    app.menu_cursor += 1;
                    app.detail_cursor = app.detail_cursor.min(detail_len(app).saturating_sub(1));
                }
            }
            Panel::Right => {
                let max = detail_len(app).saturating_sub(1);
                if app.detail_cursor < max {
                    app.detail_cursor += 1;
                    app.apply_move_is_select();
                }
            }
        },
        KeyCode::Char('g') => {
            app.detail_cursor = 0;
        }
        KeyCode::Char('G') => {
            app.detail_cursor = detail_len(app).saturating_sub(1);
        }
        KeyCode::Tab => {
            app.focused_panel = match app.focused_panel {
                Panel::Left => Panel::Right,
                Panel::Right => Panel::Left,
            };
        }
        KeyCode::BackTab => {
            app.focused_panel = match app.focused_panel {
                Panel::Left => Panel::Right,
                Panel::Right => Panel::Left,
            };
        }
        KeyCode::Char('1') => {
            app.menu_cursor = 0;
            app.detail_cursor = 0;
            app.focused_panel = Panel::Right;
        }
        KeyCode::Char('2') => {
            app.menu_cursor = 1;
            app.detail_cursor = 0;
            app.focused_panel = Panel::Right;
        }
        KeyCode::Char('3') => {
            app.menu_cursor = 2;
            app.detail_cursor = 0;
            app.focused_panel = Panel::Right;
        }
        KeyCode::Char('4') => {
            app.menu_cursor = 3;
            app.detail_cursor = 0;
            app.focused_panel = Panel::Right;
        }

        // ── Segments (only in Segments section, right panel) ──────────────────
        KeyCode::Char(' ') if app.focused_panel == Panel::Right && app.menu_cursor == 0 => {
            app.toggle_cursor();
        }
        KeyCode::Char('m') if app.focused_panel == Panel::Right && app.menu_cursor == 0 => {
            // Build segment display order to find the kind at detail_cursor.
            let display_order: Vec<crate::model::SegmentKind> = {
                let mut order = app.config.segments.clone();
                for &kind in &crate::model::SegmentKind::ALL {
                    if !app.config.segments.contains(&kind) {
                        order.push(kind);
                    }
                }
                order
            };
            match display_order.get(app.detail_cursor) {
                Some(&kind) if app.config.segments.contains(&kind) => {
                    app.reorder_mode = true;
                }
                Some(_) => {
                    app.status = Some((
                        StatusKind::Warning,
                        "Enable the segment first [Space]".into(),
                    ));
                }
                None => {}
            }
        }

        // ── Thresholds (nudge and cycle, only in Thresholds section, right panel) ─
        KeyCode::Char('-') if app.focused_panel == Panel::Right && app.menu_cursor == 3 => {
            app.nudge_threshold(threshold_field_at(app.detail_cursor), -1);
        }
        KeyCode::Char('=') if app.focused_panel == Panel::Right && app.menu_cursor == 3 => {
            app.nudge_threshold(threshold_field_at(app.detail_cursor), 1);
        }
        KeyCode::Char('_') if app.focused_panel == Panel::Right && app.menu_cursor == 3 => {
            app.nudge_threshold(threshold_field_at(app.detail_cursor), -5);
        }
        KeyCode::Char('+') if app.focused_panel == Panel::Right && app.menu_cursor == 3 => {
            app.nudge_threshold(threshold_field_at(app.detail_cursor), 5);
        }
        KeyCode::Char(' ') | KeyCode::Enter
            if app.focused_panel == Panel::Right && app.menu_cursor == 3 =>
        {
            app.cycle_threshold_enum(threshold_field_at(app.detail_cursor));
        }

        // ── Global ────────────────────────────────────────────────────────────
        KeyCode::Char('s') => {
            app.save();
        }
        KeyCode::Char('r') => {
            app.request_reset();
        }
        KeyCode::Char('p') => {
            app.cycle_sample();
        }
        KeyCode::Char('P') => {
            app.cycle_sample_back();
        }
        KeyCode::Char('?') => {
            app.show_help = true;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.is_dirty() {
                app.request_quit();
            } else {
                app.should_quit = true;
            }
        }
        _ => {}
    }
}

/// Returns the ThresholdField corresponding to a detail_cursor index in the Thresholds section.
fn threshold_field_at(idx: usize) -> ThresholdField {
    match idx {
        0 => ThresholdField::Warn,
        1 => ThresholdField::Crit,
        2 => ThresholdField::WeeklyShowAt,
        3 => ThresholdField::BarWidth,
        4 => ThresholdField::ClockMode,
        _ => ThresholdField::Layout,
    }
}

/// Returns true if the terminal coordinate (col, row) is within rect.
fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

/// Handle a mouse event — click selects panel/item; scroll navigates within panel.
fn handle_mouse(app: &mut App, event: MouseEvent) {
    let left = app.left_panel_area.get();
    let right = app.right_panel_area.get();

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let col = event.column;
            let row = event.row;
            if contains(left, col, row) {
                app.focused_panel = Panel::Left;
                let inner_row = row.saturating_sub(left.y + 1) as usize;
                if inner_row < 4 {
                    app.menu_cursor = inner_row;
                    app.detail_cursor = app.detail_cursor.min(detail_len(app).saturating_sub(1));
                }
            } else if contains(right, col, row) {
                app.focused_panel = Panel::Right;
                let inner_row = row.saturating_sub(right.y + 1) as usize;
                let max = detail_len(app).saturating_sub(1);
                app.detail_cursor = inner_row.min(max);
                app.apply_move_is_select();
            }
        }
        MouseEventKind::ScrollUp => {
            if app.focused_panel == Panel::Right && app.detail_cursor > 0 {
                app.detail_cursor -= 1;
                app.apply_move_is_select();
            } else if app.focused_panel == Panel::Left && app.menu_cursor > 0 {
                app.menu_cursor -= 1;
                app.detail_cursor = 0;
            }
        }
        MouseEventKind::ScrollDown => {
            if app.focused_panel == Panel::Right {
                let max = detail_len(app).saturating_sub(1);
                if app.detail_cursor < max {
                    app.detail_cursor += 1;
                    app.apply_move_is_select();
                }
            } else if app.focused_panel == Panel::Left && app.menu_cursor < 3 {
                app.menu_cursor += 1;
                app.detail_cursor = 0;
            }
        }
        _ => {}
    }
}
