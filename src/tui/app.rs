//! Configurator state and the pure logic that mutates it. Drawing lives in
//! `ui.rs`; this module is kept free of ratatui draw calls so its helpers can be
//! unit-tested directly.

use crate::model::{Config, SegmentKind};
use crate::tui::sample::{self, Sample};
use std::path::PathBuf;

/// Severity of a transient status message — drives the rendering color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusKind {
    Success,
    Warning,
    Error,
}

/// Direction a reorder moves the focused segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dir {
    Up,
    Down,
}

/// A threshold field in the TUI configurator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThresholdField {
    Warn,
    Crit,
    WeeklyShowAt,
    BarWidth,
    ClockMode,
    Layout,
}

/// Identifies which of the two top-row panels is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Panel {
    Left,
    Right,
}

/// Full configurator state.
pub(crate) struct App {
    pub config: Config,
    pub save_path: Option<PathBuf>,
    /// Snapshot at new()/save()/reset() used by is_dirty().
    pub saved_config: Config,
    /// Which panel (Left/Right) currently has keyboard focus.
    pub focused_panel: Panel,
    /// 0–3: which section is highlighted in the left panel.
    pub menu_cursor: usize,
    /// Position within the right panel's section item list, 0-indexed.
    pub detail_cursor: usize,
    /// Last drawn area of the left panel — set by draw(), read by mouse handler.
    pub left_panel_area: std::cell::Cell<ratatui::layout::Rect>,
    /// Last drawn area of the right panel — set by draw(), read by mouse handler.
    pub right_panel_area: std::cell::Cell<ratatui::layout::Rect>,
    /// Swatch cache built in themes::NAMES order.
    /// Slot order: [separator, dir, git_branch, bar_ok, bar_crit, model].
    pub swatch_cache: Vec<[u8; 6]>,
    pub samples: Vec<Sample>,
    pub sample_idx: usize,
    /// Transient status message with a severity level for color-coding.
    pub status: Option<(StatusKind, String)>,
    /// True when a destructive reset is awaiting confirmation.
    pub pending_reset: bool,
    /// True when a quit with unsaved changes is awaiting confirmation.
    pub pending_quit: bool,
    /// True while a segment is being reordered with j/k.
    pub reorder_mode: bool,
    /// True when the help overlay is visible.
    pub show_help: bool,
    pub should_quit: bool,
}

impl App {
    /// Build state from a loaded config and its resolved save path.
    pub(crate) fn new(config: Config, save_path: Option<PathBuf>) -> App {
        let saved_config = config.clone();

        // Swatch cache built in themes::NAMES order — draw_list() indexes by themes::NAMES position.
        // Slot order: [separator, dir, git_branch, bar_ok, bar_crit, model].
        // Reordering themes::NAMES or changing Theme struct fields requires updating here.
        let swatch_cache: Vec<[u8; 6]> = crate::themes::NAMES
            .iter()
            .map(|name| {
                let t = crate::themes::get(name);
                [
                    t.separator.0,
                    t.dir.0,
                    t.git_branch.0,
                    t.bar_ok.0,
                    t.bar_crit.0,
                    t.model.0,
                ]
            })
            .collect();

        App {
            config,
            save_path,
            saved_config,
            focused_panel: Panel::Left,
            menu_cursor: 0,
            detail_cursor: 0,
            left_panel_area: std::cell::Cell::new(ratatui::layout::Rect::default()),
            right_panel_area: std::cell::Cell::new(ratatui::layout::Rect::default()),
            swatch_cache,
            samples: sample::all(),
            sample_idx: 0,
            status: None,
            pending_reset: false,
            pending_quit: false,
            reorder_mode: false,
            show_help: false,
            should_quit: false,
        }
    }

    /// True if the current config differs from the last save/reset snapshot.
    pub(crate) fn is_dirty(&self) -> bool {
        self.config != self.saved_config
    }

    /// The currently displayed preview sample.
    pub(crate) fn current_sample(&self) -> &Sample {
        &self.samples[self.sample_idx]
    }

    /// Advance the preview sample forward.
    pub(crate) fn cycle_sample(&mut self) {
        self.sample_idx = (self.sample_idx + 1) % self.samples.len();
    }

    /// Advance the preview sample backward.
    pub(crate) fn cycle_sample_back(&mut self) {
        self.sample_idx = (self.sample_idx + self.samples.len() - 1) % self.samples.len();
    }

