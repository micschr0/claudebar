# Contributing to claudebar

Thanks for contributing. This file documents what we expect from a PR and the
local discipline we run before reviewers get involved.

## Development setup

- Rust toolchain: pin via the project's `rust-toolchain.toml` (CI uses the same).
- Edit `src/`, run `cargo test`. Snapshots live in `tests/snapshots/`; use
  `cargo insta review` after a snapshot diff to accept intentional changes.
- Pre-PR gate (must pass locally before opening a PR):

  ```bash
  cargo fmt --check
  cargo clippy --all-features -- -D warnings
  cargo test
  ```

## Coverage discipline

CI runs `cargo llvm-cov` on the `check` job (Ubuntu, all features), generates
`lcov.info`, and posts a per-PR coverage summary + line annotations via
[`scttnlsn/covrs`](https://github.com/scttnlsn/covrs). The HTML report is
uploaded as a workflow artifact (`coverage-html`) and is downloadable from the
Actions run page.

Run the same locally before pushing:

```bash
task coverage         # summary line, current baseline is 74.02%
task coverage-html    # browseable HTML at target/llvm-cov/html/index.html
task coverage-lcov    # lcov.info at repo root
```

### What the bot tells you

`covrs` annotates **new lines you add** with a check (not a block). It's
soft feedback — PRs that intentionally leave a branch untested can still be
merged; just call it out in the PR description. There is no project-wide
`--fail-under-lines` threshold.

### Fork PR caveat

PRs opened from forks do **not** get the bot comment. GitHub's `${{ github.token }}`
is read-only on fork PRs by design. As an external contributor, run
`task coverage` locally and read the diff before pushing — the maintainer can
still see your coverage in the upload-artifact tab of the CI run.

### Discipline

Do **not** regress the project total by more than 1 percentage point without a
note in the PR description. Coverage is a leading indicator, not the goal — a
small deliberate drop is fine when the alternative is an un-testable design.

## Intentionally low-coverage areas

- `src/tui/mod.rs`, `src/tui/ui.rs`, `src/tui/preview.rs` are at 0%. They use
  ratatui's terminal primitives that need a `TestBackend` harness. Tests for
  these belong in a **separate** follow-up project, not mixed into existing
  changes.
- Adding unit tests against `App` state (`src/tui/app.rs`) will be flaky until
  that refactor lands. Skip rather than fight it.

If your PR touches one of the segments in `src/segment/`, the render layer in
`src/render/`, or anything in `src/model/`, expect a reviewer to ask whether
new branches gained tests. The harness for each is in the matching
`#[cfg(test)] mod tests` block — match the existing patterns there.

## Conventions

- **Strict TDD**: write a failing test before the implementation change.
  Coverage expansion is mechanical when every new branch gets a test first.
- **No new runtime dependencies** without a discussion in the PR — the binary
  stays lean. Tests may add dev-dependencies freely.
- **`Feature gating**: anything TUI-only stays behind `#[cfg(feature = "tui")]`
  so a `--no-default-features` build is render-only.
- **Module docs** (`//!`) on every module; `///` on every `pub` item; non-obvious
  guards get a `//` comment explaining why.

## Reporting issues

Use the GitHub issue templates. For security reports, follow `SECURITY.md` —
do not file a public issue.

## Commit hygiene

- One logical change per commit. Squash locally before pushing if your branch
  accumulated exploratory commits.
- Conventional Commits subject line (`feat:`, `fix:`, `chore:`, `docs:` …).
- No `Co-Authored-By:` trailer.
