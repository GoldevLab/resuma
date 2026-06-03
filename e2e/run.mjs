import { spawn } from "node:child_process";
import process from "node:process";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const host = "127.0.0.1";
const port = Number(process.env.RESUMA_E2E_PORT ?? 3217);
const baseUrl = `http://${host}:${port}`;
const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const stoppingServers = new WeakSet();

function log(message) {
  console.log(`[e2e] ${message}`);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function isExpectedConsoleError(text) {
  return (
    text.includes("the server responded with a status of 422") ||
    (text.includes("Applying inline style violates") && text.includes("style-src"))
  );
}

async function waitForServer() {
  const deadline = Date.now() + 30_000;
  let lastError = "";

  while (Date.now() < deadline) {
    try {
      const res = await fetch(`${baseUrl}/health`);
      if (res.ok) return;
      lastError = `HTTP ${res.status}`;
    } catch (err) {
      lastError = err instanceof Error ? err.message : String(err);
    }
    await delay(250);
  }

  throw new Error(`server did not become ready at ${baseUrl}: ${lastError}`);
}

function startServer() {
  const stdio = process.platform === "win32" ? "inherit" : ["ignore", "pipe", "pipe"];
  const child = spawn("cargo", ["run", "-p", "example-e2e"], {
    cwd: repoRoot,
    env: {
      ...process.env,
      RESUMA_ADDR: `${host}:${port}`,
      RESUMA_ENV: "development",
    },
    stdio,
    windowsHide: true,
  });

  if (child.stdout) {
    child.stdout.on("data", (chunk) => process.stdout.write(`[server] ${chunk}`));
  }
  if (child.stderr) {
    child.stderr.on("data", (chunk) => process.stderr.write(`[server] ${chunk}`));
  }

  child.once("exit", (code, signal) => {
    if (stoppingServers.has(child)) return;
    if (code !== null && code !== 0) {
      console.error(`[e2e] server exited with code ${code}`);
    } else if (signal) {
      console.error(`[e2e] server exited via ${signal}`);
    }
  });

  return child;
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

async function expectText(locator, expected, label) {
  await locator.waitFor({ state: "visible", timeout: 5_000 });
  const deadline = Date.now() + 5_000;
  let actual = "";
  while (Date.now() < deadline) {
    actual = (await locator.innerText({ timeout: 1_000 })).trim();
    if (actual === expected) return;
    await delay(100);
  }
  assert(actual === expected, `${label}: expected "${expected}", got "${actual}"`);
}

async function clickUnique(locator, label) {
  const count = await locator.count();
  assert(count === 1, `${label}: expected 1 element, got ${count}`);
  await locator.click({ timeout: 5_000 });
}

async function fillUnique(locator, value, label) {
  const count = await locator.count();
  assert(count === 1, `${label}: expected 1 element, got ${count}`);
  await locator.fill(value, { timeout: 5_000 });
}

async function runBrowserChecks() {
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
    if (msg.type() === "error" && !isExpectedConsoleError(msg.text())) {
      consoleErrors.push(msg.text());
    }
  });
  page.on("pageerror", (err) => consoleErrors.push(err.message));

  try {
    log("opening home page");
    await page.goto(baseUrl, { waitUntil: "networkidle" });
    await expectText(page.getByTestId("count"), "Count: 0", "initial counter");

    log("testing resumable click handler");
    await clickUnique(page.getByTestId("increment"), "increment button");
    await expectText(page.getByTestId("count"), "Count: 1", "counter after click");

    log("testing server action");
    await clickUnique(page.getByTestId("save-count"), "save count button");
    await expectText(page.getByTestId("save-status"), "Saved 1", "server action status");

    log("testing enhanced form validation");
    await clickUnique(page.getByRole("button", { name: "Send" }), "send button");
    await page.getByText("Name is required", { exact: true }).waitFor({
      state: "visible",
      timeout: 5_000,
    });

    log("testing successful submit redirect");
    await fillUnique(page.getByLabel("Name", { exact: true }), "Ada", "name input");
    await fillUnique(page.getByLabel("Email", { exact: true }), "ada@example.com", "email input");
    await clickUnique(page.getByRole("button", { name: "Send" }), "send button");
    await page.waitForURL(`${baseUrl}/thanks`, { timeout: 5_000 });
    await expectText(page.getByTestId("thanks-copy"), "Form submitted successfully.", "thanks page");

    log("testing SPA navigation without document reload");
    await page.evaluate(() => {
      window.__resumaE2EMarker = "kept";
    });
    await clickUnique(page.getByRole("link", { name: "About" }), "about nav link");
    await page.waitForURL(`${baseUrl}/about`, { timeout: 5_000 });
    await expectText(page.getByTestId("about-copy"), "SPA navigation rendered this page.", "about page");
    const marker = await page.evaluate(() => window.__resumaE2EMarker ?? null);
    assert(marker === "kept", "SPA navigation reloaded the document instead of swapping in place");

    log("testing client effects replay after SPA navigation");
    await expectText(page.getByTestId("about-doubled"), "Doubled: 0", "computed initial after nav");
    await clickUnique(page.getByTestId("about-bump"), "about bump button");
    await expectText(page.getByTestId("about-doubled"), "Doubled: 2", "computed replays after SPA nav");

    assert(consoleErrors.length === 0, `browser console errors:\n${consoleErrors.join("\n")}`);
  } finally {
    await browser.close();
  }
}

const server = startServer();

try {
  await waitForServer();
  await runBrowserChecks();
  log("all browser checks passed");
} finally {
  stopServer(server);
}