    /// Arm the two-press reset guard. Does NOT write to self.status.
    pub(crate) fn request_reset(&mut self) {
        self.pending_reset = true;
    }

    /// Arm the two-press quit guard. Does NOT write to self.status.
    pub(crate) fn request_quit(&mut self) {
        self.pending_quit = true;
    }

    /// Reset config to defaults and send both cursors home.
    pub(crate) fn reset(&mut self) {
        self.config = Config::default();
        self.saved_config = Config::default();
        self.menu_cursor = 0;
        self.detail_cursor = 0;
        self.focused_panel = Panel::Left;
        // Status intentionally NOT set here — caller handles display.
    }

    /// Persist the config to save_path, recording outcome in status.
    pub(crate) fn save(&mut self) {
        match &self.save_path {
            Some(path) => match self.config.save(path) {
                Ok(()) => {
                    let display = path.display().to_string();
                    self.saved_config = self.config.clone();
                    self.status = Some((StatusKind::Success, format!("saved to {display}")));
                }
                Err(e) => {
                    self.status = Some((StatusKind::Error, format!("save failed: {e}")));
                }
            },
            None => {
                self.status = Some((
                    StatusKind::Warning,
                    "no save path (set $HOME or --config)".into(),
                ));
            }
        }
    }

    /// Toggle the segment under `detail_cursor`; the cursor follows it.
    /// `detail_cursor` indexes into the display order (enabled first, then
    /// disabled in `SegmentKind::ALL` order).
    pub(crate) fn toggle_cursor(&mut self) {
        // Build display order for segments: enabled in config.segments order, then disabled in ALL order.
        let display_order: Vec<SegmentKind> = {
            let mut order: Vec<SegmentKind> = self.config.segments.clone();
            for &kind in &SegmentKind::ALL {
                if !self.config.segments.contains(&kind) {
                    order.push(kind);
                }
            }
            order
        };

        let kind = match display_order.get(self.detail_cursor) {
            Some(&k) => k,
            None => return,
        };

        toggle_segment(&mut self.config.segments, kind);

        // Update detail_cursor to follow the toggled segment in new display order.
        let new_display_order: Vec<SegmentKind> = {
            let mut order: Vec<SegmentKind> = self.config.segments.clone();
            for &kind in &SegmentKind::ALL {
                if !self.config.segments.contains(&kind) {
                    order.push(kind);
                }
            }
            order
        };
        if let Some(idx) = new_display_order.iter().position(|&k| k == kind) {
            self.detail_cursor = idx;
        }
    }

    /// Nudge a threshold field by delta, with mutual clamping.
    pub(crate) fn nudge_threshold(&mut self, field: ThresholdField, delta: i16) {
        let t = &mut self.config.thresholds;
        match field {
            ThresholdField::Warn => {
                let val = (t.warn as i16 + delta).max(0) as u16;
                t.warn = val.clamp(1, t.crit.saturating_sub(1).max(1));
            }
            ThresholdField::Crit => {
                let val = (t.crit as i16 + delta).max(0) as u16;
                t.crit = val.clamp(t.warn.saturating_add(1), 99);
            }
            ThresholdField::WeeklyShowAt => {
                let val = (t.weekly_show_at as i16 + delta).max(0) as u16;
                t.weekly_show_at = val.clamp(1, 99);
            }
            ThresholdField::BarWidth => {
                let val = (i16::from(t.bar_width) + delta).clamp(2, 20) as u8;
                t.bar_width = val;
            }
            ThresholdField::ClockMode | ThresholdField::Layout => {
                // Nudging has no effect on enum-cycled fields; use cycle_threshold_enum.
            }
        }
    }

    /// Cycle a threshold enum-typed field through its options (clock_mode, layout).
    pub(crate) fn cycle_threshold_enum(&mut self, field: ThresholdField) {
        match field {
            ThresholdField::ClockMode => {
                self.config.thresholds.clock_mode = match self.config.thresholds.clock_mode.as_str()
                {
                    "auto" => "12h".into(),
                    "12h" => "24h".into(),
                    "24h" => "off".into(),
                    _ => "auto".into(),
                };
            }
            ThresholdField::Layout => {
                self.config.thresholds.layout = match self.config.thresholds.layout.as_str() {
                    "fixed" => "auto".into(),
                    _ => "fixed".into(),
                };
            }
            _ => {}
        }
    }

