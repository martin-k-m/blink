#!/usr/bin/env node
'use strict';

// Runs as the package's postinstall step. Downloads the release binary
// that matches this package's version and the current platform/arch,
// verifies it against the published SHA-256 checksum, and places it at
// bin/native/<blink|blink.exe> where bin/blink.js expects to find it.
//
// Set BLINK_LOCAL_BIN to the path of an already-built `blink` binary to
// skip the download entirely — used by this repo's own CI/tests, and
// useful for anyone testing the npm package against a local build before
// a GitHub release exists.

const fs = require('fs');
const path = require('path');
const https = require('https');
const crypto = require('crypto');

const { currentTarget, binaryFileName } = require('./platform');

const pkg = require('../package.json');

const BIN_DIR = path.join(__dirname, '..', 'bin', 'native');
const DEST = path.join(BIN_DIR, binaryFileName());
const REPO = 'martin-k-m/blink';

function releaseAssetUrl(assetName) {
  return `https://github.com/${REPO}/releases/download/v${pkg.version}/${assetName}`;
}

function download(url, redirectsLeft = 5) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { 'User-Agent': 'blink-cli-npm-installer' } }, (res) => {
        const { statusCode, headers } = res;
        if (statusCode >= 300 && statusCode < 400 && headers.location && redirectsLeft > 0) {
          res.resume();
          resolve(download(headers.location, redirectsLeft - 1));
          return;
        }
        if (statusCode !== 200) {
          res.resume();
          reject(new Error(`HTTP ${statusCode} for ${url}`));
          return;
        }
        const chunks = [];
        res.on('data', (chunk) => chunks.push(chunk));
        res.on('end', () => resolve(Buffer.concat(chunks)));
        res.on('error', reject);
      })
      .on('error', reject);
  });
}

async function verifyChecksum(buffer, checksumUrl) {
  let checksumFile;
  try {
    checksumFile = (await download(checksumUrl)).toString('utf8');
  } catch (err) {
    console.warn(`blink-cli: could not fetch checksum (${err.message}); skipping verification.`);
    return;
  }
  const match = checksumFile.match(/^([a-f0-9]{64})/m);
  if (!match) {
    console.warn('blink-cli: no checksum found in checksum file; skipping verification.');
    return;
  }
  const expected = match[1];
  const actual = crypto.createHash('sha256').update(buffer).digest('hex');
  if (actual !== expected) {
    throw new Error(`checksum mismatch: expected ${expected}, got ${actual}`);
  }
}

function installLocalBinary(sourcePath) {
  fs.mkdirSync(BIN_DIR, { recursive: true });
  fs.copyFileSync(sourcePath, DEST);
  fs.chmodSync(DEST, 0o755);
  console.log(`blink-cli: installed local binary from ${sourcePath} to ${DEST}`);
}

async function installFromRelease() {
  const target = currentTarget();
  const assetUrl = releaseAssetUrl(target);
  const checksumUrl = releaseAssetUrl(`${target}.sha256`);

  console.log(`blink-cli: downloading ${target} (v${pkg.version})...`);
  const buffer = await download(assetUrl);
  await verifyChecksum(buffer, checksumUrl);

  fs.mkdirSync(BIN_DIR, { recursive: true });
  fs.writeFileSync(DEST, buffer);
  fs.chmodSync(DEST, 0o755);
  console.log(`blink-cli: installed to ${DEST}`);
}

async function main() {
  if (process.env.BLINK_LOCAL_BIN) {
    installLocalBinary(process.env.BLINK_LOCAL_BIN);
    return;
  }
  await installFromRelease();
}

main().catch((err) => {
  console.error(`blink-cli: install failed: ${err.message}`);
  console.error('You can build Blink from source instead: https://github.com/martin-k-m/blink#installation');
  process.exit(1);
});
