/**
 * Resuma client core — lazy-loaded after the tiny loader bootstrap.
 * Signals, islands, forms, streaming slots, portals, and server actions.
 */

import { initSignals, signalId, type SignalCell, bindReactiveText, bindReactiveAttrs, bindShows, bindFor, bindMatch, type RawSignalId } from "./signals.js";
import { initIslands } from "./islands.js";
import { initEffects, flushEffectCleanups, type ClientEffectSpec } from "./effects.js";
import { prefetchLazyChunks } from "./boundaries.js";
import { invalidateLazyChunks, warmHandlerChunks, warmIslandChunks, clearInlineHandlerCache } from "./handler-loader.js";
import { resolveHandler, type Handler } from "./handler-loader.js";
import { initNavLinks, followRedirect, navigate, buildUrl, invalidate, setPageMounter, updateNavActiveClasses } from "./navigation.js";
import { runVisibleTasks, type VisibleTaskEntry } from "./visible-tasks.js";
import { flushMountCleanups } from "./mount-cleanups.js";
import { beginPortalMount, mountStaticPortals } from "./portals.js";
import type { ResumaGlobal } from "./types.js";
import "./types.js";

export type { ResumaGlobal } from "./types.js";

const FLOW_WIDGET_SELECTOR =
  "[data-r-flow-dashboard], [data-r-flow-graph], [data-r-event-stream], [data-r-worker-panel]";

function hasFlowWidgets(scope: ParentNode): boolean {
  if (scope instanceof Document) {
    return !!scope.querySelector(FLOW_WIDGET_SELECTOR);
  }
  if (scope instanceof Element) {
    return !!scope.querySelector(FLOW_WIDGET_SELECTOR);
  }
  return false;
}

async function maybeInitFlowWidgets(scope: ParentNode): Promise<void> {
  if (!hasFlowWidgets(scope)) return;
  try {
    const mod = await import("/_resuma/flow.js");
    mod.initFlowWidgets(scope);
  } catch (err) {
    console.error("[r] flow load", err);
  }
}

/** Mount Flow widgets inside `scope` (or `document` when omitted). */
export function mountFlowWidgets(scope: ParentNode = document): void {
  void maybeInitFlowWidgets(scope);
}

interface ResumePayload {
  signals: Array<{ id: RawSignalId; value: unknown }>;
  handlers: Record<string, Record<string, string>>;
  islands: string[];
  actions: string[];
  contexts?: Record<string, unknown>;
  visible_tasks?: Record<string, VisibleTaskEntry>;
  effects?: ClientEffectSpec[];
  lazy_chunks?: string[];
  chunk_digests?: Record<string, string>;
  csrf_token?: string;
  serialization_error?: boolean;
}

const STATE_SCRIPT_ID = "resuma-state";
const ROOT_ID = "resuma-root";

function csrfToken(): string {
  const node = document.getElementById(STATE_SCRIPT_ID);
  if (!node?.textContent) return "";
  try {
    const payload = JSON.parse(node.textContent) as ResumePayload;
    return payload.csrf_token ?? "";
  } catch {
    return "";
  }
}

export function mutationHeaders(extra: Record<string, string> = {}): Record<string, string> {
  const headers: Record<string, string> = { ...extra };
  const token = csrfToken();
  if (token) headers["x-resuma-csrf"] = token;
  return headers;
}

const root = (): HTMLElement => document.getElementById(ROOT_ID) ?? document.body;

function readPayload(): ResumePayload {
  const node = document.getElementById(STATE_SCRIPT_ID);
  if (!node || !node.textContent) {
    return { signals: [], handlers: {}, islands: [], actions: [] };
  }
  try {
    const payload = JSON.parse(node.textContent) as ResumePayload;
    if (payload.serialization_error) {
      console.error(
        "[r] resumability payload failed to serialize on the server — page interactivity is broken",
      );
    }
    return payload;
  } catch (e) {
    console.error("[r] state", e);
    return { signals: [], handlers: {}, islands: [], actions: [] };
  }
}

let bootstrapped = false;

