# Domain Pitfalls: Rust Multi-Arch CI/CD and CLI Release Automation

**Project:** claudebar
**Researched:** 2026-06-20
**Confidence:** LOW (all findings from community web sources; verify against current GitHub Actions docs before implementing)

---

## Critical Pitfalls

Mistakes that cause broken releases, corrupt terminals, or binaries that silently fail on user machines.

---

### Pitfall 1: `macos-latest` silently switched architecture

**What goes wrong:** In late 2024 GitHub changed `macos-latest` from x86_64 (macos-13) to arm64 (macos-14). Workflows that relied on `macos-latest` building for x86_64 started producing arm64 binaries — or failed entirely — without any warning.

**Why it happens:** GitHub updates the `macos-latest` alias to track their current default. You don't control when it changes.

**Consequences:** Users on Intel Macs download a binary that fails to execute ("Bad CPU type in executable"). Release matrix silently produces wrong artifacts.

**Prevention:** Always pin explicit runner versions:
- `runs-on: macos-13` for `x86_64-apple-darwin`
- `runs-on: macos-14` for `aarch64-apple-darwin`

Never use `macos-latest` in a release workflow. Use it at most in a quick smoke-test job where the arch doesn't matter.

**Detection:** Binary runs on your M1 Mac but not on an Intel Mac after a release. Or vice versa.

---

### Pitfall 2: Glibc version mismatch makes Linux binary unrunnable

**What goes wrong:** A binary compiled on `ubuntu-22.04` (glibc 2.35) fails on CentOS 7, older Debian, or any system with glibc < 2.35 with:
```
/lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_2.33' not found
```

**Why it happens:** glibc is forward-compatible (newer can run older binaries) but not backward-compatible. GitHub-hosted runners use recent Ubuntu. Users may have older systems.

**Consequences:** Your binary is broken for a large fraction of Linux users — particularly server environments, corporate Linux, embedded NAS devices, etc.

**Prevention (choose one):**
1. **Preferred: use musl targets.** Build `aarch64-unknown-linux-musl` and `x86_64-unknown-linux-musl`. Produces fully static binaries with no glibc dependency at all. For claudebar (a TUI + render tool with no jemalloc), the perf tradeoff is irrelevant.
2. **Alternative: use a manylinux2014 Docker container** (`quay.io/pypa/manylinux2014_x86_64`) as the CI container. This gives glibc 2.17 — the lowest common denominator for modern Rust toolchains.

**Detection:** `ldd --version` on your binary's host differs from target. Or: upload to a CentOS 7 VM and run it.

---

### Pitfall 3: cross-rs C dependency linker errors (EM_AARCH64 vs EM_X86_64)

**What goes wrong:** When using `cross` to build for `aarch64-unknown-linux-*` from an x86_64 host, any `build.rs` that compiles C code (or any `-sys` crate) still invokes the host C compiler. The resulting `.o` files are x86_64, then the aarch64 linker rejects them with `error: linking with 'cc' failed: EM 62` (EM_X86_64 where EM_AARCH64 was expected).

**Why it happens:** `cross` isolates the Rust toolchain but doesn't automatically set `CC` for every build script. Build scripts that call `cc::Build` or `cmake::Config` pick up the host gcc.

**Consequences:** Build fails. Often masked by error messages that don't mention the architecture mismatch directly.

**Prevention:** claudebar has no C dependencies (`Cargo.toml` shows only pure-Rust deps: serde, serde_json, toml, clap, thiserror, ratatui, crossterm, ansi-to-tui). This pitfall does not apply unless a new C-linked dep is added. If it is ever added, set `CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc` in the workflow env.

**Detection:** Linker error mentioning "EM 62" or "file format not recognized" during `cross build`.

---

### Pitfall 4: `strip = true` in Cargo.toml doesn't strip cross-compiled binaries on Linux

**What goes wrong:** `strip = true` in `[profile.release]` calls the host's `strip` binary. For native builds this is fine. For `aarch64-unknown-linux-gnu` builds via `cross`, the host `strip` is an x86_64 tool that rejects the aarch64 ELF, silently producing an unstripped binary (or failing).

**Why it happens:** Cargo's strip implementation calls the system strip, not a cross-aware one.

**Prevention:** Add a post-build step in CI that runs the correct strip binary explicitly:
```yaml
- run: aarch64-linux-gnu-strip target/aarch64-unknown-linux-gnu/release/claudebar
```
Or use musl targets, where the static binary can be stripped by any `strip` implementation. With `upload-rust-binary-action` or `cargo-zigbuild`, stripping is handled correctly automatically.

