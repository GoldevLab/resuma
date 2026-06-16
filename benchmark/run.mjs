#!/usr/bin/env node
/**
 * Counter page benchmark — Resuma vs Qwik, Leptos, Astro, Next.js, SvelteKit,
 * SolidStart, React (Vite), and Go templ + HTMX.
 *
 * Usage: node benchmark/run.mjs [--skip-build]
 */

import {
  readFileSync,
  existsSync,
  writeFileSync,
  readdirSync,
  statSync,
  mkdirSync,
  copyFileSync,
} from "node:fs";
import { gzipSync, brotliCompressSync } from "node:zlib";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const skipBuild = process.argv.includes("--skip-build");

function fmt(bytes) {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  return `${(bytes / 1024).toFixed(2)} KiB`;
}

function measureBytes(rawLen, gzLen, brLen) {
  return { raw: rawLen, gzip: gzLen, brotli: brLen };
}

function measureFile(path) {
  if (!existsSync(path)) return null;
  const rawBuf = readFileSync(path);
  return {
    path,
    name: path.split(/[/\\]/).pop(),
    raw: rawBuf.length,
    gzip: gzipSync(rawBuf).length,
    brotli: brotliCompressSync(rawBuf).length,
  };
}

function totals(rows) {
  const list = rows.filter(Boolean);
  const t = {
    raw: list.reduce((n, r) => n + r.raw, 0),
    gzip: list.reduce((n, r) => n + r.gzip, 0),
    brotli: list.reduce((n, r) => n + r.brotli, 0),
  };
  return t;
}

function row(label, r) {
  if (!r) return null;
  return {
    label,
    raw: r.raw,
    gzip: r.gzip,
    brotli: r.brotli,
    rawFmt: fmt(r.raw),
    gzipFmt: fmt(r.gzip),
    brotliFmt: fmt(r.brotli),
  };
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch {
    return null;
  }
}

function run(cmd, cwd, env = {}) {
  execSync(cmd, { cwd, stdio: "inherit", env: { ...process.env, ...env } });
}

function runCapture(cmd, cwd) {
  return execSync(cmd, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}

function pkgDir(name) {
  return join(__dirname, name);
}

function npmInstall(dir) {
  if (!existsSync(join(dir, "node_modules"))) {
    console.log(`[benchmark] npm install in ${relative(root, dir)}…`);
    run("npm install", dir);
  }
}

function parseHumanSize(text) {
  const m = String(text).trim().match(/^([\d.]+)\s*(B|kB|KiB|MB|MiB)$/i);
  if (!m) return null;
  const n = parseFloat(m[1]);
  const unit = m[2].toLowerCase();
  if (unit === "b") return Math.round(n);
  if (unit.startsWith("k")) return Math.round(n * 1024);
  return Math.round(n * 1024 * 1024);
}

function parseNextFirstLoadKb(output) {
  for (const line of output.split("\n")) {
    if (!/(^|\s)\/\s/.test(line) || line.includes("_not-found")) continue;
    const m = line.match(/([\d.]+\s*kB|[\d.]+\s*MB|[\d.]+\s*B)\s*$/i);
    if (m) return parseHumanSize(m[1]);
  }
  return null;
}

function scriptRefsFromHtml(html, htmlDir) {
  const refs = new Set();
  const patterns = [
    /<script[^>]+src=["']([^"']+)["']/gi,
    /<link[^>]+rel=["']modulepreload["'][^>]+href=["']([^"']+)["']/gi,
    /<link[^>]+href=["']([^"']+\.js[^"']*)["'][^>]+rel=["']modulepreload["']/gi,
    /component-url=["']([^"']+\.js)["']/gi,
    /renderer-url=["']([^"']+\.js)["']/gi,
  ];
  for (const re of patterns) {
    for (const m of html.matchAll(re)) {
      let href = m[1].split("?")[0].split("#")[0];
      if (href.startsWith("http")) continue;
      if (href.startsWith("/")) href = href.slice(1);
      refs.add(resolve(htmlDir, href));
    }
  }
  return [...refs];
}

function measureHtmlPage(htmlPath, assetRoot) {
  if (!existsSync(htmlPath)) return null;
  const html = readFileSync(htmlPath, "utf8");
  const refs = scriptRefsFromHtml(html, assetRoot);
  const files = refs.map((p) => measureFile(p)).filter(Boolean);
  return { files, total: totals(files) };
}

function findIndexHtml(dir) {
  const direct = join(dir, "index.html");
  if (existsSync(direct)) return direct;
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) {
      const nested = findIndexHtml(p);
      if (nested) return nested;
    }
  }
  return null;
}