/**
 * Mount (or re-mount) the current page: rebuild the `__resuma` global from the
 * `#resuma-state` payload, then run every per-page initializer.
 *
 * Called on first load by [`bootstrap`] and again on each SPA navigation (via
 * the mounter registered with `setPageMounter`). Document-level listeners that
 * must attach only once (forms, NavLink delegation, dev bridge) live in
 * [`bootstrap`], not here.
 */
export function mountPage(): void {
  // Tear down the previous mount's effect subscriptions, debounce timers, and
  // viewport observers before re-initializing against fresh signal cells.
  flushEffectCleanups();
  flushMountCleanups();
  beginPortalMount();

  const payload = readPayload();
  const signals = initSignals(payload.signals);

  const state: Record<string, SignalCell<unknown>> = {};
  for (const [k, cell] of signals) state[k] = cell;

  const prev = window.__resuma;
  const loaded = prev?.loaded ?? new Map<string, Record<string, Function>>();
  const islandLoaded = prev?.islandLoaded ?? new Map<string, Record<string, Function>>();
  const prevDigests = prev?.chunkDigests ?? {};
  const newDigests = payload.chunk_digests ?? {};

  const handlerChunks = new Set(payload.lazy_chunks ?? []);
  const islandChunks = new Set(payload.islands ?? []);
  for (const el of document.querySelectorAll<HTMLElement>("[data-r-on\\:click], [data-r-on\\:submit], [data-r-on\\:input], [data-r-on\\:change]")) {
    for (const attr of el.attributes) {
      if (!attr.name.startsWith("data-r-on:")) continue;
      const ref = attr.value;
      const hash = ref.indexOf("#");
      if (hash === -1) continue;
      const chunk = ref.slice(0, hash);
      if (chunk && chunk !== "__page__") handlerChunks.add(chunk);
    }
  }

  const scope = root();
  for (const el of scope.querySelectorAll<HTMLElement>("resuma-island[data-r-chunk]")) {
    const chunk = el.getAttribute("data-r-chunk");
    if (chunk) islandChunks.add(chunk);
    delete el.dataset.rHydrated;
  }

  const staleHandlers: string[] = [];
  for (const chunk of handlerChunks) {
    if (!newDigests[chunk] || prevDigests[chunk] !== newDigests[chunk]) {
      staleHandlers.push(chunk);
    }
  }
  const staleIslands: string[] = [];
  for (const chunk of islandChunks) {
    if (!newDigests[chunk] || prevDigests[chunk] !== newDigests[chunk]) {
      staleIslands.push(chunk);
    }
  }
  if (staleHandlers.length || staleIslands.length) {
    invalidateLazyChunks(
      [
        ...staleHandlers.map((chunk) => ({ kind: "handler" as const, chunk })),
        ...staleIslands.map((chunk) => ({ kind: "island" as const, chunk })),
      ],
      loaded,
      islandLoaded,
    );
  }
  clearInlineHandlerCache();

  const __resuma: ResumaGlobal = {
    state,
    signals,
    handlers: payload.handlers,
    contexts: payload.contexts ?? {},
    loaded,
    islandLoaded,
    chunkDigests: newDigests,
    action: callServerAction,
    safeAction: callServerActionSafe,
    refreshIsland,
    context: (key: string) => __resuma.contexts[key],
    navigate,
    buildUrl,
    invalidate,
  };
  window.__resuma = __resuma;
  if (handlerChunks.size) warmHandlerChunks(handlerChunks);
  if (islandChunks.size) warmIslandChunks(islandChunks);

  bindReactiveText(scope, signals);
  bindReactiveAttrs(scope, signals);
  bindShows(scope, signals);
  bindFor(scope, signals);
  bindMatch(scope, signals);
  initIslands(scope, signals);
  applyStreamSlots(scope);
  mountStaticPortals(scope);
  initViewTransitions(scope);
  void maybeInitFlowWidgets(scope);
  runVisibleTasks(payload.visible_tasks ?? {}, signals, state, root);
  initEffects(payload.effects ?? [], signals, __resuma);
  prefetchLazyChunks(payload.lazy_chunks ?? [], scope);
}

