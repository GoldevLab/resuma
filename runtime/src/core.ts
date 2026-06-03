/**
 * Resuma client core — lazy-loaded after the tiny loader bootstrap.
 * Signals, islands, forms, streaming slots, portals, and server actions.
 */

import { initSignals, type SignalCell, applyDom, bindReactiveText, bindReactiveAttrs, type RawSignalId } from "./signals.js";
import { initIslands } from "./islands.js";
import { initEffects, type ClientEffectSpec } from "./effects.js";
import { prefetchLazyChunks } from "./boundaries.js";
import { resolveHandler, type Handler } from "./handler-loader.js";
import { initNavLinks, followRedirect, navigate, buildUrl, setPageMounter } from "./navigation.js";

interface ResumePayload {
  signals: Array<{ id: RawSignalId; value: unknown }>;
  handlers: Record<string, Record<string, string>>;
  islands: string[];
  actions: string[];
  contexts?: Record<string, unknown>;
  visible_tasks?: Record<string, string>;
  effects?: ClientEffectSpec[];
  lazy_chunks?: string[];
  csrf_token?: string;
}

export interface ResumaGlobal {
  state: Record<string, SignalCell<unknown>>;
  signals: Map<string, SignalCell<unknown>>;
  handlers: Record<string, Record<string, string>>;
  contexts: Record<string, unknown>;
  action: (name: string, args: unknown[]) => Promise<unknown>;
  safeAction: (
    name: string,
    args: unknown[],
  ) => Promise<{ ok: true; value: unknown } | { ok: false; error: string }>;
  loaded: Map<string, Record<string, Function>>;
  refreshIsland: (id: string) => Promise<void>;
  context: (key: string) => unknown;
  navigate: (href: string, pushState?: boolean) => Promise<void>;
  buildUrl: (path: string, query?: Record<string, string | null | undefined>) => string;
}

declare global {
  interface Window {
    __resuma?: ResumaGlobal;
    __resumaCoreReady?: Promise<void>;
  }
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

function mutationHeaders(extra: Record<string, string> = {}): Record<string, string> {
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
    return JSON.parse(node.textContent) as ResumePayload;
  } catch (e) {
    console.error("[resuma] failed to parse state payload", e);
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
  };
  window.__resuma = __resuma;

  const scope = root();
  bindReactiveText(scope, signals);
  bindReactiveAttrs(scope, signals);
  initIslands(scope, signals);
  applyStreamSlots(scope);
  initPortals(scope);
  initViewTransitions(scope);
  runVisibleTasks(payload.visible_tasks ?? {}, state);
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
  connectDevBridge();
}

export function buildLocalState(captures: string[]): Record<string, SignalCell<unknown>> {
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
            console.error("[resuma] submit error", data.error ?? `submit ${name} failed`);
          }
          return;
        }
        clearFieldErrors(form);
        if (data.redirect) followRedirect(data.redirect);
      } catch (err) {
        console.error("[resuma] submit error", err);
      }
    },
    true,
  );
}

function showFieldErrors(form: HTMLFormElement, errors: Record<string, string>): void {
  clearFieldErrors(form);
  for (const [name, message] of Object.entries(errors)) {
    const input = form.querySelector(`[name="${name}"]`) as HTMLElement | null;
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

function initPortals(scope: HTMLElement): void {
  scope.querySelectorAll("template[data-r-portal]").forEach((tpl) => {
    const targetId = tpl.getAttribute("data-r-portal");
    if (!targetId) return;
    const target =
      document.getElementById(targetId) ??
      document.querySelector(`[data-r-portal-target="${targetId}"]`);
    if (!target) return;
    const frag = document.createDocumentFragment();
    while (tpl.content.firstChild) frag.appendChild(tpl.content.firstChild);
    target.appendChild(frag);
    tpl.remove();
  });
}

function initViewTransitions(scope: HTMLElement): void {
  if (!("startViewTransition" in document)) return;
  scope.querySelectorAll("[data-r-vt]").forEach((el) => {
    el.addEventListener("click", (ev) => {
      const anchor = (ev.target as HTMLElement | null)?.closest("a[href]");
      if (!anchor || anchor.getAttribute("target") === "_blank") return;
      const href = anchor.getAttribute("href");
      if (!href || href.startsWith("#") || href.startsWith("javascript:")) return;
      ev.preventDefault();
      const run = () => {
        window.location.href = href;
      };
      (document as Document & { startViewTransition?: (cb: () => void) => void }).startViewTransition?.(
        run,
      );
    });
  });
}

function runVisibleTasks(
  tasks: Record<string, string>,
  state: Record<string, SignalCell<unknown>>,
): void {
  const entries = Object.entries(tasks);
  if (!entries.length) return;

  const run = (id: string, source: string) => {
    try {
      const fn = new Function("state", "__resuma", `return ${source}`) as (
        state: unknown,
        resuma: ResumaGlobal,
      ) => Promise<void> | void;
      void Promise.resolve(fn(state, window.__resuma!));
    } catch (err) {
      console.error("[resuma] visible task", id, err);
    }
  };

  if ("IntersectionObserver" in window) {
    const io = new IntersectionObserver(
      (entries, obs) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          const id = (entry.target as HTMLElement).dataset.rVisibleTask;
          const source = id ? tasks[id] : undefined;
          if (source) run(id, source);
          obs.unobserve(entry.target);
        }
      },
      { rootMargin: "50px" },
    );
    for (const [id] of entries) {
      const marker = document.createElement("span");
      marker.hidden = true;
      marker.dataset.rVisibleTask = id;
      root().appendChild(marker);
      io.observe(marker);
    }
  } else {
    for (const [id, source] of entries) run(id, source);
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
  if (data.redirect) followRedirect(data.redirect);
  return data.value;
}

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

async function refreshIsland(instance: string): Promise<void> {
  const res = await fetch(`/_resuma/island/${encodeURIComponent(instance)}`);
  if (!res.ok) return;
  const html = await res.text();
  const target = document.querySelector(`resuma-island[data-r-instance="${instance}"]`);
  if (target) target.outerHTML = html;
  applyDom();
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
