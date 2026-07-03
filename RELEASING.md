# Releasing claudebar

**Version 1.0.0**

**[SPEC]**
[`cargo-dist`](https://opensource.axo.dev/cargo-dist/) (the `dist` CLI) generates claudebar's release pipeline. Pushing a CalVer tag builds the four target archives, a checksum file, and a shell installer, then publishes them as a GitHub Release.

**[NOTE]**
This document is the source of truth for **how to cut a release**. Read it before tagging — the version model inverted in Phase 07 and a mismatched tag fails the release.

## AI READING INSTRUCTION

**[SPEC]** Read the `[SPEC]` and `[BUG]` tagged blocks for authoritative facts.
**[NOTE]** Read `[NOTE]` tagged blocks only if additional context is needed.
**[?]** Blocks tagged `[?]` are unverified — treat with lower confidence.

## 1. Version model: Cargo.toml is the source of truth

**[SPEC]**
`Cargo.toml` `[package] version` is the **single source of truth** for the release version.

- `dist` reads the version from the manifest and **requires the pushed git tag to equal it**. A tag that disagrees with `Cargo.toml` errors the release.
- We removed the old `cargo set-version` CI step (which derived the version from the tag). Do **not** reintroduce it. Bump `Cargo.toml` manually and deliberately **before** you tag.

Practical consequence: bump the manifest first, commit it, then tag the exact same version string.

### CalVer format — no leading zeros

**[SPEC]**
claudebar uses digit-first CalVer: `YYYY.M.PATCH` (e.g. `2026.6.24`).

- **Valid:** `2026.6.25`
- **Invalid:** `2026.06.25` — leading zeros are **not** valid semver. They break both Cargo (manifest parse) and `dist` (tag/version match). The tag has no `v` prefix.

The release trigger glob in `.github/workflows/release.yml` is `'**[0-9]+.[0-9]+.[0-9]+*'`, which matches the digit-first tag with no `v` prefix.

## 2. Release ritual

**[SPEC]**
1. **Bump the version** in `Cargo.toml`:

   ```toml
   [package]
   version = "2026.6.25"   # new version — no leading zeros
   ```

2. **Sync the lockfile and commit both files.** Run a cargo command so `Cargo.lock` picks up the new version, then commit `Cargo.toml` **and** `Cargo.lock` together. You must keep the lockfile in sync — `dist`'s `--locked`-equivalent build fails if `Cargo.lock` is stale.

   ```bash
   cargo build                 # updates Cargo.lock to the new version
   git add Cargo.toml Cargo.lock
   git commit -m "chore(release): bump version to 2026.6.25"
   ```

3. **Tag the exact version.** The tag string must equal the new `Cargo.toml` version, byte-for-byte (no `v` prefix, no leading zeros):

   ```bash
   git tag 2026.6.25
   ```

4. **Push the commit and the tag:**

   ```bash
   git push
   git push --tags
   ```

   Pushing the tag triggers `release.yml`, which builds the four archives and publishes the GitHub Release.

## 3. Local pre-flight checks

**[SPEC]**
Run these before tagging to catch problems without burning a tag:

- **`dist plan`** — offline dry-run. Prints the planned artifacts: the four `claudebar-<target>.tar.gz` archives, the per-archive `.sha256` files, the unified `sha256.sum`, and `claudebar-installer.sh`. Use `dist plan --output-format=json` to inspect the build matrix and confirm that the announced version matches `Cargo.toml`.

  ```bash
  dist plan
  ```

- **`dist generate --check`** — the same drift guard CI runs (in `rust.yml`). It fails if `.github/workflows/release.yml` is out of sync with `[workspace.metadata.dist]` in `Cargo.toml`. Run it after any change to the dist config:

  ```bash
  dist generate --check
  ```

**[NOTE]**
These static checks have a hard ceiling: they confirm the *plan* and the *workflow*, but a real release may still fail to build, upload, or install. Use the smoke-tag below to close that gap.

## 4. Smoke-tag verification (the end-to-end check)

**[SPEC]**
`dist plan` and `dist generate --check` only validate the plan and the workflow — not the published release. To verify the full pipeline end-to-end — real archives, real checksums, a correct `--version` output, and a verified `install.sh` check against `sha256.sum` — push a throwaway smoke tag, then tear it down.
1. **Bump to a throwaway version** (e.g. `2026.6.99`) and tag + push it:

   ```bash
   # in Cargo.toml: version = "2026.6.99"
   cargo build
   git add Cargo.toml Cargo.lock
   git commit -m "chore: smoke-tag 2026.6.99 (will be reverted)"
   git tag 2026.6.99
   git push && git push --tags
   ```

2. **Confirm the GitHub Release** for `2026.6.99` has:
   - four `claudebar-2026.6.99-<target>.tar.gz`... *— note:* dist names archives by target only, **`claudebar-<target>.tar.gz`** (e.g. `claudebar-x86_64-unknown-linux-musl.tar.gz`), which omits the version segment. Expect four archives, one per target.
   - the unified **`sha256.sum`** checksum file (plus per-archive `.sha256` files).
   - the **`claudebar-installer.sh`** shell installer.

3. **Download the `x86_64-unknown-linux-musl` archive** and confirm the binary reports the tag:

   ```bash
   curl -fsSL -o claudebar.tar.gz \
     https://github.com/micschr0/claudebar/releases/download/2026.6.99/claudebar-x86_64-unknown-linux-musl.tar.gz
   tar -xf claudebar.tar.gz
   ./claudebar --version   # must print 2026.6.99
   ```

4. **Verify `install.sh` against the real release.** Run the installer (or just `download_prebuilt`) against `2026.6.99` and confirm `verify_checksum` passes — i.e. install.sh fetches `sha256.sum`, finds the `claudebar-x86_64-unknown-linux-musl.tar.gz` entry, and the archive's sha256 matches.

5. **Tear it all down.** Delete the GitHub Release for `2026.6.99` (via the GitHub UI or `gh release delete 2026.6.99`), delete the remote tag, and revert the throwaway bump:

   ```bash
   git push origin :refs/tags/2026.6.99   # delete remote tag
   git tag -d 2026.6.99                    # delete local tag
   # revert the smoke-tag commit so Cargo.toml/Cargo.lock return to the real version
   git revert --no-edit HEAD               # or reset if not yet shared
   git push
   ```

Use this smoke-tag procedure to validate a `dist` release end-to-end. Run it after any change to the dist config, the target matrix, or `install.sh`'s download path.