/** Initialize signals, DOM bindings, and progressive enhancements. */
export async function bootstrap(): Promise<void> {
  if (bootstrapped) return;
  bootstrapped = true;

  // SPA navigation replays the same full mount pipeline as first load.
  setPageMounter(mountPage);
  mountPage();

  // Document-level listeners — attach exactly once.
  attachFormEnhancement();
  initLoaderRefreshForms();
  initNavLinks();
  updateNavActiveClasses(location.pathname + location.search);
  connectDevBridge();
}

export function buildLocalState(captures: string[]): Record<string, SignalCell<unknown>> {
  const r = window.__resuma!;
  if (!captures.length) return r.state;
  const local: Record<string, SignalCell<unknown>> = {};
  for (const pair of captures) {
    // Split on the first `:` only — signal ids may themselves contain colons.
    const sep = pair.indexOf(":");
    const name = sep === -1 ? pair : pair.slice(0, sep);
    const id = sep === -1 ? undefined : pair.slice(sep + 1);
    const key = id != null ? signalId(id) : name;
    const cell = r.signals.get(key);
    if (cell) local[name] = cell;
  }
  return Object.assign(Object.create(r.state), local);
}

export async function runHandler(
  ref: string,
  inline: string | null,
  ev: Event,
  captures: string[],
): Promise<void> {
  const fn: Handler = await resolveHandler(ref, inline);
  const localState = buildLocalState(captures);
  await fn(ev, localState, window.__resuma!);
}

function attachFormEnhancement(): void {
  document.addEventListener(
    "submit",
    async (ev) => {
      if (!(ev.target instanceof HTMLFormElement)) return;
      const form = ev.target;
      if (!form.getAttribute("data-r-submit")) return;
      ev.preventDefault();
      const name = form.getAttribute("data-r-submit")!;
      const fd = new FormData(form);
      const body: Record<string, string> = {};
      fd.forEach((v, k) => {
        body[k] = String(v);
      });
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
            console.error("[r] submit", data.error ?? name);
          }
          return;
        }
        clearFieldErrors(form);
        if (data.redirect) followRedirect(data.redirect);
      } catch (err) {
        console.error("[r] submit", err);
      }
    },
    true,
  );
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

/** GET forms marked with `data-r-loader-refresh` (see `loader_refresh_form` in Rust). */
function initLoaderRefreshForms(): void {
  document.addEventListener(
    "submit",
    (ev) => {
      if (!(ev.target instanceof HTMLFormElement)) return;
      const form = ev.target;
      if (!form.hasAttribute("data-r-loader-refresh")) return;
      if (form.getAttribute("data-r-on:submit") || form.querySelector("[data-r-on\\:submit]")) return;
      ev.preventDefault();
      const fd = new FormData(form);
      const params: Record<string, string> = {};
      fd.forEach((val, key) => {
        const s = String(val);
        if (s) params[key] = s;
      });
      const action = form.getAttribute("action") || location.pathname;
      void window.__resuma?.navigate(buildUrl(action, params));
    },
    true,
  );
}

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
      // NavLink clicks use navigate(), which already wraps SPA swaps in VT.
      if (anchor.hasAttribute("data-r-nav")) return;
      ev.preventDefault();
      void navigateForViewTransition(href);
    });
  });
}

interface ActionResponse {
  ok?: boolean;
  value?: unknown;
  error?: string;
  field_errors?: Record<string, string>;
  redirect?: string;
}

async function callServerAction(name: string, args: unknown[]): Promise<unknown> {
  const res = await fetch(`/_resuma/action/${encodeURIComponent(name)}`, {
    method: "POST",
    credentials: "same-origin",
    headers: mutationHeaders({ "content-type": "application/json" }),
    body: JSON.stringify({ args }),
  });
  let data: ActionResponse;
  try {
    data = (await res.json()) as ActionResponse;
  } catch {
    if (!res.ok) throw new Error(`action ${name}: ${res.status}`);
    throw new Error(`action ${name}: invalid response`);
  }
  if (!res.ok || data.ok === false) {
    throw new Error(data.error ?? `action ${name}: ${res.status}`);
  }
  if (data.redirect) followRedirect(data.redirect);
  return data.value;
}