**Detection:** Compare `ls -lh` on expected stripped vs actual binary. Stripped claudebar should be under 2MB; unstripped with debug info can be 10MB+.

---

### Pitfall 5: LTO + `codegen-units=1` destroys CI cache hit rate and multiplies build time

**What goes wrong:** The Cargo.toml release profile has `lto = true` and `codegen-units = 1`. These settings disable incremental compilation entirely (LTO requires whole-crate analysis). In CI, every release build is a full rebuild. With four targets, each taking 5–10 minutes, the matrix totals 20–40 minutes per release.

**Why it happens:** LTO and incremental compilation are mutually exclusive by design.

**Consequences:** Release workflows that take 40+ minutes per run, making iterative debugging painful. macOS 10x billing multiplier amplifies cost.

**Prevention:**
- Keep `lto = true` and `codegen-units = 1` — they are correct for final release binaries.
- Use aggressive Cargo dependency caching (`actions/cache` on `~/.cargo/registry`, `~/.cargo/git`, `./target`). Cache key on `Cargo.lock`.
- Accept that release builds are slow. Only CI jobs and release jobs should use the release profile. PR/CI checks should use debug profile.
- Consider a `[profile.ci]` profile that enables LTO only for final release tags.

**Detection:** GitHub Actions job duration > 20 minutes for a 4-target release matrix is expected with LTO; plan for it.

---

## Moderate Pitfalls

---

### Pitfall 6: `--no-default-features` not tested in CI

**What goes wrong:** The render-only build (`cargo build --no-default-features`) is never tested in CI. A PR introduces a reference to `ratatui` or `crossterm` in non-feature-gated code, which compiles fine with the default `tui` feature but breaks the render-only path.

**Prevention:** Add a CI job step:
```yaml
- run: cargo build --no-default-features
```
This is distinct from `cargo test`, which tests the default (full) feature set. Both must pass.

**Detection:** `cargo build --no-default-features` fails locally after a PR that added tui-related code outside a `#[cfg(feature = "tui")]` guard.

---

### Pitfall 7: Install script arch detection failure

**What goes wrong:** `uname -m` returns `arm64` on macOS and `aarch64` on Linux for the same physical architecture. An install script that checks for `aarch64` will miss macOS arm64 users and fall back to x86 or error.

**Prevention:** In `install.sh`, normalize the arch:
```bash
ARCH=$(uname -m)
case "$ARCH" in
  aarch64|arm64) ARCH=aarch64 ;;
  x86_64|amd64)  ARCH=x86_64 ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac
```
Then construct the asset name from `$ARCH` and `$OS` separately.

**Detection:** macOS arm64 users get "No binary found for your architecture" or silently download the x86 binary (which runs under Rosetta but is slower and confusing).

---

### Pitfall 8: `GITHUB_TOKEN` missing `contents: write` permission

**What goes wrong:** The release workflow uploads binaries to a GitHub Release using `softprops/action-gh-release` or similar, but the job fails with "403 Forbidden" or "Resource not accessible by integration".

**Why it happens:** Default `GITHUB_TOKEN` permissions in GitHub Actions are read-only for most scopes. Release asset upload requires write access.

**Prevention:** Add to the release job or workflow-level permissions block:
```yaml
permissions:
  contents: write
```

**Detection:** The build succeeds but the upload step fails with a 403.

---

### Pitfall 9: crossterm panic leaves terminal in raw mode

**What goes wrong:** If `claudebar config` panics after `enable_raw_mode()`, the terminal stays in raw mode. The user's shell becomes unusable — characters don't echo, Ctrl+C may not work, line endings are wrong.

**Prevention:** claudebar already has a `TerminalGuard` RAII struct that implements `Drop` (verified in `src/tui/mod.rs`). This is the correct pattern. However, `Drop` runs *after* the panic message is printed. The panic output itself may be garbled because raw mode is still active at print time.

The extra step: install a custom panic hook that calls `disable_raw_mode()` before printing:
```rust
let default_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen);
    default_hook(info);
}));
```
Install this before `TerminalGuard::enter()`.

**Detection:** Run `claudebar config`, trigger a panic (e.g., with a deliberate `panic!()` in the event loop), and observe whether your terminal is usable afterwards.

---

### Pitfall 10: Screenshot generation Docker socket path is environment-specific