    /// Apply move-is-select for theme/style sections after navigation.
    /// In the two-panel model, keyed to menu_cursor + detail_cursor.
    pub(crate) fn apply_move_is_select(&mut self) {
        match self.menu_cursor {
            1 => {
                if let Some(&name) = crate::themes::NAMES.get(self.detail_cursor) {
                    self.config.theme = name.to_string();
                }
            }
            2 => {
                if let Some(&name) = crate::styles::NAMES.get(self.detail_cursor) {
                    self.config.style = name.to_string();
                }
            }
            // Segments and Thresholds have nothing to select on move.
            _ => {}
        }
    }

    pub(crate) fn context_help(&self) -> Option<&'static str> {
        if self.focused_panel != Panel::Right {
            return None;
        }
        match self.menu_cursor {
            0 => {
                let mut order: Vec<SegmentKind> = self.config.segments.clone();
                for &kind in &SegmentKind::ALL {
                    if !self.config.segments.contains(&kind) {
                        order.push(kind);
                    }
                }
                order
                    .get(self.detail_cursor)
                    .map(|&kind| segment_help(kind))
            }
            3 => {
                const ORDER: [ThresholdField; 6] = [
                    ThresholdField::Warn,
                    ThresholdField::Crit,
                    ThresholdField::WeeklyShowAt,
                    ThresholdField::BarWidth,
                    ThresholdField::ClockMode,
                    ThresholdField::Layout,
                ];
                ORDER
                    .get(self.detail_cursor)
                    .map(|&field| threshold_help(field))
            }
            _ => None,
        }
    }
}

/// Enable seg (append) if absent, else disable it (remove).
pub(crate) fn toggle_segment(segments: &mut Vec<SegmentKind>, seg: SegmentKind) {
    if let Some(idx) = segments.iter().position(|s| *s == seg) {
        segments.remove(idx);
    } else {
        segments.push(seg);
    }
}

/// Swap the element at idx with its neighbor in dir. Out-of-range or
/// boundary moves are no-ops.
pub(crate) fn move_segment(segments: &mut [SegmentKind], idx: usize, dir: Dir) {
    if idx >= segments.len() {
        return;
    }
    match dir {
        Dir::Up if idx > 0 => segments.swap(idx, idx - 1),
        Dir::Down if idx + 1 < segments.len() => segments.swap(idx, idx + 1),
        _ => {}
    }
}

/// Number of items in the right panel for the section at app.menu_cursor.
pub(crate) fn detail_len(app: &App) -> usize {
    match app.menu_cursor {
        0 => crate::model::SegmentKind::ALL.len(),
        1 => crate::themes::NAMES.len(),
        2 => crate::styles::NAMES.len(),
        3 => 6, // ThresholdField variants: Warn, Crit, WeeklyShowAt, BarWidth, ClockMode, Layout
        _ => 0,
    }
}

/// Human-readable description for each segment, shown in the status line
/// when the segment is selected in the right panel.
pub(crate) fn segment_help(kind: SegmentKind) -> &'static str {
    match kind {
        SegmentKind::Directory => "Directory — current working directory",
        SegmentKind::Git => "Git — current branch and working-tree status",
        SegmentKind::Model => "Model — active Claude model and reasoning effort",
        SegmentKind::Context => "Context — token usage vs. context window budget",
        SegmentKind::RateLimits => "Rate Limits — 5-hour and 7-day API rate-limit usage",
        SegmentKind::DevContext => "Dev Context — current development context name",
        SegmentKind::Cost => "Cost — session cost in USD",
        SegmentKind::Lines => "Lines — lines added and removed this session",
        SegmentKind::Duration => "Duration — session wall-clock duration",
        SegmentKind::Burn => "Burn — cumulative session cost",
        SegmentKind::Clock => "Clock — current time (12h/24h, configurable seconds)",
        SegmentKind::UpdateNotice => {
            "Update Notice — badge when a newer release exists (checks once a day)"
        }
    }
}

fn threshold_help(field: ThresholdField) -> &'static str {
    match field {
        ThresholdField::Warn => "warn — bar turns yellow above this context usage %",
        ThresholdField::Crit => "crit — bar turns red above this context usage %",
        ThresholdField::WeeklyShowAt => {
            "weekly_show_at — weekly window shown once usage reaches this %"
        }
        ThresholdField::BarWidth => "bar_width — width of progress bars in cells",
        ThresholdField::ClockMode => "clock_mode — 12h, 24h, or off",
        ThresholdField::Layout => "layout — fixed (single line) or auto (responsive wrap)",
    }
}

