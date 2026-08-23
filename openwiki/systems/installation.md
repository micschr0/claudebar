# Installation

`install.sh` — one-command install with supply-chain verification.

## Flow

1. Channel select: stable / beta.
2. `detect_target` (OS/arch).
3. TLS 1.2+ `curl` fetch of the release asset.
4. Checksum verify (`sha256sum`/`shasum`).
5. Attestation verify via `gh` (non-fatal if unavailable/stale).
6. Asset-URL by regex; unsafe-path archive guard.
7. `install_binary`, `link_onto_path`.
8. `setup` wiring (statusLine patch).
9. Nerd Font report.

## Verification layers

- **Fatal**: SHA256 checksum mismatch → abort.
- **Best-effort**: provenance attestation (requires `gh`, `--signer-workflow`
  scoping vs `--repo`).
- Accepted checksum formats pinned in `install.bats`.

## Alternate install paths

- Homebrew / mise (tap — see release-pipeline).
- `cargo build --release` fallback (needs toolchain).

## CI verification

- `verify-install.yml` — end-to-end install + render smoke on fresh VMs.

## Tests

- `tests/install.bats` (246 lines): each branch — channel, checksum,
  attestation, archive guard, link, setup wiring.
- `SECURITY.md` documents the threat model.