/**
 * Client-side navigation for `<NavLink data-r-nav>` — fetches SSR HTML and
 * swaps `#resuma-root` + `#resuma-state` without a full document reload.
 */

import { bindReactiveAttrs, bindReactiveText, initSignals, type SignalCell, type RawSignalId } from "./signals.js";
import { initIslands } from "./islands.js";

interface ResumePayload {
  signals: Array<{ id: RawSignalId; value: unknown }>;
  handlers: Record<string, Record<string, string>>;
  islands: string[];
  actions: string[];
  contexts?: Record<string, unknown>;
  visible_tasks?: Record<string, string>;
}

interface ResumaGlobal {
  state: Record<string, SignalCell<unknown>>;
  signals: Map<string, SignalCell<unknown>>;
  handlers: Record<string, Record<string, string>>;
  contexts: Record<string, unknown>;
  action: (name: string, args: unknown[]) => Promise<unknown>;
  safeAction: (name: string, args: unknown[]) => Promise<{ ok: true; value: unknown } | { ok: false; error: string }>;
  loaded: Map<string, Record<string, Function>>;
  refreshIsland: (id: string) => Promise<void>;
  context: (key: string) => unknown;
  /** SPA navigation — refetches SSR HTML and remounts signals (use for query-driven `#[load]`). */
  navigate: (href: string, pushState?: boolean) => Promise<void>;
  /** Build same-origin path + query (`{ fecha: "2026-01-01" }`). */
  buildUrl: (path: string, query?: Record<string, string | null | undefined>) => string;
}

const ROOT_ID = "resuma-root";
const STATE_SCRIPT_ID = "resuma-state";

function root(): HTMLElement {
  return document.getElementById(ROOT_ID) ?? document.body;
}

function readPayloadFromScript(node: HTMLElement | null): ResumePayload {
  if (!node?.textContent) {
    return { signals: [], handlers: {}, islands: [], actions: [] };
  }
  try {
    return JSON.parse(node.textContent) as ResumePayload;
  } catch (e) {
    console.error("[resuma] failed to parse state payload", e);
    return { signals: [], handlers: {}, islands: [], actions: [] };
  }
}

function pathsMatch(href: string, current: string): boolean {
  if (href === current) return true;
  const base = "http://resuma.local";
  const a = new URL(href, base);
  const b = new URL(current, base);
  if (a.search) {
    return a.pathname + a.search === b.pathname + b.search;
  }
  if (a.pathname === b.pathname) return true;
  if (a.pathname !== "/" && b.pathname.startsWith(a.pathname)) {
    const next = b.pathname.charCodeAt(a.pathname.length);
    return next === undefined || next === 47;
  }
  return false;
}

function updateNavActiveClasses(path: string): void {
  document.querySelectorAll<HTMLAnchorElement>("a[data-r-nav]").forEach((a) => {
    const href = a.getAttribute("href");
    if (!href) return;
    const activeClass = a.getAttribute("data-r-active-class");
    if (!activeClass) return;
    const base = (a.getAttribute("data-r-base-class") ?? a.className)
      .split(/\s+/)
      .filter((c) => c && c !== activeClass)
      .join(" ");
    a.setAttribute("data-r-base-class", base);
    a.className = pathsMatch(href, path) ? `${base} ${activeClass}`.trim() : base;
  });
}

/** Re-mount signals and bindings after swapping page HTML. */
export function remountPage(): void {
  const payload = readPayloadFromScript(document.getElementById(STATE_SCRIPT_ID));
  const signals = initSignals(payload.signals);

  const state: Record<string, SignalCell<unknown>> = {};
  for (const [k, cell] of signals) state[k] = cell;

  const prev = window.__resuma;
  if (!prev) {
    // Core not yet bootstrapped — let the loader/core take over via full load.
    window.location.reload();
    return;
  }
  const __resuma: ResumaGlobal = {
    state,
    signals,
    handlers: payload.handlers,
    contexts: payload.contexts ?? {},
    loaded: prev.loaded ?? new Map(),
    action: prev.action,
    safeAction: prev.safeAction,
    refreshIsland: prev.refreshIsland,
    context: (key: string) => __resuma.contexts[key],
    navigate,
    buildUrl,
  };
  window.__resuma = __resuma;

  const scope = root();
  bindReactiveText(scope, signals);
  bindReactiveAttrs(scope, signals);
  initIslands(scope, signals);
  updateNavActiveClasses(location.pathname + location.search);
}

