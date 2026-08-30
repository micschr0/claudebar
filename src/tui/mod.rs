//! TUI configurator: split-pane design with left menu panel, right detail panel, and bottom preview.
//! Save writes the edited [`crate::model::Config`] back to the config path as TOML.

mod app;
mod preview;
mod sample;
mod ui;

use app::{App, Dir, Panel, StatusKind, ThresholdField, detail_len, move_segment};
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
///
/// # Errors
///
/// Returns `Err` if the terminal cannot be entered (raw mode, alternate screen,
/// or mouse capture setup fails) or if the event loop reports a terminal error.
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
    // In reorder mode, the moved segment is always enabled, so its index in
    // config.segments equals its position in the right panel's item list.
    if let Some(idx) = app.config.segments.iter().position(|&k| k == moved_kind) {
        app.detail_cursor = idx;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Config, SegmentKind};

    /// The configurator's navigation runs entirely on `focused_panel`,
    /// `menu_cursor` and `detail_cursor`. These tests drive `handle_key` — the
    /// real dispatch the event loop calls — and assert on those fields only, so
    /// they pin behaviour rather than any particular internal representation.
    fn app() -> App {
        App::new(Config::default(), None)
    }

    fn press(app: &mut App, c: char) {
        handle_key(app, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }

    fn press_code(app: &mut App, code: KeyCode) {
        handle_key(app, KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn press_all(app: &mut App, keys: &str) {
        for c in keys.chars() {
            press(app, c);
        }
    }

    // ── Panel focus ──────────────────────────────────────────────────────────

    #[test]
    fn h_and_l_move_panel_focus() {
        let mut a = app();
        press(&mut a, 'l');
        assert_eq!(a.focused_panel, Panel::Right);
        press(&mut a, 'h');
        assert_eq!(a.focused_panel, Panel::Left);
    }

    #[test]
    fn arrow_keys_move_panel_focus() {
        let mut a = app();
        press_code(&mut a, KeyCode::Right);
        assert_eq!(a.focused_panel, Panel::Right);
        press_code(&mut a, KeyCode::Left);
        assert_eq!(a.focused_panel, Panel::Left);
    }

    #[test]
    fn tab_and_backtab_both_toggle_focus() {
        // Documented quirk: Shift-Tab toggles rather than reversing.
        let mut a = app();
        press_code(&mut a, KeyCode::Tab);
        assert_eq!(a.focused_panel, Panel::Right);
        press_code(&mut a, KeyCode::Tab);
        assert_eq!(a.focused_panel, Panel::Left);
        press_code(&mut a, KeyCode::BackTab);
        assert_eq!(a.focused_panel, Panel::Right);
        press_code(&mut a, KeyCode::BackTab);
        assert_eq!(a.focused_panel, Panel::Left);
    }

    // ── Section jumps ────────────────────────────────────────────────────────

    #[test]
    fn digits_jump_to_section_and_focus_right() {
        for (key, want) in [('1', 0), ('2', 1), ('3', 2), ('4', 3)] {
            let mut a = app();
            press(&mut a, key);
            assert_eq!(a.menu_cursor, want, "key {key} selects section {want}");
            assert_eq!(a.detail_cursor, 0, "key {key} resets the detail cursor");
            assert_eq!(
                a.focused_panel,
                Panel::Right,
                "key {key} focuses the right panel"
            );
        }
    }

    // ── Cursor movement ──────────────────────────────────────────────────────

    #[test]
    fn jk_move_menu_cursor_in_left_panel() {
        let mut a = app();
        assert_eq!(a.focused_panel, Panel::Left);
        press_all(&mut a, "jj");
        assert_eq!(a.menu_cursor, 2);
        press(&mut a, 'k');
        assert_eq!(a.menu_cursor, 1);
    }

    #[test]
    fn menu_cursor_clamps_at_both_ends() {
        let mut a = app();
        press_all(&mut a, "kkk");
        assert_eq!(a.menu_cursor, 0, "cannot go above the first section");
        press_all(&mut a, "jjjjjj");
        assert_eq!(a.menu_cursor, 3, "cannot go past the last section");
    }

    #[test]
    fn jk_move_detail_cursor_in_right_panel() {
        let mut a = app();
        press(&mut a, '1');
        press_all(&mut a, "jj");
        assert_eq!(a.detail_cursor, 2);
        press(&mut a, 'k');
        assert_eq!(a.detail_cursor, 1);
    }

    #[test]
    fn detail_cursor_clamps_at_the_end_of_the_section() {
        let mut a = app();
        press(&mut a, '3'); // Style section
        for _ in 0..50 {
            press(&mut a, 'j');
        }
        assert_eq!(a.detail_cursor, crate::styles::NAMES.len() - 1);
    }

    #[test]
    fn g_and_shift_g_jump_to_first_and_last() {
        let mut a = app();
        press(&mut a, '2'); // Theme section
        press(&mut a, 'G');
        assert_eq!(a.detail_cursor, crate::themes::NAMES.len() - 1);
        press(&mut a, 'g');
        assert_eq!(a.detail_cursor, 0);
    }

    #[test]
    fn switching_section_clamps_a_now_out_of_range_detail_cursor() {
        let mut a = app();
        press(&mut a, '2'); // Themes: 16 entries
        press(&mut a, 'G');
        let themes_last = a.detail_cursor;
        press(&mut a, 'h'); // back to the left panel
        press(&mut a, 'k'); // up to Segments: 12 entries
        assert_eq!(a.menu_cursor, 0);
        assert!(
            a.detail_cursor < themes_last,
            "detail cursor must be clamped into the shorter section"
        );
        assert_eq!(a.detail_cursor, SegmentKind::ALL.len() - 1);
    }

    // ── Theme and style selection ────────────────────────────────────────────

    #[test]
    fn moving_in_theme_section_applies_the_theme() {
        let mut a = app();
        press(&mut a, '2');
        press(&mut a, 'j');
        assert_eq!(a.config.theme, crate::themes::NAMES[1]);
        press(&mut a, 'G');
        press(&mut a, 'k');
        assert_eq!(
            a.config.theme,
            crate::themes::NAMES[crate::themes::NAMES.len() - 2]
        );
    }

    #[test]
    fn moving_in_style_section_applies_the_style() {
        let mut a = app();
        press(&mut a, '3');
        press(&mut a, 'j');
        assert_eq!(a.config.style, crate::styles::NAMES[1]);
    }

    // ── Segments ─────────────────────────────────────────────────────────────

    #[test]
    fn space_toggles_the_segment_under_the_cursor() {
        let mut a = app();
        press(&mut a, '1');
        let first = a.config.segments[0];
        press_code(&mut a, KeyCode::Char(' '));
        assert!(
            !a.config.segments.contains(&first),
            "space must disable the focused segment"
        );
        // The cursor follows the segment into the disabled group; toggling again
        // must bring it back.
        press_code(&mut a, KeyCode::Char(' '));
        assert!(a.config.segments.contains(&first));
    }

    #[test]
    fn m_enters_reorder_mode_only_for_an_enabled_segment() {
        let mut a = app();
        press(&mut a, '1');
        press(&mut a, 'm');
        assert!(a.reorder_mode, "an enabled segment can be reordered");

        let mut b = app();
        press(&mut b, '1');
        press(&mut b, 'G'); // last row is a disabled segment
        press(&mut b, 'm');
        assert!(!b.reorder_mode, "a disabled segment cannot be reordered");
        assert!(b.status.is_some(), "and the refusal is explained");
    }

    #[test]
    fn reorder_moves_the_segment_and_the_cursor_follows_it() {
        let mut a = app();
        press(&mut a, '1');
        let moved = a.config.segments[0];
        press(&mut a, 'm');
        press(&mut a, 'j');
        assert_eq!(a.config.segments[1], moved, "segment moved down one slot");
        assert_eq!(a.detail_cursor, 1, "cursor follows the moved segment");
        press(&mut a, 'k');
        assert_eq!(a.config.segments[0], moved, "and back up again");
        assert_eq!(a.detail_cursor, 0);
    }

    #[test]
    fn reorder_mode_exits_on_m_and_esc() {
        let mut a = app();
        press(&mut a, '1');
        press(&mut a, 'm');
        press(&mut a, 'm');
        assert!(!a.reorder_mode);

        press(&mut a, 'm');
        assert!(a.reorder_mode);
        press_code(&mut a, KeyCode::Esc);
        assert!(!a.reorder_mode);
    }

    // ── Thresholds ───────────────────────────────────────────────────────────

    #[test]
    fn minus_and_equals_nudge_the_focused_threshold_by_one() {
        let mut a = app();
        press(&mut a, '4');
        let before = a.config.thresholds.warn;
        press(&mut a, '=');
        assert_eq!(a.config.thresholds.warn, before + 1);
        press(&mut a, '-');
        assert_eq!(a.config.thresholds.warn, before);
    }

    #[test]
    fn underscore_and_plus_nudge_by_five() {
        let mut a = app();
        press(&mut a, '4');
        let before = a.config.thresholds.warn;
        press(&mut a, '+');
        assert_eq!(a.config.thresholds.warn, before + 5);
        press(&mut a, '_');
        assert_eq!(a.config.thresholds.warn, before);
    }

    #[test]
    fn space_cycles_an_enum_threshold() {
        let mut a = app();
        press(&mut a, '4');
        press(&mut a, 'G'); // last threshold row is the Layout enum
        let before = a.config.thresholds.layout.clone();
        press_code(&mut a, KeyCode::Char(' '));
        assert_ne!(a.config.thresholds.layout, before, "space cycles the value");
    }

    #[test]
    fn nudge_keys_do_nothing_outside_the_thresholds_section() {
        let mut a = app();
        press(&mut a, '1'); // Segments
        let before = a.config.thresholds.clone();
        press_all(&mut a, "=-+_");
        assert_eq!(a.config.thresholds, before);
    }

    // ── Preview samples ──────────────────────────────────────────────────────

    #[test]
    fn p_cycles_the_preview_sample_both_ways() {
        let mut a = app();
        let n = a.samples.len();
        press(&mut a, 'p');
        assert_eq!(a.sample_idx, 1);
        press(&mut a, 'P');
        assert_eq!(a.sample_idx, 0);
        press(&mut a, 'P');
        assert_eq!(a.sample_idx, n - 1, "wraps backwards past the start");
    }

    // ── Overlays and guards ──────────────────────────────────────────────────

    #[test]
    fn question_mark_opens_help_and_help_swallows_input() {
        let mut a = app();
        press(&mut a, '?');
        assert!(a.show_help);

        // While the overlay is up, navigation must not move underneath it.
        let menu_before = a.menu_cursor;
        press_all(&mut a, "jjj");
        assert_eq!(
            a.menu_cursor, menu_before,
            "help overlay swallows navigation"
        );

        press(&mut a, '?');
        assert!(!a.show_help);
    }

    #[test]
    fn quitting_clean_exits_but_dirty_asks_first() {
        let mut a = app();
        press(&mut a, 'q');
        assert!(a.should_quit, "a clean config quits straight away");

        let mut b = app();
        press(&mut b, '2');
        press(&mut b, 'j'); // changes the theme -> dirty
        assert!(b.is_dirty());
        press(&mut b, 'q');
        assert!(!b.should_quit, "a dirty config must not quit silently");
        assert!(b.pending_quit, "it arms the confirmation instead");
        press(&mut b, 'q');
        assert!(b.should_quit, "confirming quits");
    }

    #[test]
    fn reset_is_two_press_guarded() {
        let mut a = app();
        press(&mut a, '2');
        press(&mut a, 'j');
        assert!(a.is_dirty());
        press(&mut a, 'r');
        assert!(a.pending_reset, "first press only arms the guard");
        assert!(a.is_dirty(), "and changes nothing yet");
    }

    #[test]
    fn navigation_keys_do_not_cancel_a_pending_guard() {
        let mut a = app();
        press(&mut a, 'r');
        assert!(a.pending_reset);
        press_all(&mut a, "jk1234");
        press_code(&mut a, KeyCode::Tab);
        assert!(
            a.pending_reset,
            "navigation must preserve the confirmation banner"
        );
    }
}
