# claudebar

**A powerline statusline for Claude Code: segments, themes, and a live TUI configurator in a single native binary.**

This is the npm distribution package for [claudebar](https://github.com/micschr0/claudebar).
It installs the prebuilt native binary for your platform and exposes it on your `PATH` as `claudebar`.

## Install

```bash
npm install -g @micschr0/claudebar
```

pnpm (or Yarn/Bun) works too: pnpm reads the same npm-registry
`optionalDependencies` and selects the right platform package via `os`/`cpu`:

```bash
pnpm add -g @micschr0/claudebar
```

Then wire it into Claude Code:

```bash
claudebar setup
```

The binary is a native executable shipped as an optional dependency
(`@micschr0/claudebar-<platform>`); no Node runtime or postinstall download is required.

## Support

- macOS (Intel x64, Apple silicon arm64)
- Linux (x64, arm64 — musl builds)

Other install methods (Homebrew, mise, `install.sh`) and full documentation:
<https://github.com/micschr0/claudebar>