async function callServerActionSafe(
  name: string,
  args: unknown[],
): Promise<
  | { ok: true; value: unknown }
  | { ok: false; error: string; field_errors?: Record<string, string> }
> {
  try {
    const res = await fetch(`/_resuma/action/${encodeURIComponent(name)}`, {
      method: "POST",
      credentials: "same-origin",
      headers: mutationHeaders({ "content-type": "application/json" }),
      body: JSON.stringify({ args }),
    });
    let data: ActionResponse;
    try {
      data = (await res.json()) as ActionResponse;
    } catch {
      return { ok: false, error: `action ${name}: ${res.status}` };
    }
    if (!res.ok || data.ok === false) {
      return {
        ok: false,
        error: data.error ?? `action ${name}: ${res.status}`,
        field_errors: data.field_errors,
      };
    }
    if (data.redirect) followRedirect(data.redirect);
    return { ok: true, value: data.value };
  } catch (err) {
    const error = err instanceof Error ? err.message : String(err);
    return { ok: false, error };
  }
}

/** Show field-level errors returned by `safeAction` near matching named inputs. */
export function showActionFieldErrors(
  scope: ParentNode,
  fieldErrors: Record<string, string>,
): void {
  scope.querySelectorAll("[data-r-field-error]").forEach((n) => n.remove());
  for (const [name, message] of Object.entries(fieldErrors)) {
    const input = scope.querySelector(`[name="${CSS.escape(name)}"]`) as HTMLElement | null;
    if (!input) continue;
    const el = document.createElement("span");
    el.className = "resuma-field-error";
    el.setAttribute("data-r-field-error", name);
    el.textContent = message;
    input.insertAdjacentElement("afterend", el);
  }
}

async function refreshIsland(instance: string): Promise<void> {
  const res = await fetch(`/_resuma/island/${encodeURIComponent(instance)}`);
  if (!res.ok) return;
  const html = await res.text();
  const target = document.querySelector(`resuma-island[data-r-instance="${instance}"]`);
  if (!target) return;
  target.outerHTML = html;

  const fresh = document.querySelector<HTMLElement>(
    `resuma-island[data-r-instance="${CSS.escape(instance)}"]`,
  );
  const signals = window.__resuma?.signals;
  if (!fresh || !signals) return;

  // Re-bind the swapped subtree against the EXISTING signal cells so live
  // client state is preserved (a full `mountPage()` would rebuild every signal
  // from the stale SSR payload and drop user interactions). Event handlers are
  // delegated at the document level by the loader, so they need no rewiring.
  bindReactiveText(fresh, signals);
  bindReactiveAttrs(fresh, signals);
  bindShows(fresh, signals);
  bindFor(fresh, signals);
  bindMatch(fresh, signals);
  initIslands(fresh, signals);
}

function connectDevBridge(): void {
  // Only in dev: the dev-reload script (injected when RESUMA_DEV=1) sets this
  // flag. In production the /_resuma/dev/ws route does not exist, so connecting
  // would loop on reconnects forever.
  if (!(window as unknown as { __resumaDev?: boolean }).__resumaDev) return;
  if (typeof WebSocket === "undefined") return;
  const proto = location.protocol === "https:" ? "wss" : "ws";
  let hadConnection = false;

  const connect = (): void => {
    const ws = new WebSocket(`${proto}://${location.host}/_resuma/dev/ws`);
    ws.addEventListener("open", () => {
      if (hadConnection) {
        location.reload();
        return;
      }
      hadConnection = true;
    });
    ws.addEventListener("message", (ev) => {
      const msg = String(ev.data);
      if (msg === "reload") {
        location.reload();
        return;
      }
      if (msg.startsWith("island:")) {
        void refreshIsland(msg.slice("island:".length));
      }
    });
    ws.addEventListener("close", () => {
      setTimeout(connect, 500);
    });
    ws.addEventListener("error", () => {
      ws.close();
    });
  };

  connect();
}
