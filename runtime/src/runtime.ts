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
 * The page is "frozen" by the server, and the client thaws individual
 * interactions on demand.
 */

import { initSignals, signalId, type SignalCell, bindReactiveText, bindReactiveAttrs, bindShows, bindFor, bindMatch, type RawSignalId } from "./signals.js";
import { initIslands } from "./islands.js";
import { initEffects, flushEffectCleanups, type ClientEffectSpec } from "./effects.js";
import { resolveHandler } from "./handler-loader.js";
import { followRedirect, initNavLinks, setPageMounter, navigate, buildUrl, invalidate } from "./navigation.js";
import { runVisibleTasks, type VisibleTaskEntry } from "./visible-tasks.js";
import { flushMountCleanups } from "./mount-cleanups.js";
import { beginPortalMount, mountStaticPortals } from "./portals.js";
import type { ResumaGlobal } from "./types.js";
import "./types.js";

interface ResumePayload {
  signals: Array<{ id: RawSignalId; value: unknown }>;
  handlers: Record<string, Record<string, string>>;
  islands: string[];
  actions: string[];
  contexts?: Record<string, unknown>;
  visible_tasks?: Record<string, VisibleTaskEntry>;
  effects?: ClientEffectSpec[];
  csrf_token?: string;
  serialization_error?: boolean;
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
    const payload = JSON.parse(node.textContent) as ResumePayload;
    if (payload.serialization_error) {
      console.error(
        "[resuma] resumability payload failed to serialize on the server — page interactivity is broken",
      );
    }
    return payload;
  } catch (e) {
    console.error("[resuma] failed to parse state payload", e);
    return { signals: [], handlers: {}, islands: [], actions: [] };
  }
}

function csrfToken(): string {
  return readPayload().csrf_token ?? "";
}

function mutationHeaders(extra: Record<string, string> = {}): Record<string, string> {
  const headers: Record<string, string> = { ...extra };
  const token = csrfToken();
  if (token) headers["x-resuma-csrf"] = token;
  return headers;
}

function bootstrap(): void {
  // SPA navigation replays the same per-page mount used on first load.
  setPageMounter(mountCurrentPage);
  mountCurrentPage();
  attachEventDelegation();
  attachFormEnhancement();
  initNavLinks();
}

function mountCurrentPage(): void {
  flushEffectCleanups();
  flushMountCleanups();
  beginPortalMount();
  const payload = readPayload();
  const signals = initSignals(payload.signals);

  const state: Record<string, SignalCell<unknown>> = {};
  for (const [k, cell] of signals) state[k] = cell;

  const prev = window.__resuma;
  const __resuma: ResumaGlobal = {
    state,
    signals,
    handlers: payload.handlers,
    contexts: payload.contexts ?? {},
    loaded: prev?.loaded ?? new Map(),
    action: callServerAction,
    safeAction: callServerActionSafe,
    refreshIsland,
    context: (key: string) => __resuma.contexts[key],
    navigate,
    buildUrl,
    invalidate,
  };
  window.__resuma = __resuma;

  bindReactiveText(root(), signals);
  bindReactiveAttrs(root(), signals);
  bindShows(root(), signals);
  bindFor(root(), signals);
  bindMatch(root(), signals);
  initIslands(root(), signals);
  applyStreamSlots(root());
  mountStaticPortals(root());
  initViewTransitions(root());
  if (payload.effects?.length) initEffects(payload.effects, signals, __resuma);
  runVisibleTasks(payload.visible_tasks ?? {}, signals, state, root);
}

/* ------------------------------------------------------------------- */
/*  Event delegation                                                   */
/* ------------------------------------------------------------------- */

const KNOWN_EVENTS = [
  "click", "input", "change", "submit", "focus", "blur", "keydown",
  "keyup", "keypress", "mousedown", "mouseup", "mousemove", "mouseenter",
  "mouseleave", "pointerdown", "pointerup", "pointermove", "touchstart",
  "touchend", "scroll", "wheel", "dragstart", "dragend", "drop", "load",
];

function attachEventDelegation(): void {
  for (const ev of KNOWN_EVENTS) {
    document.addEventListener(ev, dispatchEvent, true);
  }
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
    const sep = pair.indexOf(":");
    const name = sep === -1 ? pair : pair.slice(0, sep);
    const id = sep === -1 ? undefined : pair.slice(sep + 1);
    const key = id != null ? signalId(id) : name;
    const cell = r.signals.get(key);
    if (cell) local[name] = cell;
  }
  return Object.assign(Object.create(r.state), local);
}

/* ------------------------------------------------------------------- */
/*  Flow form submit (progressive enhancement)                       */
/* ------------------------------------------------------------------- */

