#!/usr/bin/env node
/**
 * Report minified + gzip + brotli sizes for Resuma runtime bundles.
 * Usage: node scripts/measure.mjs [file ...]
 */

import { readFileSync, existsSync } from "node:fs";
import { gzipSync, brotliCompressSync } from "node:zlib";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const dist = join(__dirname, "..", "dist");

const LOADER_GZIP_BUDGET = 1024;
const CORE_GZIP_BUDGET = 5700;

const defaults = ["loader.js", "core.js", "runtime.js"].map((f) => join(dist, f));

function fmt(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  return `${(bytes / 1024).toFixed(2)} KiB`;
}

function measure(path) {
  if (!existsSync(path)) {
    console.log(`${path}: (missing — run npm run build)`);
    return null;
  }
  const raw = readFileSync(path);
  const gz = gzipSync(raw);
  const br = brotliCompressSync(raw);
  const name = path.split(/[/\\]/).pop();
  console.log(`${name}:`);
  console.log(`  raw:    ${fmt(raw.length)} (${raw.length} bytes)`);
  console.log(`  gzip:   ${fmt(gz.length)} (${gz.length} bytes)`);
  console.log(`  brotli: ${fmt(br.length)} (${br.length} bytes)`);
  return { name, raw: raw.length, gzip: gz.length, brotli: br.length };
}

console.log("Resuma runtime bundle sizes\n");

const files = process.argv.length > 2 ? process.argv.slice(2) : defaults;
const rows = files.map(measure).filter(Boolean);

if (rows.length >= 2) {
  const loader = rows.find((r) => r.name === "loader.js");
  const core = rows.find((r) => r.name === "core.js");
  if (loader && core) {
    console.log("\nSplit total (loader + core, first interaction):");
    console.log(`  raw:  ${fmt(loader.raw + core.raw)}`);
    console.log(`  gzip: ${fmt(loader.gzip + core.gzip)}`);
  }
  let failed = false;
  if (loader && loader.gzip > LOADER_GZIP_BUDGET) {
    console.error(`\nERROR: loader.js gzip ${loader.gzip} B exceeds budget ${LOADER_GZIP_BUDGET} B`);
    failed = true;
  }
  if (core && core.gzip > CORE_GZIP_BUDGET) {
    console.error(`\nERROR: core.js gzip ${core.gzip} B exceeds budget ${CORE_GZIP_BUDGET} B`);
    failed = true;
  }
  if (failed) process.exit(1);
}
