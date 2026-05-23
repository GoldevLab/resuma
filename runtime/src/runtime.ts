/**
 * Resuma client runtime.
 *
 * The runtime is intentionally tiny (~3KB minified). It does *not* re-execute
 * components. Instead it:
 *
 *   1. Reads the resumability payload embedded as
 *      `<script type="resuma/state">…</script>`.
 *   2. Reconstructs each `Signal` as a tiny reactive cell with `.value`,
 *      `.set()`, `.update()` methods.
 *   3. Wires data-r-bind attributes for reactive DOM updates.
 *   4. Listens for *every* DOM event at the document level. When a node has a
 *      matching `data-r-on:*` attribute it lazy-loads the handler chunk and
 *      executes the handler with `(event, state, actions)`.
 *   5. Provides `__resuma.action(name, args)` which POSTs to
 *      `/_resuma/action/<name>` and returns the response JSON.
 *
 * This is the closest analog to Qwik's resumability model — the page is
 * "frozen" by the server, and the client thaws individual interactions on
 * demand.
 */

import { initSignals, type SignalCell, applyDom, bindReactiveText, bindReactiveAttrs } from "./signals.js";
import { initIslands } from "./islands.js";
import { resolveHandler } from "./loader.js";

interface ResumePayload {
  signals: Array<{ id: { 0: number } | string; value: unknown }>;
  handlers: Record<string, Record<string, string>>;
  islands: string[];
  actions: string[];
}

interface ResumaGlobal {
  state: Record<string, SignalCell<unknown>>;
  signals: Map<string, SignalCell<unknown>>;
  handlers: Record<string, Record<string, string>>;
  action: (name: string, args: unknown[]) => Promise<unknown>;
  loaded: Map<string, Record<string, Function>>;
  refreshIsland: (id: string) => Promise<void>;
}

declare global {
  interface Window { __resuma?: ResumaGlobal; }
}

const STATE_SCRIPT_ID = "resuma-state";
const ROOT_ID = "resuma-root";
const HANDLER_PREFIX = "data-r-on:";
const CAPTURES_PREFIX = "data-r-cap:";
const INLINE_PREFIX = "data-r-inline:";

const root = (): HTMLElement => document.getElementById(ROOT_ID) ?? document.body;

function readPayload(): ResumePayload {
  const node = document.getElementById(STATE_SCRIPT_ID);
  if (!node || !node.textContent) return { signals: [], handlers: {}, islands: [], actions: [] };
  try {
    return JSON.parse(node.textContent) as ResumePayload;
  } catch (e) {
    console.error("[resuma] failed to parse state payload", e);
    return { signals: [], handlers: {}, islands: [], actions: [] };
  }
}

function bootstrap(): void {
  const payload = readPayload();
  const signals = initSignals(payload.signals.map((s) => ({
    id: typeof s.id === "string" ? s.id : `s${(s.id as { 0: number })[0]}`,
    value: s.value,
  })));

  const state: Record<string, SignalCell<unknown>> = {};
  for (const [k, cell] of signals) state[k] = cell;

  const __resuma: ResumaGlobal = {
    state,
    signals,
    handlers: payload.handlers,
    loaded: new Map(),
    action: callServerAction,
    refreshIsland,
  };
  window.__resuma = __resuma;

  bindReactiveText(root(), signals);
  bindReactiveAttrs(root(), signals);
  initIslands(root(), signals);
  attachEventDelegation();
}

/* ------------------------------------------------------------------- */
/*  Event delegation                                                   */
/* ------------------------------------------------------------------- */

const KNOWN_EVENTS = [
  "click", "input", "change", "submit", "focus", "blur", "keydown",
  "keyup", "keypress", "mousedown", "mouseup", "mousemove", "mouseenter",
  "mouseleave", "pointerdown", "pointerup", "pointermove", "touchstart",
  "touchend", "scroll", "wheel", "dragstart", "dragend", "drop",
];

function attachEventDelegation(): void {
  for (const ev of KNOWN_EVENTS) {
    document.addEventListener(ev, dispatchEvent, true);
  }
}

async function dispatchEvent(ev: Event): Promise<void> {
  let target = ev.target as HTMLElement | null;
  if (!target) return;

  const attr = HANDLER_PREFIX + ev.type;
  const capAttr = CAPTURES_PREFIX + ev.type;
  const inlineAttr = INLINE_PREFIX + ev.type;

  while (target && target !== document.body) {
    const ref = target.getAttribute(attr);
    if (ref) {
      const captures = (target.getAttribute(capAttr) ?? "")
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      const inline = target.getAttribute(inlineAttr);
      try {
        const fn = await resolveHandler(ref, inline);
        const localState = buildLocalState(captures);
        const actions = window.__resuma!;
        await fn(ev, localState, actions);
      } catch (err) {
        console.error("[resuma] handler error", err);
      }
      return;
    }
    target = target.parentElement;
  }
}

function buildLocalState(captures: string[]): Record<string, SignalCell<unknown>> {
  // Each capture is a `name:id` pair — name is the Rust identifier, id is
  // the stable signal id allocated by the SSR pass.
  const r = window.__resuma!;
  if (!captures.length) return r.state;
  const local: Record<string, SignalCell<unknown>> = {};
  for (const pair of captures) {
    const [name, id] = pair.split(":");
    const key = id ?? name;
    const cell = r.signals.get(key);
    if (cell) local[name] = cell;
  }
  return Object.assign(Object.create(r.state), local);
}

/* ------------------------------------------------------------------- */
/*  Server actions                                                     */
/* ------------------------------------------------------------------- */

async function callServerAction(name: string, args: unknown[]): Promise<unknown> {
  const res = await fetch(`/_resuma/action/${encodeURIComponent(name)}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ args }),
  });
  if (!res.ok) throw new Error(`[resuma] action ${name} failed: ${res.status}`);
  const data = await res.json();
  if (data.ok === false) throw new Error(data.error ?? "action failed");
  return data.value;
}

/* ------------------------------------------------------------------- */
/*  Island refresh — used by the dev server hot reload                 */
/* ------------------------------------------------------------------- */

async function refreshIsland(instance: string): Promise<void> {
  const res = await fetch(`/_resuma/island/${encodeURIComponent(instance)}`);
  if (!res.ok) return;
  const html = await res.text();
  const target = document.querySelector(`resuma-island[data-r-instance="${instance}"]`);
  if (target) target.outerHTML = html;
  applyDom();
}

/* ------------------------------------------------------------------- */
/*  Boot                                                               */
/* ------------------------------------------------------------------- */

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrap, { once: true });
} else {
  bootstrap();
}
