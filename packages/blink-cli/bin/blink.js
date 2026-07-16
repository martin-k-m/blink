#!/usr/bin/env node
'use strict';

// Thin shim: this is what `npm install -g` puts on PATH as `blink`. It
// never contains any of Blink's actual logic — it just execs the native
// binary that scripts/install.js downloaded, forwarding args/stdio and the
// exit code untouched.

const path = require('path');
const { spawnSync } = require('child_process');
const { binaryFileName } = require('../scripts/platform');

const binaryPath = path.join(__dirname, 'native', binaryFileName());
const result = spawnSync(binaryPath, process.argv.slice(2), { stdio: 'inherit' });

if (result.error) {
  if (result.error.code === 'ENOENT') {
    console.error(
      'blink-cli: native binary not found. This usually means `npm install` ' +
        'did not finish successfully. Try reinstalling: npm install -g blink-cli'
    );
  } else {
    console.error(`blink-cli: failed to launch: ${result.error.message}`);
  }
  process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