function result(framework, version, initial, firstInteraction, staticPage, details, note) {
  return {
    framework,
    version,
    initial,
    firstInteraction: firstInteraction ?? initial,
    staticPage: staticPage ?? null,
    details: details ?? {},
    note,
  };
}

function ensureResumaRuntime() {
  if (existsSync(join(root, "runtime", "dist", "loader.js"))) return;
  console.log("[benchmark] building Resuma runtime…");
  run("npm run build", join(root, "runtime"));
}

function readWorkspaceVersion() {
  const cargo = join(root, "Cargo.toml");
  if (!existsSync(cargo)) return "0.4.8";
  const text = readFileSync(cargo, "utf8");
  const m = text.match(/^version\s*=\s*"([^"]+)"/m);
  return m?.[1] ?? "0.4.8";
}

function ensureResumaHandlerSample() {
  const out = join(root, "benchmark", ".resuma-counter-handler.js");
  if (skipBuild && existsSync(out)) return out;
  console.log("[benchmark] writing representative Counter handler chunk…");
  run(
    "cargo test -p resuma --quiet write_benchmark_counter_handler -- --exact --nocapture",
    root,
    { RESUMA_WRITE_BENCHMARK_HANDLER: "1" },
  );
  if (!existsSync(out)) {
    throw new Error("benchmark handler sample missing — cargo test write_benchmark_counter_handler failed");
  }
  return out;
}

function measureResuma() {
  ensureResumaHandlerSample();
  const dist = join(root, "runtime", "dist");
  const loader = measureFile(join(dist, "loader.js"));
  const core = measureFile(join(dist, "core.js"));
  const handler = measureFile(join(root, "benchmark", ".resuma-counter-handler.js"));
  const first = totals([loader, core, handler]);
  first.note = "loader.js + core.js + Counter handler chunk (first click)";
  return result(
    "Resuma",
    readWorkspaceVersion(),
    loader,
    first,
    { raw: 0, gzip: 0, brotli: 0 },
    { loader, core, handler },
    "Rust SSR + resumability",
  );
}

function measureQwik() {
  const dir = pkgDir("qwik-counter");
  if (!skipBuild && !existsSync(join(dir, "dist", "q-manifest.json"))) {
    npmInstall(dir);
    run("npm run build", dir);
  }
  const manifest = readJson(join(dir, "dist", "q-manifest.json"));
  const buildDir = join(dir, "dist", "build");
  const file = (n) => measureFile(join(buildDir, n));
  const preloader = file(manifest.preloader);
  const core = file(manifest.core);
  let routeEntry = null;
  let counterChunk = null;
  for (const [name, meta] of Object.entries(manifest.bundles ?? {})) {
    const origins = (meta.origins ?? []).join(" ");
    if (origins.includes("src\\routes\\index.tsx") && !origins.includes("onClick")) {
      routeEntry = file(name);
    }
    if (origins.includes("onClick") && origins.includes("index.tsx")) {
      counterChunk = file(name);
    }
  }
  const first = totals([preloader, core, routeEntry, counterChunk]);
  return result(
    "Qwik",
    readJson(join(dir, "package.json"))?.devDependencies?.["@builder.io/qwik"]?.replace("^", "") ??
      "1.20.0",
    preloader,
    first,
    null,
    { preloader, core, routeEntry, counterChunk },
    "Resumability (JS)",
  );
}

function measureLeptos() {
  const dir = pkgDir("leptos-counter");
  const wasm = join(dir, "pkg", "leptos_counter_bench_bg.wasm");
  const glue = join(dir, "pkg", "leptos_counter_bench.js");
  if (!skipBuild && (!existsSync(wasm) || !existsSync(glue))) {
    run("rustup target add wasm32-unknown-unknown", dir);
    run("cargo build --release --target wasm32-unknown-unknown", dir);
    run("wasm-pack build --target web --release --out-dir pkg --out-name leptos_counter_bench", dir);
  }
  const wasmM = measureFile(wasm);
  const glueM = measureFile(glue);
  const initial = totals([wasmM, glueM]);
  return result("Leptos", "0.7.8", initial, initial, null, { wasm: wasmM, glue: glueM }, "Rust SSR + WASM hydrate");
}

