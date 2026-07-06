# Security Policy

## Supported Versions

claudebar uses [CalVer](https://calver.org/) (`YYYY.M.D`) with no maintained
LTS branches. Only the latest published release receives security fixes.

| Version                | Supported          |
| ---------------------- | ------------------ |
| Latest published release | :white_check_mark: |
| Older releases         | :x:                |

## Reporting a Vulnerability

Do not open a public GitHub issue for security vulnerabilities. Instead, use
GitHub's private reporting flow:
[Report a vulnerability](https://github.com/micschr0/claudebar/security/advisories/new).

Include:

- A description of the vulnerability and its impact.
- Reproduction steps. For render-pipeline issues, a minimal JSON input in the
  style of `fixtures/*.json` is the fastest repro path.
- The affected version (`claudebar --version`).

claudebar is solo-maintained. Expect a best-effort acknowledgment within a
few days — there is no formal SLA.

Release artifacts carry GitHub artifact attestations; see
[Verify a release](README.md#verify-a-release) to confirm a download's
provenance. Provenance verification is available via `install.sh` or the manual
`gh attestation verify` command — it does **not** apply to Homebrew installs,
which are limited to the formula's SHA256 checksum (`HOMEBREW_VERIFY_ATTESTATIONS`
only covers `homebrew/core` bottles, not third-party taps).