/** Move focus to the page root for assistive tech after an SPA swap. */
function focusMain(): void {
  const scope = root();
  const target =
    scope.querySelector<HTMLElement>("[autofocus]") ??
    scope.querySelector<HTMLElement>("h1, [role='heading'], main") ??
    scope;
  if (!target.hasAttribute("tabindex")) target.setAttribute("tabindex", "-1");
  try {
    target.focus({ preventScroll: true });
  } catch {
    target.focus();
  }
}

/** Build `/path?key=value` for SPA navigation. Null/empty values are skipped. */
export function buildUrl(
  path: string,
  query?: Record<string, string | null | undefined>,
): string {
  const url = new URL(path, location.origin);
  if (query) {
    for (const [key, value] of Object.entries(query)) {
      if (value != null && value !== "") url.searchParams.set(key, value);
    }
  }
  return url.pathname + url.search;
}

export async function navigate(href: string, pushState = true): Promise<void> {
  try {
    const res = await fetch(href, {
      headers: { Accept: "text/html" },
      credentials: "same-origin",
    });
    if (!res.ok) {
      window.location.href = href;
      return;
    }
    const html = await res.text();
    const doc = new DOMParser().parseFromString(html, "text/html");
    const newRoot = doc.getElementById(ROOT_ID);
    const newState = doc.getElementById(STATE_SCRIPT_ID);
    if (!newRoot || !newState?.textContent) {
      window.location.href = href;
      return;
    }

    root().innerHTML = newRoot.innerHTML;
    const stateScript = document.getElementById(STATE_SCRIPT_ID);
    if (stateScript) stateScript.textContent = newState.textContent;
    if (doc.title) document.title = doc.title;

    if (pushState) history.pushState({ resumaNav: true }, "", href);
    remountPage();
    // New navigations land at the top; back/forward keep the browser's scroll.
    if (pushState) window.scrollTo(0, 0);
    focusMain();
  } catch (err) {
    console.error("[resuma] navigation failed", err);
    window.location.href = href;
  }
}

/** Follow redirect hints from submit/action JSON — uses SPA nav for same-origin paths. */
export function followRedirect(path: string): void {
  if (path.startsWith("/") && !path.startsWith("//")) {
    void navigate(path);
  } else {
    window.location.assign(path);
  }
}

function shouldEnhanceLink(a: HTMLAnchorElement, ev: MouseEvent): boolean {
  if (ev.defaultPrevented || ev.button !== 0) return false;
  if (ev.metaKey || ev.ctrlKey || ev.shiftKey || ev.altKey) return false;
  if (a.target && a.target !== "_self") return false;
  const href = a.getAttribute("href");
  if (!href || href.startsWith("#") || href.startsWith("javascript:")) return false;
  if (href.startsWith("http://") || href.startsWith("https://")) {
    try {
      const u = new URL(href);
      return u.origin === location.origin;
    } catch {
      return false;
    }
  }
  return true;
}

export function initNavLinks(): void {
  document.addEventListener("click", (ev) => {
    const target = ev.target;
    if (!(target instanceof Element)) return;
    const a = target.closest("a[data-r-nav]") as HTMLAnchorElement | null;
    if (!a || !shouldEnhanceLink(a, ev as MouseEvent)) return;
    const href = a.getAttribute("href");
    if (!href) return;
    ev.preventDefault();
    void navigate(href);
  });

  window.addEventListener("popstate", () => {
    void navigate(location.pathname + location.search, false);
  });
}

declare global {
  interface Window { __resuma?: ResumaGlobal; }
}