function measureReact() {
  const dir = pkgDir("react-counter");
  npmInstall(dir);
  if (!skipBuild) run("npm run build", dir);
  const htmlPath = join(dir, "dist", "index.html");
  const measured = measureHtmlPage(htmlPath, join(dir, "dist"));
  if (!measured?.total) throw new Error("React build missing dist/index.html scripts");
  return result(
    "React (Vite)",
    readJson(join(dir, "package.json"))?.dependencies?.react?.replace("^", "") ?? "19",
    measured.total,
    measured.total,
    null,
    Object.fromEntries(measured.files.map((f, i) => [`chunk${i}`, f])),
    "Client-rendered SPA baseline",
  );
}

function measureNext() {
  const dir = pkgDir("next-counter");
  npmInstall(dir);
  const statsPath = join(dir, ".next", "diagnostics", "route-bundle-stats.json");
  if (!skipBuild || !existsSync(statsPath)) {
    try {
      runCapture("npm run build", dir);
    } catch (e) {
      if (!existsSync(statsPath)) throw e;
    }
  }
  const stats = readJson(statsPath);
  const route = stats?.find((r) => r.route === "/");
  if (!route?.firstLoadChunkPaths?.length) {
    throw new Error("Next.js route-bundle-stats.json missing / route");
  }
  const files = route.firstLoadChunkPaths.map((p) =>
    measureFile(join(dir, p.replace(/\\/g, "/"))),
  );
  const total = totals(files);
  total.note = "firstLoadChunkPaths from next build diagnostics";
  return result(
    "Next.js",
    readJson(join(dir, "package.json"))?.dependencies?.next?.replace("^", "") ?? "16",
    total,
    total,
    null,
    Object.fromEntries(files.map((f, i) => [`chunk${i}`, f])),
    "React SSR + App Router hydration",
  );
}

function measureAstro() {
  const dir = pkgDir("astro-counter");
  npmInstall(dir);
  if (!skipBuild) run("npm run build", dir);
  const htmlPath = join(dir, "dist", "index.html");
  let measured = measureHtmlPage(htmlPath, join(dir, "dist"));
  if (!measured?.total?.gzip) {
    const astroDir = join(dir, "dist", "_astro");
    if (existsSync(astroDir)) {
      const files = readdirSync(astroDir)
        .filter((f) => f.endsWith(".js"))
        .map((f) => measureFile(join(astroDir, f)));
      measured = { files, total: totals(files) };
    }
  }
  if (!measured?.total?.gzip) throw new Error("Astro build missing measurable JS");
  return result(
    "Astro",
    readJson(join(dir, "package.json"))?.dependencies?.astro?.replace("^", "") ?? "5",
    measured.total,
    measured.total,
    null,
    Object.fromEntries(measured.files.map((f, i) => [`script${i}`, f])),
    "SSR + React island (client:load)",
  );
}

function measureSvelteKit() {
  const dir = pkgDir("sveltekit-counter");
  npmInstall(dir);
  if (!skipBuild) run("npm run build", dir);
  const htmlPath = join(dir, "dist", "index.html");
  let measured = measureHtmlPage(htmlPath, join(dir, "dist"));
  if (!measured?.total?.gzip) {
    const clientDir = join(dir, ".svelte-kit", "output", "client");
    const htmlAlt = findIndexHtml(clientDir);
    if (htmlAlt) measured = measureHtmlPage(htmlAlt, dirname(htmlAlt));
  }
  if (!measured?.total?.gzip) throw new Error("SvelteKit build missing measurable JS");
  return result(
    "SvelteKit",
    readJson(join(dir, "package.json"))?.devDependencies?.["@sveltejs/kit"]?.replace("^", "") ?? "2",
    measured.total,
    measured.total,
    null,
    Object.fromEntries(measured.files.map((f, i) => [`script${i}`, f])),
    "SSR + client hydration",
  );
}

