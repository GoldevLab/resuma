/**
 * Resuma loader — tiny bootstrap (~1–2 KB minified).
 *
 * Registers document-level event listeners and lazy-loads `core.js` on first
 * interaction or immediately when the page needs reactive bindings upfront.
 */

const STATE_SCRIPT_ID = "resuma-state";
const ROOT_ID = "resuma-root";
const HANDLER_PREFIX = "data-r-on:";
const CAPTURES_PREFIX = "data-r-cap:";
const INLINE_PREFIX = "data-r-inline:";
const CORE_URL = "/_resuma/core.js";

interface ResumePayload {
  signals: unknown[];
  handlers: Record<string, Record<string, string>>;
  islands: string[];
  visible_tasks?: Record<string, string>;
  lazy_chunks?: string[];
}

const KNOWN_EVENTS = [
  "click",
  "input",
  "change",
  "submit",
  "focus",
  "blur",
  "keydown",
  "keyup",
  "keypress",
  "mousedown",
  "mouseup",
  "mousemove",
  "mouseenter",
  "mouseleave",
  "pointerdown",
  "pointerup",
  "pointermove",
  "touchstart",
  "touchend",
  "scroll",
  "wheel",
  "dragstart",
  "dragend",
  "drop",
  "load",
];

function readPayload(): ResumePayload {
  const node = document.getElementById(STATE_SCRIPT_ID);
  if (!node || !node.textContent) {
    return { signals: [], handlers: {}, islands: [] };
  }
  try {
    return JSON.parse(node.textContent) as ResumePayload;
  } catch {
    return { signals: [], handlers: {}, islands: [] };
  }
}

function pageRoot(): HTMLElement {
  return document.getElementById(ROOT_ID) ?? document.body;
}

function needsCoreNow(payload: ResumePayload, scope: HTMLElement): boolean {
  if (payload.signals.length) return true;
  if (payload.islands.length) return true;
  if (payload.visible_tasks && Object.keys(payload.visible_tasks).length) return true;
  if (payload.lazy_chunks?.length) return true;
  return !!scope.querySelector(
    "resuma-island, resuma-boundary, resuma-dyn, [data-r-bind], [data-r-submit], template[data-r-portal], template[data-r-stream-chunk], [data-r-vt]",
  );
}

async function ensureCore(): Promise<void> {
  if (!window.__resumaCoreReady) {
    window.__resumaCoreReady = import(CORE_URL).then((mod) => mod.bootstrap());
  }
  await window.__resumaCoreReady;
}

function eventTargetElement(ev: Event): Element | null {
  const t = ev.target;
  if (t instanceof Element) return t;
  if (t instanceof Text) return t.parentElement;
  return null;
}

async function dispatchEvent(ev: Event): Promise<void> {
  let target = eventTargetElement(ev);
  if (!target) return;

  const attr = HANDLER_PREFIX + ev.type;
  const capAttr = CAPTURES_PREFIX + ev.type;
  const inlineAttr = INLINE_PREFIX + ev.type;

  while (target && target !== document.body) {
    const prevent = target.getAttribute(`data-r-prevent:${ev.type}`);
    if (prevent !== null) ev.preventDefault();
    const stop = target.getAttribute(`data-r-stop:${ev.type}`);
    if (stop !== null) ev.stopPropagation();

    const ref = target.getAttribute(attr);
    if (ref) {
      const captures = (target.getAttribute(capAttr) ?? "")
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      const inline = target.getAttribute(inlineAttr);
      try {
        await ensureCore();
        const core = await import(CORE_URL);
        await core.runHandler(ref, inline, ev, captures);
      } catch (err) {
        console.error("[resuma] handler error", err);
      }
      return;
    }
    target = target.parentElement;
  }
}

function attachDelegation(): void {
  for (const ev of KNOWN_EVENTS) {
    document.addEventListener(ev, dispatchEvent, true);
  }
}

function boot(): void {
  attachDelegation();
  const payload = readPayload();
  if (needsCoreNow(payload, pageRoot())) {
    void ensureCore();
  }
}

declare global {
  interface Window {
    __resumaCoreReady?: Promise<void>;
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", boot, { once: true });
} else {
  boot();
}