**What goes wrong:** `scripts/gen_screenshots.py` hardcodes `DOCKER_HOST=unix:///run/user/1002/docker.sock` (user 1002's socket). In CI (or on a different machine), the Docker socket is at `/var/run/docker.sock` or a different UID's socket. The script silently fails to connect to Docker.

**Prevention:**
- In CI, use the standard Docker socket: `DOCKER_HOST=unix:///var/run/docker.sock` or unset (default).
- Do not hardcode the socket path in the script. Read from `$DOCKER_HOST` env var if set, else use the default.
- For the README, use the `--svg` path which requires no Docker. Reserve PNG generation for a manual release step, not automated CI.

**Detection:** `docker info` passes but `gen_screenshots.py` reports connection refused.

---

## Minor Pitfalls

---

### Pitfall 11: aarch64-apple-darwin cannot be cross-compiled from Linux

**What goes wrong:** Unlike `aarch64-unknown-linux-gnu` (easily cross-compiled from Linux with `cross`), `aarch64-apple-darwin` requires Apple's SDK. The SDK is not freely redistributable, so there is no official Docker image for it. Tools like `osxcross` exist but require obtaining the SDK separately, which adds setup friction.

**Prevention:** Use a native `macos-14` runner for the `aarch64-apple-darwin` build. This is the only practical approach without legal and setup complexity. Accept the macOS billing cost.

---

### Pitfall 12: Binary archive naming inconsistency breaks install scripts

**What goes wrong:** The install script expects `claudebar-x86_64-unknown-linux-musl.tar.gz` but the release workflow produces `claudebar-x86_64-unknown-linux-musl-v0.1.0.tar.gz` or `claudebar_x86_64_linux.tar.gz`. The script fails to find the asset.

**Prevention:** Decide on the naming convention before writing the release workflow and install script simultaneously. Recommended: `claudebar-{target}.tar.gz` with no version in the filename (GitHub Releases already scopes by tag). The `upload-rust-binary-action` uses `{bin}-{target}` by default — match this in the install script.

---

### Pitfall 13: Insta snapshot tests break when run in CI without review step

**What goes wrong:** `cargo test` fails in CI because snapshots are missing (first run) or stale (after a render change). CI has no `cargo insta review` step, so it exits 1 and blocks the workflow.

**Prevention:** CI should use `cargo test` with `INSTA_UPDATE=unseen` (accepts new snapshots) or better, commit all snapshots in the PR that changes rendering. The correct CI mode is `INSTA_UPDATE=no` (fail on any mismatch) — treat snapshot divergence as a test failure requiring a deliberate `cargo insta review` + commit.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| GitHub Actions CI setup | macos-latest arch ambiguity | Pin macos-13 / macos-14 explicitly |
| Linux cross-compilation | glibc version mismatch | Build musl targets or use manylinux2014 container |
| Linux aarch64 cross-compile | strip not applied to cross target | Post-build strip step with aarch64 strip binary |
| Release workflow | GITHUB_TOKEN 403 on asset upload | Add `permissions: contents: write` |
| Render-only build | --no-default-features not tested | Add explicit CI job step |
| Install script | uname -m returns arm64 on macOS | Normalize arch string in case block |
| TUI config command | Panic leaves raw mode | Install panic hook before TerminalGuard |
| Screenshot CI integration | Docker socket path mismatch | Use --svg path; read DOCKER_HOST from env |
| LTO + caching | Cache always misses, slow CI | Accept slow release builds; aggressive dep caching |
| aarch64 macOS build | Cannot cross-compile from Linux | Require native macos-14 runner |

---

## Sources

- [cross-rs/cross: "Zero setup" cross compilation](https://github.com/cross-rs/cross)
- [Cross-compiling Rust on GitHub Actions](https://obviy.us/blog/cross-compiling-rust-on-gha/)
- [Building Rust binaries in CI that work with older GLIBC](https://kobzol.github.io/rust/ci/2021/05/07/building-rust-binaries-in-ci-that-work-with-older-glibc.html)
- [taiki-e/upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action)
- [GitHub Actions: macOS M1 runner available for all plans](https://github.com/actions/runner-images/issues/9254)
- [GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- [Is raw mode disabled after panic? (crossterm issue #368)](https://github.com/crossterm-rs/crossterm/issues/368)
- [Cargo Features reference](https://doc.rust-lang.org/cargo/reference/features.html)
- [openssl-sys cross-platform issues (taiki-e/upload-rust-binary-action #76)](https://github.com/taiki-e/upload-rust-binary-action/issues/76)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