function measureSolidStart() {
  const dir = pkgDir("solidstart-counter");
  npmInstall(dir);
  if (!skipBuild) run("npm run build", dir);
  const assetsDir = join(dir, ".output", "public", "_build", "assets");
  let measured = null;
  if (existsSync(assetsDir)) {
    const files = readdirSync(assetsDir)
      .filter((f) => f.endsWith(".js") && !f.endsWith(".js.gz"))
      .map((f) => measureFile(join(assetsDir, f)));
    measured = { files, total: totals(files) };
  }
  if (!measured?.total?.gzip) {
    const publicDir = join(dir, ".output", "public");
    const htmlPath = findIndexHtml(publicDir);
    if (htmlPath) measured = measureHtmlPage(htmlPath, dirname(htmlPath));
  }
  if (!measured?.total?.gzip) throw new Error("SolidStart build missing measurable JS");
  return result(
    "SolidStart",
    readJson(join(dir, "package.json"))?.dependencies?.["@solidjs/start"]?.replace("^", "") ?? "1.2",
    measured.total,
    measured.total,
    null,
    Object.fromEntries(measured.files.map((f, i) => [`script${i}`, f])),
    "Solid SSR + hydration",
  );
}

function measureTemplHtmx() {
  const dir = pkgDir("templ-htmx-counter");
  npmInstall(dir);
  const staticDir = join(dir, "static");
  mkdirSync(staticDir, { recursive: true });
  const srcHtmx = join(dir, "node_modules", "htmx.org", "dist", "htmx.min.js");
  const dstHtmx = join(staticDir, "htmx.min.js");
  if (existsSync(srcHtmx)) copyFileSync(srcHtmx, dstHtmx);
  const htmx = measureFile(dstHtmx) ?? measureFile(srcHtmx);
  if (!htmx) throw new Error("htmx.min.js not found — run npm install in templ-htmx-counter");
  const initial = { ...totals([htmx]), note: "htmx.min.js only — server-side templ, no client app bundle" };
  return result(
    "templ + HTMX",
    "htmx 2",
    initial,
    initial,
    null,
    { htmx },
    "Go SSR + HTMX (server round-trip)",
  );
}

const MEASURERS = [
  measureResuma,
  measureTemplHtmx,
  measureQwik,
  measureAstro,
  measureReact,
  measureNext,
  measureSvelteKit,
  measureSolidStart,
  measureLeptos,
];

function main() {
  ensureResumaRuntime();
  const frameworks = [];
  const errors = [];

  for (const measure of MEASURERS) {
    const label = measure.name;
    try {
      console.log(`\n[benchmark] measuring ${label}…`);
      frameworks.push(measure());
    } catch (err) {
      console.error(`[benchmark] ${label} failed:`, err.message);
      errors.push({ framework: label, error: err.message });
    }
  }

  frameworks.sort((a, b) => a.initial.gzip - b.initial.gzip);

  for (const f of frameworks) {
    f.initialRow = row("Initial load", f.initial);
    f.firstRow = row("First interaction", f.firstInteraction);
    if (f.staticPage) f.staticRow = row("Static page", f.staticPage);
  }

  const results = {
    generatedAt: new Date().toISOString(),
    methodology:
      "SSR counter page (heading + increment button). Gzip/brotli from minified artifacts or framework build output.",
    frameworks,
    errors,
  };

  writeFileSync(join(__dirname, "results.json"), JSON.stringify(results, null, 2));
  printTable(results);
  if (errors.length) {
    console.log("\nErrors:", errors);
    process.exitCode = 1;
  }
}

function printTable(results) {
  console.log("\nCounter page benchmark (gzip)\n");
  console.log(["Framework", "Initial", "First click", "Static"].map((h) => h.padEnd(20)).join(""));
  for (const f of results.frameworks) {
    console.log(
      `${f.framework.padEnd(20)}${(f.initialRow?.gzipFmt ?? "—").padEnd(20)}${(f.firstRow?.gzipFmt ?? "—").padEnd(20)}${(f.staticRow?.gzipFmt ?? "—").padEnd(20)}`,
    );
  }
  console.log(`\nWrote ${join(__dirname, "results.json")}`);
}

main();
