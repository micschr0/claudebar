#!/usr/bin/env node
'use strict';
const path = require('path');
const { spawnSync } = require('child_process');

// npm selects the right @micschr0/claudebar-<platform> optionalDependency by
// os/cpu; we just spawn its binary.
const PLATFORMS = {
  'darwin-x64': '@micschr0/claudebar-darwin-x64',
  'darwin-arm64': '@micschr0/claudebar-darwin-arm64',
  'linux-x64': '@micschr0/claudebar-linux-x64-musl',
  'linux-arm64': '@micschr0/claudebar-linux-arm64-musl',
};

const key = process.platform + '-' + process.arch;
const pkg = PLATFORMS[key];
if (!pkg) {
  console.error(`claudebar: unsupported platform ${key} (supported: macOS/Linux x64+arm64)`);
  process.exit(1);
}

let bin;
try {
  bin = path.join(path.dirname(require.resolve(pkg + '/package.json')), 'claudebar');
} catch {
  console.error('claudebar: platform binary missing — reinstall with: npm i -g @micschr0/claudebar --force');
  process.exit(1);
}

const r = spawnSync(bin, process.argv.slice(2), { stdio: 'inherit' });
if (r.error) { console.error(`claudebar: ${r.error.message}`); process.exit(1); }
if (r.signal) process.kill(process.pid, r.signal);
else process.exit(r.status === null ? 0 : r.status);
