# Architecture: GitHub Actions Release Pipeline

**Domain:** Rust CLI tool — CI + multi-arch release
**Researched:** 2026-06-20
**Confidence:** MEDIUM (cross-checked web sources)

## Recommended Architecture

Two separate workflow files. CI is fast and always-on; release is slow and tag-gated. Never combine them into one file — that either slows every PR or forces CI to run twice.

```
.github/
  workflows/
    ci.yml       # push + PR → check, lint, test
    release.yml  # v* tag → build matrix → GitHub Release
  release.yml    # optional: category config for auto-generated notes
```

### Workflow Responsibilities

| File | Trigger | Jobs | Purpose |
|------|---------|------|---------|
| `ci.yml` | `push`, `pull_request` | `check`, `test` | Fast feedback — fails before release |
| `release.yml` | `push tags: v*` | `build` (matrix), `release` | Cross-compile all targets, publish |

---

## CI Workflow (`ci.yml`)

### Jobs

```
check  ──┐
         ├──► test
lint   ──┘
```

- **check**: `cargo check --all-targets` — catches compile errors fast without full build
- **lint**: `cargo clippy -- -D warnings` + `shellcheck statusline-command.sh` — run on `ubuntu-latest` only (no need to duplicate across OS)
- **test**: `cargo test` — run on both `ubuntu-latest` and `macos-latest` to catch platform differences; snapshot tests via `insta` are included here

### Rust Toolchain

Use `dtolnay/rust-toolchain@stable` (not `actions-rs/toolchain` — unmaintained). Pin to `stable`, add `clippy` and `rustfmt` components.

### Caching

Use `Swatinem/rust-cache@v2`. Caches `~/.cargo/registry`, `~/.cargo/git`, and `target/` keyed on `Cargo.lock`. This is the community standard for Rust GHA caching — drop-in, correct invalidation, no manual key management.

---

## Release Workflow (`release.yml`)

### Job Dependency Graph

```
build (matrix: 4 targets, parallel)
  └─► release (needs: [build], downloads all artifacts)
```

The `release` job only runs after all 4 build matrix jobs succeed. If any target fails to compile, no release is created.

### Permissions

```yaml
permissions:
  contents: write   # required by softprops/action-gh-release
```

Set at the job level on `release`, not workflow level — principle of least privilege.

---

## Matrix Strategy

Use `matrix.include` (not `matrix.os` × `matrix.target`) because each target has a specific runner and cross-compilation requirement that doesn't compose cleanly as a product.

```yaml
strategy:
  matrix:
    include:
      - target: x86_64-unknown-linux-gnu
        runner: ubuntu-latest
        archive: tar.gz
      - target: aarch64-unknown-linux-gnu
        runner: ubuntu-latest
        archive: tar.gz
      - target: x86_64-apple-darwin
        runner: macos-latest
        archive: tar.gz
      - target: aarch64-apple-darwin
        runner: macos-latest
        archive: tar.gz
```

### Cross-Compilation Tool

Use `houseabsolute/actions-rust-cross@v0` for all four targets. It auto-selects:
- `cargo` for `x86_64-unknown-linux-gnu` (native on ubuntu runner)
- `cross` (Docker-based) for `aarch64-unknown-linux-gnu` (cross-arch on ubuntu runner)
- `cargo` for both macOS targets (macos-latest supports both via Xcode toolchain)

This is correct for claudebar because the project has no native C dependencies beyond standard glibc — `serde`, `toml`, `clap`, `ratatui`, `crossterm` are all pure Rust or use standard platform libs. Do not use `cargo-zigbuild` — it has known compatibility issues with some projects and the `cross` approach has broader community adoption.

---

## Artifact Packaging

### Naming Convention

```
claudebar-{version}-{target}.tar.gz
```

