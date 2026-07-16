'use strict';

// Maps Node's `process.platform`/`process.arch` to the asset names
// produced by .github/workflows/release.yml. Keep this in sync with that
// workflow's matrix — it's the single source of truth for what gets built.
const TARGETS = {
  'darwin-x64': 'blink-macos-x64',
  'darwin-arm64': 'blink-macos-arm64',
  'linux-x64': 'blink-linux-x64',
  'linux-arm64': 'blink-linux-arm64',
  'win32-x64': 'blink-windows-x64.exe',
};

function currentTarget() {
  const key = `${process.platform}-${process.arch}`;
  const target = TARGETS[key];
  if (!target) {
    const supported = Object.keys(TARGETS).join(', ');
    throw new Error(
      `Blink has no prebuilt binary for ${process.platform}/${process.arch}. ` +
        `Supported platforms: ${supported}. ` +
        'Build from source instead: https://github.com/martin-k-m/blink#installation'
    );
  }
  return target;
}

function binaryFileName() {
  return process.platform === 'win32' ? 'blink.exe' : 'blink';
}

module.exports = { TARGETS, currentTarget, binaryFileName };