#[cfg(test)]
mod tests {

    /// Every `SegmentKind` needs a help line — a new variant added without one
    /// would otherwise ship an empty description in the configurator.
    #[test]
    fn every_segment_kind_has_help_text() {
        for &kind in &SegmentKind::ALL {
            let help = segment_help(kind);
            assert!(!help.is_empty(), "{kind:?}: empty help");
            assert!(
                help.contains(kind.label()),
                "{kind:?}: help should name the segment"
            );
        }
    }
    use super::*;

    #[test]
    fn toggle_enables_absent_segment() {
        let mut segs = vec![SegmentKind::Directory];
        toggle_segment(&mut segs, SegmentKind::Git);
        assert_eq!(segs, vec![SegmentKind::Directory, SegmentKind::Git]);
    }

    #[test]
    fn toggle_disables_present_segment() {
        let mut segs = vec![SegmentKind::Directory, SegmentKind::Git];
        toggle_segment(&mut segs, SegmentKind::Directory);
        assert_eq!(segs, vec![SegmentKind::Git]);
    }

    #[test]
    fn toggle_is_involutive() {
        let mut segs = vec![SegmentKind::Context];
        toggle_segment(&mut segs, SegmentKind::Git);
        toggle_segment(&mut segs, SegmentKind::Git);
        assert_eq!(segs, vec![SegmentKind::Context]);
    }

    #[test]
    fn move_up_swaps_with_predecessor() {
        let mut segs = vec![SegmentKind::Directory, SegmentKind::Git, SegmentKind::Model];
        move_segment(&mut segs, 1, Dir::Up);
        assert_eq!(
            segs,
            vec![SegmentKind::Git, SegmentKind::Directory, SegmentKind::Model]
        );
    }

    #[test]
    fn move_down_swaps_with_successor() {
        let mut segs = vec![SegmentKind::Directory, SegmentKind::Git, SegmentKind::Model];
        move_segment(&mut segs, 1, Dir::Down);
        assert_eq!(
            segs,
            vec![SegmentKind::Directory, SegmentKind::Model, SegmentKind::Git]
        );
    }

    #[test]
    fn move_at_boundary_is_noop() {
        let mut segs = vec![SegmentKind::Directory, SegmentKind::Git];
        move_segment(&mut segs, 0, Dir::Up);
        move_segment(&mut segs, 1, Dir::Down);
        assert_eq!(segs, vec![SegmentKind::Directory, SegmentKind::Git]);
    }

    #[test]
    fn move_out_of_range_is_noop() {
        let mut segs = vec![SegmentKind::Directory];
        move_segment(&mut segs, 5, Dir::Up);
        assert_eq!(segs, vec![SegmentKind::Directory]);
    }

    #[test]
    fn new_syncs_theme_style_cursors() {
        let cfg = Config {
            theme: "nord".into(),
            style: "ascii".into(),
            ..Config::default()
        };
        let app = App::new(cfg, None);
        assert_eq!(app.config.theme, "nord");
        assert_eq!(app.config.style, "ascii");
    }

    #[test]
    fn reset_restores_defaults_and_cursors() {
        let mut app = App::new(Config::default(), None);
        app.config.theme = "dracula".into();
        app.config.segments.clear();
        app.reset();
        assert_eq!(app.config, Config::default());
        assert!(!app.is_dirty());
    }

    #[test]
    fn save_clears_dirty() {
        let dir = std::env::temp_dir();
        let path = dir.join("claudebar_test_save.toml");
        let mut app = App::new(Config::default(), Some(path.clone()));
        // Make config dirty.
        app.config.theme = "nord".into();
        assert!(app.is_dirty());
        app.save();
        assert!(!app.is_dirty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cycle_sample_wraps() {
        let mut app = App::new(Config::default(), None);
        let n = app.samples.len();
        for _ in 0..n {
            app.cycle_sample();
        }
        assert_eq!(app.sample_idx, 0);
    }

    #[test]
    fn request_reset_arms_pending_flag() {
        let mut app = App::new(Config::default(), None);
        app.request_reset();
        assert!(app.pending_reset);
        // New design: status is NOT written by request_reset().
    }

    #[test]
    fn request_quit_arms_pending_flag() {
        let mut app = App::new(Config::default(), None);
        app.request_quit();
        assert!(app.pending_quit);
        // New design: status is NOT written by request_quit().
    }
}
