//! TUI configurator: single flat-list design with six vertical zones.
//! Save writes the edited [`crate::model::Config`] back to the config path as TOML.

mod app;
mod preview;
mod sample;
mod ui;

use app::{build_list, current_section, enforce_scroll, move_segment, App, Dir, RowItem, StatusKind};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
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
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend).map_err(|e| format!("init terminal: {e}"))?;
        Ok(TerminalGuard { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
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
            if let Event::Key(key) = event::read().map_err(|e| format!("read: {e}"))? {
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key);
                }
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
        KeyCode::Char('j') | KeyCode::Char('k')
        | KeyCode::Up | KeyCode::Down
        | KeyCode::Char('h') | KeyCode::Char('l')
        | KeyCode::Char('H') | KeyCode::Char('L')
        | KeyCode::Char('1') | KeyCode::Char('2')
        | KeyCode::Char('3') | KeyCode::Char('4')
        | KeyCode::Tab | KeyCode::BackTab => {}
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
                app.status = Some((StatusKind::Error, "no save path (set $HOME or --config)".into()));
            } else {
                app.save();
                app.should_quit = true;
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        // Navigation keys: silent no-op (preserve banner).
        KeyCode::Char('j') | KeyCode::Char('k')
        | KeyCode::Up | KeyCode::Down
        | KeyCode::Char('h') | KeyCode::Char('l')
        | KeyCode::Char('H') | KeyCode::Char('L')
        | KeyCode::Char('1') | KeyCode::Char('2')
        | KeyCode::Char('3') | KeyCode::Char('4')
        | KeyCode::Tab | KeyCode::BackTab => {}
        // Any other key: silent cancel.
        _ => {
            app.pending_quit = false;
        }
    }
}

fn handle_reorder(app: &mut App, key: KeyEvent) {
    // Find the SegmentKind under cursor and its index in config.segments.
    let moved_kind = match app.cursor_row() {
        Some(RowItem::SegmentRow(k)) => *k,
        _ => {
            // Should not happen; exit reorder mode.
            app.reorder_mode = false;
            return;
        }
    };

    let seg_idx = match app.config.segments.iter().position(|s| *s == moved_kind) {
        Some(idx) => idx,
        None => {
            app.reorder_mode = false;
            return;
        }
    };

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            // Boundary check: if last in segments, no-op.
            if seg_idx + 1 >= app.config.segments.len() {
                return;
            }
            move_segment(&mut app.config.segments, seg_idx, Dir::Down);
            rebuild_and_follow(app, moved_kind);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // Boundary check: if first in segments, no-op.
            if seg_idx == 0 {
                return;
            }
            move_segment(&mut app.config.segments, seg_idx, Dir::Up);
            rebuild_and_follow(app, moved_kind);
        }
        KeyCode::Char('m') | KeyCode::Enter => {
            app.reorder_mode = false;
            // No status flash for intentional commit.
        }
        KeyCode::Esc => {
            app.reorder_mode = false;
            app.status = Some((
                StatusKind::Success,
                "Reorder committed — reorder again to undo, or [r] to reset all".into(),
            ));
        }
        // All other keys: no-op (reorder_mode consumes all input).
        _ => {}
    }
}

fn rebuild_and_follow(app: &mut App, moved_kind: crate::model::SegmentKind) {
    let (list_rows, selectable_indices, section_starts) = build_list(&app.config);
    app.list_rows = list_rows;
    app.selectable_indices = selectable_indices;
    app.section_starts = section_starts;
    // Cursor follows the moved segment.
    if let Some(new_si) = app.selectable_indices.iter().enumerate().find_map(|(si, &dr)| {
        if let RowItem::SegmentRow(k) = &app.list_rows[dr] {
            if *k == moved_kind {
                return Some(si);
            }
        }
        None
    }) {
        app.flat_cursor = new_si;
    }
    enforce_scroll(app);
}

fn handle_normal(app: &mut App, key: KeyEvent) {
    let max_idx = app.selectable_indices.len().saturating_sub(1);
    let _ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        // ── Movement ─────────────────────────────────────────────────────────
        KeyCode::Char('j') | KeyCode::Down => {
            if app.flat_cursor < max_idx {
                app.flat_cursor += 1;
                app.apply_move_is_select();
                enforce_scroll(app);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.flat_cursor > 0 {
                app.flat_cursor -= 1;
                app.apply_move_is_select();
                enforce_scroll(app);
            }
        }
        KeyCode::Char('g') => {
            app.flat_cursor = 0;
            enforce_scroll(app);
        }
        KeyCode::Char('G') => {
            app.flat_cursor = max_idx;
            enforce_scroll(app);
        }
        KeyCode::Tab => {
            let sec = current_section(app.flat_cursor, &app.section_starts);
            app.flat_cursor = app.section_starts[(sec + 1) % 4];
            enforce_scroll(app);
        }
        KeyCode::BackTab => {
            let sec = current_section(app.flat_cursor, &app.section_starts);
            app.flat_cursor = app.section_starts[(sec + 3) % 4];
            enforce_scroll(app);
        }
        KeyCode::Char('1') => {
            app.flat_cursor = app.section_starts[0];
            enforce_scroll(app);
        }
        KeyCode::Char('2') => {
            app.flat_cursor = app.section_starts[1];
            enforce_scroll(app);
        }
        KeyCode::Char('3') => {
            app.flat_cursor = app.section_starts[2];
            enforce_scroll(app);
        }
        KeyCode::Char('4') => {
            app.flat_cursor = app.section_starts[3];
            enforce_scroll(app);
        }

        // ── Segments ──────────────────────────────────────────────────────────
        KeyCode::Char(' ') => {
            if let Some(RowItem::SegmentRow(_)) = app.cursor_row() {
                app.toggle_cursor();
                enforce_scroll(app);
            }
        }
        KeyCode::Char('m') => {
            match app.cursor_row().cloned() {
                Some(RowItem::SegmentRow(kind)) if app.config.segments.contains(&kind) => {
                    app.reorder_mode = true;
                }
                Some(RowItem::SegmentRow(_)) => {
                    app.status = Some((
                        StatusKind::Warning,
                        "Enable the segment first [Space]".into(),
                    ));
                }
                _ => {}
            }
        }

        // ── Thresholds ────────────────────────────────────────────────────────
        KeyCode::Char('h') | KeyCode::Left => {
            if let Some(RowItem::ThresholdRow(field)) = app.cursor_row().cloned() {
                app.nudge_threshold(field, -1);
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if let Some(RowItem::ThresholdRow(field)) = app.cursor_row().cloned() {
                app.nudge_threshold(field, 1);
            }
        }
        KeyCode::Char('H') => {
            if let Some(RowItem::ThresholdRow(field)) = app.cursor_row().cloned() {
                app.nudge_threshold(field, -5);
            }
        }
        KeyCode::Char('L') => {
            if let Some(RowItem::ThresholdRow(field)) = app.cursor_row().cloned() {
                app.nudge_threshold(field, 5);
            }
        }

        // ── Global ────────────────────────────────────────────────────────────
        // s or Ctrl-S: save.
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
