import { spawn } from "node:child_process";
import process from "node:process";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const host = "127.0.0.1";
/** Prefer ephemeral port unless RESUMA_E2E_TODO_PORT is set (avoids stale dev servers). */
const bindAddr =
  process.env.RESUMA_E2E_TODO_PORT != null
    ? `${host}:${process.env.RESUMA_E2E_TODO_PORT}`
    : `${host}:0`;
const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const stoppingServers = new WeakSet();

const LISTEN_RE = /listening on (https?:\/\/[^\s]+)/i;

function log(message) {
  console.log(`[e2e:todo] ${message}`);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function parseListeningUrl(text) {
  const match = text.match(LISTEN_RE);
  return match ? match[1].replace(/\/$/, "") : null;
}

async function waitForServer(baseUrl) {
  const deadline = Date.now() + 45_000;
  let lastError = "";

  while (Date.now() < deadline) {
    try {
      const res = await fetch(baseUrl);
      if (res.ok) return;
      lastError = `HTTP ${res.status}`;
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
    }
    await delay(250);
  }

  throw new Error(`todo server did not become ready at ${baseUrl}: ${lastError}`);
}

function startServer() {
  return new Promise((resolve, reject) => {
    const stdio = process.platform === "win32" ? "inherit" : ["ignore", "pipe", "pipe"];
    const child = spawn("cargo", ["run", "-p", "example-todo"], {
      cwd: repoRoot,
      env: {
        ...process.env,
        RESUMA_ADDR: bindAddr,
        // Updated once we know the bound port; generic host is fine for localhost E2E.
        SITE_URL: `http://${host}`,
        RESUMA_ENV: "development",
      },
      stdio,
      windowsHide: true,
    });

    let baseUrl = null;
    const deadline = setTimeout(() => {
      if (!baseUrl) {
        reject(new Error("timed out waiting for todo server to print listening URL"));
        stopServer(child);
      }
    }, 60_000);

    const onListen = (chunk) => {
      process.stdout.write(`[todo] ${chunk}`);
      const url = parseListeningUrl(chunk.toString());
      if (url && !baseUrl) {
        baseUrl = url;
        clearTimeout(deadline);
        log(`server at ${baseUrl}`);
        resolve({ child, baseUrl });
      }
    };

    if (child.stdout) child.stdout.on("data", onListen);
    if (child.stderr) child.stderr.on("data", onListen);

    child.once("exit", (code, signal) => {
      if (stoppingServers.has(child) || baseUrl) return;
      clearTimeout(deadline);
      if (code !== null && code !== 0) {
        reject(new Error(`todo server exited with code ${code}`));
      } else if (signal) {
        reject(new Error(`todo server exited via ${signal}`));
      }
    });
  });
}

function stopServer(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  stoppingServers.add(child);
  if (process.platform === "win32") {
    spawn("taskkill", ["/pid", String(child.pid), "/T", "/F"], {
      stdio: "ignore",
      windowsHide: true,
    });
  } else {
    child.kill("SIGTERM");
  }
}

async function waitForTodoTitle(page, title, ms = 15_000) {
  const deadline = Date.now() + ms;
  while (Date.now() < deadline) {
    const found = await page.evaluate((expected) => {
      if (document.querySelector(".todo-title")?.textContent?.includes(expected)) return true;
      const r = window.__resuma;
      if (!r) return false;
      for (const cell of r.signals.values()) {
        const v = cell.value;
        if (Array.isArray(v) && v.some((t) => t?.title === expected)) return true;
      }
      return false;
    }, title);
    if (found) return;
    await delay(200);
  }
  throw new Error(`todo "${title}" did not appear within ${ms}ms`);
}

async function runBrowserChecks(baseUrl) {
  const launchOptions = {};
  if (process.env.RESUMA_E2E_BROWSER_PATH) {
    launchOptions.executablePath = process.env.RESUMA_E2E_BROWSER_PATH;
  }
  if (process.env.RESUMA_E2E_BROWSER_CHANNEL) {
    launchOptions.channel = process.env.RESUMA_E2E_BROWSER_CHANNEL;
  }

  const browser = await chromium.launch(launchOptions);
  const page = await browser.newPage();
  const consoleErrors = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") consoleErrors.push(msg.text());
  });
  page.on("pageerror", (err) => consoleErrors.push(err.message));

  try {
    log("opening todo page");
    await page.goto(baseUrl, { waitUntil: "networkidle" });
    await page.getByRole("heading", { name: "Todo" }).waitFor({ timeout: 10_000 });

    log("waiting for workspace");
    const input = page.getByLabel("New task", { exact: true });
    await input.waitFor({ state: "visible", timeout: 10_000 });

    log("waiting for initial todo load");
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
      const loaded = await page.evaluate(() => {
        const r = window.__resuma;
        if (!r) return false;
        for (const cell of r.signals.values()) {
          if (Array.isArray(cell.value) && cell.value.length > 0) return true;
        }
        return document.querySelectorAll(".todo-item").length > 0;
      });
      if (loaded) break;
      await delay(200);
    }

    log("adding a task");
    await input.fill("Buy milk");
    await input.dispatchEvent("input");
    // Island form uses preventDefault — no full navigation; don't wait for one.
    await page.getByRole("button", { name: "Add" }).click({ noWaitAfter: true });
    await waitForTodoTitle(page, "Buy milk");

    log("toggling theme");
    const themeBtn = page.locator(".theme-toggle");
    await themeBtn.click({ noWaitAfter: true });
    await delay(200);

    log("checking server action round-trip via signals");
    const count = await page.evaluate(() => {
      const r = window.__resuma;
      if (!r) return -1;
      for (const cell of r.signals.values()) {
        const v = cell.value;
        if (Array.isArray(v) && v.length > 0 && v[0]?.title != null) return v.length;
      }
      return -1;
    });
    assert(count >= 1, `expected todos signal length >= 1, got ${count}`);

    assert(consoleErrors.length === 0, `browser console errors:\n${consoleErrors.join("\n")}`);
  } finally {
    await browser.close();
  }
}

const { child: server, baseUrl } = await startServer();

try {
  await waitForServer(baseUrl);
  await runBrowserChecks(baseUrl);
  log("all todo checks passed");
} finally {
  stopServer(server);
}
