//! Full-line golden snapshots: render every fixture under the default config
//! (Tokyo Night + Powerline) with a fixed clock and a fixed `$HOME`, so the
//! output is deterministic. ESC bytes are rendered as `\e` for readable diffs.
//!
//! Update snapshots with `cargo insta review` (or `INSTA_UPDATE=always`).
//!
//! Note: fixtures whose `cwd` points at a non-existent path produce no git
//! segment (the git command fails), which keeps these snapshots independent of
//! the checkout's own git state.

use claudebar::model::{Config, InputData};
use claudebar::render::render_with;
use claudebar::{styles, themes};

/// Just before the fixtures' far-future `resets_at` epochs, so countdowns are
/// present and stable.
const FIXED_NOW: i64 = 1_899_990_000;

fn render_fixture(json: &str) -> String {
    let input = InputData::parse(json);
    let cfg = Config::default();
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let line = render_with(&input, &cfg, &theme, &style, FIXED_NOW, Some("/home/me"));
    // Readable, and proves no raw ESC from host strings leaks through (every
    // ESC in the output is one we emitted as a color code).
    line.replace('\x1b', "\\e")
}

#[test]
fn golden_lines() {
    insta::glob!(env!("CARGO_MANIFEST_DIR"), "fixtures/*.json", |path| {
        let json = std::fs::read_to_string(path).unwrap();
        insta::assert_snapshot!(render_fixture(&json));
    });
}
