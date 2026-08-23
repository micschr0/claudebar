# Testing overview

Layered test strategy: unit per module → integration → golden snapshots →
bats suites → benchmark SLO.

## Pyramid

| Layer | Where | Coverage |
|---|---|---|
| Unit | `#[cfg(test)]` in each `src/` module | formatters, threshold bands, coercion, parse tables, TUI app logic |
| Integration | `tests/cli_main_dispatch.rs` (404 ln), `tests/cli_smoke.rs` (62 ln) | subcommand dispatch, stdin/stdout contract |
| Golden | `tests/render_golden.rs` (170 ln) + `tests/snapshots/` (insta) | exact ANSI output per fixture |
| Bash | `tests/statusline.bats`, `tests/install.bats` (246), `tests/scripts.bats` (41), `tests/demo_repos.bats` (71) | bash fallback contracts, install branches, script tooling |
| SLO | `scripts/benchmark.sh` | p95 < 100 ms, subprocess ≤ 5 |

## Fixtures

`fixtures/` — `typical.json`, `over_limit_5h.json`, `over_100_context.json`,
`bad_types.json`, `injection.json`, `no_git.json`, `missing_resets.json`,
`weekly_at_50.json`, `empty.json`, …

## Updating golden snapshots

```sh
cargo insta review   # review pending snapshots
# or
INSTA_UPDATE=always cargo test
```

Snapshots live in `tests/snapshots/`.

## Running bats

```sh
bats tests/statusline.bats
bats tests/install.bats
```

Requires `jq`/`git` on PATH (bash fallback deps).

## Coverage discipline

- Baseline ~74% via `llvm-cov` (`covrs`).
- TUI `mod.rs`/`ui.rs`/`preview.rs` are **0% by design** (UI shell; verified
  by screenshot scripts) — stated in `CONTRIBUTING.md:56-62`.
- `app.rs` has ~25 pure-logic unit tests; `sample.rs` one.

## What to watch out for

- Golden snapshots must stay deterministic — `now`/`home` injection is the
  mechanism.
- New segment behavior → unit tests + golden coverage.
- Bash fallback changes → `statusline.bats`.