Examples:
- `claudebar-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `claudebar-0.1.0-aarch64-apple-darwin.tar.gz`

Extract `{version}` from the git tag: `${{ github.ref_name }}` strips the `v` prefix with `${GITHUB_REF_NAME#v}` in shell, or reference `${{ github.ref_name }}` directly and let the filename include the `v` — both conventions exist, strip-v is more common for install scripts.

### Archive Contents

Each `.tar.gz` must include:
1. `claudebar` binary (the release build output from `target/{target}/release/claudebar`)
2. `README.md`
3. `LICENSE`

Create a staging directory per build to keep the archive clean:

```bash
mkdir -p dist
cp target/${{ matrix.target }}/release/claudebar dist/
cp README.md LICENSE dist/
cd dist
tar czf ../claudebar-${{ github.ref_name }}-${{ matrix.target }}.tar.gz .
```

### Checksums

Generate per-artifact checksums in the build job, then aggregate in the release job. In each build matrix job:

```bash
sha256sum claudebar-*.tar.gz > claudebar-${{ matrix.target }}.sha256
```

Upload both the `.tar.gz` and the `.sha256` as artifacts. In the release job, after `actions/download-artifact@v4` pulls everything down, concatenate into a single `SHA256SUMS.txt`:

```bash
cat *.sha256 > SHA256SUMS.txt
```

Upload `SHA256SUMS.txt` alongside all archives in the release.

### Artifact Upload Between Jobs

Build job:
```yaml
- uses: actions/upload-artifact@v4
  with:
    name: claudebar-${{ matrix.target }}
    path: |
      claudebar-*.tar.gz
      *.sha256
```

Release job:
```yaml
- uses: actions/download-artifact@v4
  with:
    path: artifacts/
    merge-multiple: true
```

`merge-multiple: true` flattens all per-target artifact directories into one flat directory — required when each matrix leg uploads under a different name.

---

## Release Creation

Use `softprops/action-gh-release@v2`. Set `generate_release_notes: true` — this uses GitHub's native release notes generator, which groups PRs since the last tag by label. Zero additional config needed for a single-maintainer project.

```yaml
- uses: softprops/action-gh-release@v2
  with:
    generate_release_notes: true
    files: |
      artifacts/*.tar.gz
      artifacts/SHA256SUMS.txt
```

If more control is needed over release note categories, add `.github/release.yml` with a `changelog` section — but this is optional for initial release.

Do not use the old `actions/create-release` + `actions/upload-release-asset` pattern — both are deprecated and unmaintained.

---

## Patterns to Follow

### Pattern: Extract version from tag in shell

```bash
VERSION="${GITHUB_REF_NAME#v}"   # strips leading v from v0.1.0 → 0.1.0
ARCHIVE="claudebar-${VERSION}-${{ matrix.target }}.tar.gz"
```

Use this in the packaging step so the filename matches `claudebar-0.1.0-*` not `claudebar-v0.1.0-*`.

### Pattern: Build with locked deps

```yaml
args: --release --locked
```

`--locked` ensures CI uses the exact versions in `Cargo.lock`, preventing "works locally, breaks in CI" surprises from yanked crates.

### Pattern: Feature flag for release build

The `tui` feature is the default and should be included in release builds. The Cargo.toml already has `lto = true`, `strip = true`, `codegen-units = 1` in `[profile.release]` — no additional flags needed in the workflow.

---

## Anti-Patterns to Avoid

### Anti-Pattern: Single combined workflow

**What:** One file with CI jobs and release jobs, triggered on both push and tag.
**Why bad:** PRs trigger slow cross-compilation builds. Tag pushes run linting redundantly. Release blocks on CI matrix, not just relevant checks.
**Instead:** Two files. CI ignores tags (`branches: [main]`); release ignores branches.

### Anti-Pattern: Global `permissions: write-all`

**What:** Setting `permissions: write-all` at the top level.
**Why bad:** Any compromised action in CI gets write access to the repo.
**Instead:** `permissions: contents: write` on the release job only.

### Anti-Pattern: `cargo install cross` in the workflow

**What:** Running `cargo install cross` before each build job.
**Why bad:** `cross` takes ~90 seconds to compile from source on every run.
**Instead:** Use `houseabsolute/actions-rust-cross` — it installs a prebuilt binary of `cross` and caches it.

### Anti-Pattern: Single artifact upload for all targets

**What:** Uploading all 4 archives under one artifact name.
**Why bad:** `actions/upload-artifact@v4` requires unique names per matrix leg; collisions either error or overwrite.
**Instead:** Name artifacts per target: `claudebar-${{ matrix.target }}`, then use `merge-multiple: true` in the download step.

### Anti-Pattern: `uses: actions-rs/*` actions

**What:** Using any action from the `actions-rs` org (actions-rs/toolchain, actions-rs/cargo, actions-rs/clippy-check).
**Why bad:** The entire `actions-rs` organization is unmaintained and archived since 2023.
**Instead:** `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `houseabsolute/actions-rust-cross`.

---

## Scalability Considerations

| Concern | Now (single maintainer) | If OSS grows |
|---------|------------------------|--------------|
| Build time | ~8–12 min per release (4 parallel targets) | Add `Swatinem/rust-cache` to release jobs too |
| Windows support | Skip — no demand per PROJECT.md | Add `x86_64-pc-windows-msvc` matrix entry + `.zip` packaging |
| Package managers | Skip — post-release | Homebrew tap via separate workflow on release published event |
| MSRV testing | Not required yet | Add `toolchain: 1.xx` matrix dimension to CI |

---

## Sources

- [houseabsolute/actions-rust-cross](https://github.com/houseabsolute/actions-rust-cross) — MEDIUM confidence
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release) — MEDIUM confidence
- [Building a cross platform Rust CI/CD pipeline with GitHub Actions](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/) — MEDIUM confidence
- [Rust Cross-Compilation With GitHub Actions](https://reemus.dev/tldr/rust-cross-compilation-github-actions) — MEDIUM confidence
- [How to Deploy Rust Binaries with GitHub Actions](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/) — MEDIUM confidence
- [GitHub Actions best practices for Rust projects](https://www.infinyon.com/blog/2021/04/github-actions-best-practices/) — MEDIUM confidence (older, patterns still valid)