function attachFormEnhancement(): void {
  document.addEventListener("submit", async (ev) => {
    if (!(ev.target instanceof HTMLFormElement)) return;
    const form = ev.target;
    if (!form.getAttribute("data-r-submit")) return;
    ev.preventDefault();
    const name = form.getAttribute("data-r-submit")!;
    const fd = new FormData(form);
    const body: Record<string, string> = {};
    fd.forEach((v, k) => { body[k] = String(v); });
    const params = new URLSearchParams(body);
    try {
      const res = await fetch(form.action || `/_resuma/submit/${encodeURIComponent(name)}`, {
        method: "POST",
        credentials: "same-origin",
        headers: mutationHeaders({
          "content-type": "application/x-www-form-urlencoded",
          accept: "application/json",
        }),
        body: params.toString(),
      });
      const data = await res.json();
      if (!res.ok || data.ok === false) {
        showFieldErrors(form, data.field_errors ?? {});
        if (res.status >= 500 || !data.field_errors) {
          console.error("[resuma] submit error", data.error ?? `submit ${name} failed`);
        }
        return;
      }
      clearFieldErrors(form);
      if (data.redirect) {
        followRedirect(data.redirect);
        return;
      }
      console.info("[resuma] submit ok", data.value);
    } catch (err) {
      console.error("[resuma] submit error", err);
    }
  }, true);
}

function showFieldErrors(form: HTMLFormElement, errors: Record<string, string>): void {
  clearFieldErrors(form);
  for (const [name, message] of Object.entries(errors)) {
    const input = form.querySelector(`[name="${CSS.escape(name)}"]`) as HTMLElement | null;
    if (!input) continue;
    const el = document.createElement("span");
    el.className = "resuma-field-error";
    el.setAttribute("data-r-field-error", name);
    el.textContent = message;
    input.insertAdjacentElement("afterend", el);
  }
}

function clearFieldErrors(form: HTMLFormElement): void {
  form.querySelectorAll("[data-r-field-error]").forEach((n) => n.remove());
}

/* ------------------------------------------------------------------- */
/*  Streaming SSR slots                                                */
/* ------------------------------------------------------------------- */

function applyStreamSlots(scope: HTMLElement): void {
  scope.querySelectorAll("template[data-r-stream-chunk]").forEach((chunk) => {
    const name = chunk.getAttribute("data-r-stream-chunk");
    if (!name) return;
    const slot = scope.querySelector(`template[data-r-stream="${name}"]`);
    if (!slot || !slot.parentElement) return;
    const html = chunk.innerHTML;
    const frag = document.createRange().createContextualFragment(html);
    slot.replaceWith(frag);
    chunk.remove();
  });
}

/* ------------------------------------------------------------------- */
/*  View Transitions                                                   */
/* ------------------------------------------------------------------- */

function navigateForViewTransition(href: string): void {
  if (href.startsWith("/") && !href.startsWith("//")) {
    void navigate(href);
    return;
  }
  try {
    const url = new URL(href, location.origin);
    if (url.origin === location.origin && (url.protocol === "http:" || url.protocol === "https:")) {
      void navigate(url.pathname + url.search + url.hash);
      return;
    }
  } catch {
    /* fall through */
  }
  window.location.assign(href);
}

function initViewTransitions(scope: HTMLElement): void {
  scope.querySelectorAll("[data-r-vt]").forEach((el) => {
    el.addEventListener("click", (ev) => {
      const anchor = (ev.target as HTMLElement | null)?.closest("a[href]");
      if (!anchor || anchor.getAttribute("target") === "_blank") return;
      const href = anchor.getAttribute("href");
      if (!href || href.startsWith("#") || href.startsWith("javascript:")) return;
      if (anchor.hasAttribute("data-r-nav")) return;
      ev.preventDefault();
      void navigateForViewTransition(href);
    });
  });
}

/* ------------------------------------------------------------------- */
/*  Server actions                                                     */
/* ------------------------------------------------------------------- */

async function callServerActionSafe(
  name: string,
  args: unknown[],
): Promise<{ ok: true; value: unknown } | { ok: false; error: string }> {
  try {
    const value = await callServerAction(name, args);
    return { ok: true, value };
  } catch (err) {
    const error = err instanceof Error ? err.message : String(err);
    return { ok: false, error };
  }
}

async function callServerAction(name: string, args: unknown[]): Promise<unknown> {
  const res = await fetch(`/_resuma/action/${encodeURIComponent(name)}`, {
    method: "POST",
    credentials: "same-origin",
    headers: mutationHeaders({ "content-type": "application/json" }),
    body: JSON.stringify({ args }),
  });
  if (!res.ok) throw new Error(`[resuma] action ${name} failed: ${res.status}`);
  const data = await res.json();
  if (data.ok === false) throw new Error(data.error ?? "action failed");
  if (data.redirect) {
    followRedirect(data.redirect);
    return data.value;
  }
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
  const { applyDom } = await import("./signals.js");
  applyDom();
  const signals = window.__resuma?.signals;
  if (signals) initIslands(root(), signals);
}

/* ------------------------------------------------------------------- */
/*  Boot                                                               */
/* ------------------------------------------------------------------- */

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrap, { once: true });
} else {
  bootstrap();
}
