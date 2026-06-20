# Technology Stack: claudebar CI/CD Release Pipeline

**Project:** claudebar
**Researched:** 2026-06-20
**Confidence:** MEDIUM (cross-validated across multiple sources; LOW provider tier but consistent findings)

---

## Recommended Stack

### GitHub Actions: Workflow Jobs

| Job | Runner | Purpose |
|-----|--------|---------|
| `ci` | `ubuntu-latest` | cargo test, cargo clippy, cargo fmt --check |
| `release` (matrix) | see below | build + upload binaries on tag push |
| `create-release` | `ubuntu-latest` | create the GitHub Release (runs before matrix) |

### Cross-Compilation Strategy

**Use hybrid: native runners for macOS, cross-rs for Linux ARM.**

| Target | Runner | Build Method | Rationale |
|--------|--------|-------------|-----------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | native `cargo` | No cross-compile needed |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `cross` via Docker | No free ARM Linux runner; cross-rs has zero extra setup |
| `x86_64-apple-darwin` | `macos-latest` | native `cargo` | Intel available on macos-latest |
| `aarch64-apple-darwin` | `macos-latest` | native `cargo` | Apple Silicon runner available; native is faster |

**Why not cargo-zigbuild?** claudebar has no C dependencies beyond the system linker. zigbuild's main benefit is glibc version pinning for musl targets — irrelevant here since we target `-gnu`. It adds complexity without benefit for this project.

