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

## Verifying a release

Every `claudebar-*.tar.gz` release asset carries a [GitHub artifact
attestation](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations).
Verify it was signed by this repository's release workflow specifically (not
just any workflow in the repo):

```bash
gh attestation verify claudebar-<target>.tar.gz \
  --repo micschr0/claudebar \
  --signer-workflow micschr0/claudebar/.github/workflows/release.yml
```

`<target>` is your platform triple, e.g. `x86_64-unknown-linux-musl` or
`aarch64-apple-darwin`. `install.sh` runs this check automatically when `gh` is
installed and authenticated; when it isn't, the install continues — the SHA256
checksum remains the mandatory integrity gate.

Provenance verification is available via `install.sh` or the manual `gh
attestation verify` command — it does **not** apply to Homebrew installs, which
are limited to the formula's SHA256 checksum (`HOMEBREW_VERIFY_ATTESTATIONS`
only covers `homebrew/core` bottles, not third-party taps).
