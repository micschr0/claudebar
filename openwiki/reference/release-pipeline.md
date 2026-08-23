---
type: "Reference"
title: "Release pipeline"
openwiki_generated: true
---

# Release pipeline

CalVer-based releases automated via `cargo-dist` + GitHub Actions.

## Version model

- **CalVer** — `Cargo.toml` is the source of truth; no leading zeros.
- Changelog via `cliff.toml` (git-cliff), spliced into `CHANGELOG.md`.

## Workflows

| Workflow | Purpose |
|---|---|
| `release-prep.yml` | Version resolve, bump, changelog splice, open PR, tag via GitHub App token |
| `release.yml` | `cargo-dist` plan/build/host, attestation, Homebrew tap publish, `claudebar@beta` prerelease |
| `publish-prereleases` | prerelease channel pruning |
| `benchmark.yml` | SLO benchmark |
| `pages.yml` | docs site deploy (GitHub Pages) |
| `rust.yml` | test/lint matrix (both feature configs) |
| `security.yml` | gitleaks + zizmor |
| `renovate.yml` | dependency updates (pin SHAs) |
| `verify-install.yml` | end-to-end install + render smoke |
| `openwiki-update.yml` | scheduled wiki self-maintenance |

## Homebrew tap publish (release.yml:298-410)

- `custom-homebrew-app-token` job prerequisites.
- Bottle strip, class rename to `ClaudebarBeta` with verified `re.subn` count,
  `brew style --fix`, Formulary load-test guard, byte-identical-diff no-op
  commit.
- Prerelease `@beta` naming rule.
- ⚠️ **Reapply after `dist generate`** — drift warning.

## Attestation

- Release artifacts carry provenance/attestation; `install.sh` verifies
  best-effort (see installation).

## Process

See `RELEASING.md` for the full runbook.