**Why not cargo-dist?** cargo-dist has an open, unresolved issue (#1378) for aarch64-unknown-linux-gnu cross-compilation. It is also opinionated about Cargo.toml metadata and generates workflows you cannot easily hand-edit. The taiki-e action pair gives equivalent output with full transparency.

**Why not musl?** The project targets `-gnu` per PROJECT.md. musl binaries are larger due to static libc linking and skip the standard glibc ABI users expect on Debian/Ubuntu. Stick with `-gnu`.

### Release Actions

| Action | Version | Purpose |
|--------|---------|---------|
| `taiki-e/create-gh-release-action` | `v1` | Creates the GitHub Release from the pushed tag; reads CHANGELOG if present |
| `taiki-e/upload-rust-binary-action` | `v1` | Builds, strips, archives, checksums, and uploads binary per matrix target |
| `houseabsolute/actions-rust-cross` | `v0` | Wraps `cross` on Linux ARM targets; falls back to native `cargo` on macOS |

These three actions compose cleanly: `create-gh-release` runs once, then `upload-rust-binary` runs in parallel per matrix target using `needs: create-release`.

### Core CI Actions

| Action | Version | Purpose |
|--------|---------|---------|
| `actions/checkout` | `v4` | Checkout |
| `dtolnay/rust-toolchain` | `@stable` | Installs Rust; replaces deprecated `actions-rs/toolchain@v1` |
| `Swatinem/rust-cache` | `v2` | Caches `~/.cargo` and `./target`; dramatically cuts CI time |
| `actions/upload-artifact` | `v4` | Passes binaries between jobs when needed |

### Artifact Naming Convention

```
claudebar-{target}.tar.gz          # archive (e.g. claudebar-aarch64-apple-darwin.tar.gz)
claudebar-{target}.tar.gz.sha256   # checksum sidecar
```

This is the taiki-e default (`{bin}-{target}`). Version is encoded in the GitHub Release tag, not repeated in the filename — consistent with ripgrep, fd, bat, and other widely-used Rust CLI tools.

**Checksum algorithm:** SHA256 only. sha1 and md5 are cryptographically broken. b2 requires an extra install on macOS runners. SHA256 is universally available and sufficient.

### Workflow File Layout

```
.github/
  workflows/
    ci.yml        # runs on push + PR: test, clippy, fmt
    release.yml   # runs on tag push v*.*.*: build matrix + GitHub Release
```

Two separate files because CI runs on every push, releases only on tags. Combining them forces every PR to evaluate release conditions unnecessarily.

---

## Matrix Configuration (release.yml)

```yaml
strategy:
  matrix:
    include:
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
      - target: aarch64-unknown-linux-gnu
        os: ubuntu-latest
      - target: x86_64-apple-darwin
        os: macos-latest
      - target: aarch64-apple-darwin
        os: macos-latest
```

For `aarch64-unknown-linux-gnu`, set `use_cross: true` in the `houseabsolute/actions-rust-cross` step. For the three native targets, `cross` is not invoked.

---

## Install Script Pattern (improved install.sh)

Current install.sh only builds from a local checkout or falls back to the bash script. It must gain a third path: download a prebuilt binary from GitHub Releases.

**Priority order:**
1. **Prebuilt binary** — detect OS + arch, download from GitHub Releases, verify SHA256
2. **cargo build** — if `cargo` is present and `Cargo.toml` is local (checkout scenario)
3. **bash fallback** — download `statusline-command.sh`

**OS/arch detection:**

```bash
OS="$(uname -s)"      # Linux | Darwin
ARCH="$(uname -m)"    # x86_64 | arm64 | aarch64

case "$ARCH" in
  arm64|aarch64) ARCH="aarch64" ;;
  x86_64)        ARCH="x86_64"  ;;
  *) ARCH="" ;;   # unsupported, skip to fallback
esac

case "$OS" in
  Linux)  PLATFORM="${ARCH}-unknown-linux-gnu" ;;
  Darwin) PLATFORM="${ARCH}-apple-darwin"      ;;
  *)      PLATFORM="" ;;
esac
```

**Download + verify:**

```bash
ARCHIVE="claudebar-${PLATFORM}.tar.gz"
URL="https://github.com/micschr0/claudebar/releases/latest/download/${ARCHIVE}"
curl -fsSL "$URL" -o "/tmp/${ARCHIVE}"
curl -fsSL "${URL}.sha256" -o "/tmp/${ARCHIVE}.sha256"

# verify
(cd /tmp && sha256sum --check "${ARCHIVE}.sha256")   # Linux
# or on macOS: shasum -a 256 --check "${ARCHIVE}.sha256"

tar -xzf "/tmp/${ARCHIVE}" -C "$HOME/.claude/"
chmod +x "$HOME/.claude/claudebar"
```

SHA256 verification is mandatory — `curl | bash` patterns are already a trust decision; the checksum at minimum guards against corrupted downloads.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| ARM Linux cross-compile | `cross-rs` | `cargo-zigbuild` | No benefit without C deps; adds zig toolchain as dependency |
| ARM Linux cross-compile | `cross-rs` | Native ARM runner (self-hosted) | Requires infrastructure; free tier has no ARM Linux |
| Release workflow | `taiki-e` action pair | `cargo-dist` | cargo-dist has open aarch64-gnu issue (#1378); generated workflows are opaque |
| Release workflow | `taiki-e` action pair | Custom workflow (softprops + manual tar) | More boilerplate; no automatic checksum generation |
| Rust toolchain setup | `dtolnay/rust-toolchain@stable` | `actions-rs/toolchain@v1` | `actions-rs/toolchain` is deprecated and unmaintained |
| Dependency caching | `Swatinem/rust-cache@v2` | `actions/cache@v4` with manual keys | rust-cache handles all Cargo cache locations automatically |

---

## Sources

- [houseabsolute/actions-rust-cross](https://github.com/houseabsolute/actions-rust-cross) — cross-rs wrapper action
- [taiki-e/upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action) — binary upload action with checksum
- [taiki-e/setup-cross-toolchain-action](https://github.com/taiki-e/setup-cross-toolchain-action) — cross toolchain setup (alternative to cross-rs)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) — modern Rust toolchain action
- [Rust Cross-Compilation With GitHub Actions - reemus.dev](https://reemus.dev/tldr/rust-cross-compilation-github-actions)
- [Cross-compiling Rust from GitHub Actions - ahmedjama.com (Dec 2025)](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/)
- [cargo-dist aarch64-linux-gnu issue #1378](https://github.com/axodotdev/cargo-dist/issues/1378) — open issue, reason to avoid cargo-dist
- [cross-rs/cross](https://github.com/cross-rs/cross) — zero-setup Rust cross compilation
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) — alternative cross-linker
- [Fully Automated Releases for Rust - orhun.dev](https://blog.orhun.dev/automated-rust-releases/)
- [My New GitHub Action for Releasing Rust Projects - urth.org (Oct 2024)](https://blog.urth.org/2024/10/27/my-new-github-action-for-releasing-rust-projects/)
