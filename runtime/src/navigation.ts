/**
 * Client-side navigation for `<NavLink data-r-nav>` — fetches SSR HTML and
 * swaps `#resuma-root` + `#resuma-state` without a full document reload.
 */

import { activateClientComponents, resetClientReady } from "./client-ready.js";
import "./types.js";

const ROOT_ID = "resuma-root";
const STATE_SCRIPT_ID = "resuma-state";

/** Prefetched SSR HTML keyed by same-origin href (NavLink hover). */
const prefetchCache = new Map<string, string>();
const prefetchInFlight = new Set<string>();

/** Sentinel from `Redirect::revalidate()` — soft-invalidate current path. */
export const REVALIDATE_SENTINEL = "__revalidate__";

export type NavigateOptions = {
  /** Push a history entry (default true). Soft invalidates use false. */
  pushState?: boolean;
  /** Scroll to top after swap (default: same as pushState). */
  scroll?: boolean;
  /** Reuse NavLink prefetch cache (default true). Submit redirects must use false. */
  usePrefetch?: boolean;
  /** URL written to history/location (defaults to fetch href; use to strip `_r`). */
  historyHref?: string;
};

async function prefetchRoute(href: string): Promise<void> {
  if (!href || prefetchInFlight.has(href)) return;
  if (href.startsWith("http") && !href.startsWith(location.origin)) return;
  prefetchInFlight.add(href);
  try {
    const res = await fetch(href, {
      headers: { Accept: "text/html" },
      credentials: "same-origin",
      cache: "no-store",
    });
    if (res.ok) prefetchCache.set(href, await res.text());
  } catch {
    /* ignore prefetch errors */
  } finally {
    prefetchInFlight.delete(href);
  }
}

/**
 * Per-page mount routine registered by whichever runtime bootstrapped
 * (`core.ts` or the legacy `runtime.ts`). Decoupling it here avoids a circular
 * static import while ensuring SPA navigation re-runs the *full* mount pipeline
 * (effects, visible tasks, lazy chunks, portals, stream slots, view transitions)
 * — not just the reactive-text/attr/island subset.
 */
let pageMounter: (() => void) | null = null;

export function setPageMounter(fn: () => void): void {
  pageMounter = fn;
}

function root(): HTMLElement {
  return document.getElementById(ROOT_ID) ?? document.body;
}

/**
 * True when `href` resolves to the current origin. SPA navigation fetches the
 * target and injects its markup into the page, so cross-origin hrefs must never
 * be swapped in — that would render attacker-controlled HTML in our origin.
 */
function isSameOrigin(href: string): boolean {
  try {
    return new URL(href, location.origin).origin === location.origin;
  } catch {
    return false;
  }
}

function pathsMatch(href: string, current: string, exact = false): boolean {
  if (exact) {
    if (href === current) return true;
    const base = "http://resuma.local";
    const a = new URL(href, base);
    const b = new URL(current, base);
    if (a.search) {
      return a.pathname + a.search === b.pathname + b.search;
    }
    return a.pathname === b.pathname;
  }
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

function applyNavLinkActive(
  a: HTMLAnchorElement,
  activeClass: string,
  isActive: boolean,
): void {
  let base = a.getAttribute("data-r-base-class");
  if (!base) {
    base = a.className
      .split(/\s+/)
      .filter((c) => c && c !== activeClass)
      .join(" ");
    if (!base) base = a.className;
    a.setAttribute("data-r-base-class", base);
  }
  a.className = isActive ? `${base} ${activeClass}`.trim() : base;
  if (isActive) a.setAttribute("aria-current", "page");
  else a.removeAttribute("aria-current");
}

function bestNavMatch(
  links: HTMLAnchorElement[],
  path: string,
): HTMLAnchorElement | null {
  let best: HTMLAnchorElement | null = null;
  let bestLen = -1;
  for (const a of links) {
    const href = a.getAttribute("href");
    if (!href) continue;
    const exact = a.hasAttribute("data-r-nav-exact");
    if (!pathsMatch(href, path, exact)) continue;
    if (href.length > bestLen) {
      best = a;
      bestLen = href.length;
    }
  }
  return best;
}

export function updateNavActiveClasses(path: string): void {
  const links = [...document.querySelectorAll<HTMLAnchorElement>("a[data-r-nav]")];
  const exclusiveRoots = new Set<Element>();
  for (const root of document.querySelectorAll("[data-r-nav-exclusive]")) {
    exclusiveRoots.add(root);
  }

  const handled = new Set<HTMLAnchorElement>();
  for (const root of exclusiveRoots) {
    const group = links.filter((a) => root.contains(a));
    const best = bestNavMatch(group, path);
    for (const a of group) {
      const activeClass = a.getAttribute("data-r-active-class");
      if (!activeClass) continue;
      applyNavLinkActive(a, activeClass, a === best);
      handled.add(a);
    }
  }

  for (const a of links) {
    if (handled.has(a)) continue;
    const href = a.getAttribute("href");
    if (!href) continue;
    const activeClass = a.getAttribute("data-r-active-class");
    if (!activeClass) continue;
    const exact = a.hasAttribute("data-r-nav-exact");
    applyNavLinkActive(a, activeClass, pathsMatch(href, path, exact));
  }
}

/** Re-mount signals and bindings after swapping page HTML. */
export async function remountPage(): Promise<void> {
  if (!window.__resuma) {
    window.location.reload();
    return;
  }

  if (pageMounter) {
    pageMounter();
    updateNavActiveClasses(location.pathname + location.search);
    return;
  }

  // Fallback when no mounter registered — replay core mount pipeline.
  try {
    const mod = await import("./core.js");
    mod.mountPage();
    updateNavActiveClasses(location.pathname + location.search);
  } catch {
    window.location.reload();
  }
}

/** Move focus to the page root for assistive tech after an SPA swap. */
function focusMain(): void {
  requestFocus(() => {
    const scope = root();
    return (
      scope.querySelector<HTMLElement>("[autofocus]") ??
      scope.querySelector<HTMLElement>("h1, [role='heading'], main") ??
      scope
    );
  });
}

type FocusTarget = HTMLElement | (() => HTMLElement | null | undefined);
const focusQ: FocusTarget[] = [];
let focusScheduled = false;

function flushFocus(): void {
  focusScheduled = false;
  while (focusQ.length) {
    const t = focusQ.pop()!;
    const el = typeof t === "function" ? t() : t;
    if (!el?.isConnected) continue;
    if (!el.hasAttribute("tabindex") && !el.matches("a,button,input,select,textarea,[tabindex]")) {
      el.setAttribute("tabindex", "-1");
    }
    try {
      el.focus({ preventScroll: true });
    } catch {
      el.focus();
    }
    return;
  }
}

/** Queued focus — last writer in the microtask wins. */
export function requestFocus(target: FocusTarget): void {
  focusQ.push(target);
  if (focusScheduled) return;
  focusScheduled = true;
  queueMicrotask(flushFocus);
}

export function cancelFocus(): void {
  focusQ.length = 0;
  focusScheduled = false;
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

/** Drop Resuma cache-bust `_r` from a path+search string. */
function stripCacheBust(href: string): string {
  try {
    const url = new URL(href, location.origin);
    url.searchParams.delete("_r");
    const q = url.searchParams.toString();
    return url.pathname + (q ? `?${q}` : "");
  } catch {
    return href;
  }
}

function normalizePathSearch(href: string): string {
  try {
    const url = new URL(href, location.origin);
    url.searchParams.delete("_r");
    const q = url.searchParams.toString();
    return url.pathname + (q ? `?${q}` : "");
  } catch {
    return href;
  }
}

function resolveNavigateOptions(
  pushStateOrOpts: boolean | NavigateOptions | undefined,
): Required<Pick<NavigateOptions, "pushState" | "scroll" | "usePrefetch">> &
  Pick<NavigateOptions, "historyHref"> {
  if (typeof pushStateOrOpts === "boolean" || pushStateOrOpts === undefined) {
    const pushState = pushStateOrOpts ?? true;
    return { pushState, scroll: pushState, usePrefetch: true };
  }
  const pushState = pushStateOrOpts.pushState ?? true;
  return {
    pushState,
    scroll: pushStateOrOpts.scroll ?? pushState,
    usePrefetch: pushStateOrOpts.usePrefetch ?? true,
    historyHref: pushStateOrOpts.historyHref,
  };
}

/**
 * Re-run server loaders for a path via SPA navigation (cache-bust query).
 *
 * Soft by default: `replaceState`, no scroll-to-top, and `_r` is stripped from
 * the visible URL so `loader_poll` does not flood history.
 */
export async function invalidate(
  path?: string,
  query?: Record<string, string | null | undefined>,
): Promise<void> {
  prefetchCache.clear();
  const targetPath = path?.split("?")[0] ?? location.pathname;
  const bust = String(Date.now());
  const fetchHref = buildUrl(targetPath, { ...query, _r: bust });
  const historyHref = buildUrl(targetPath, query);
  await navigate(fetchHref, {
    pushState: false,
    scroll: false,
    usePrefetch: false,
    historyHref,
  });
}

// Generation counter + in-flight controller: a slow fetch from a stale
// navigation must never overwrite the DOM after a newer navigation started.
let navGen = 0;
let navController: AbortController | null = null;

type DocWithVt = Document & {
  startViewTransition?: (cb: () => void | Promise<void>) => { finished: Promise<void> };
};

async function swapToHtml(
  href: string,
  html: string,
  opts: Required<Pick<NavigateOptions, "pushState" | "scroll">> &
    Pick<NavigateOptions, "historyHref">,
  gen: number,
): Promise<void> {
  if (gen !== navGen) return;
  const doc = new DOMParser().parseFromString(html, "text/html");
  const newRoot = doc.getElementById(ROOT_ID);
  const newState = doc.getElementById(STATE_SCRIPT_ID);
  if (!newRoot || !newState?.textContent) {
    window.location.href = href;
    return;
  }

  resetClientReady();
  root().innerHTML = newRoot.innerHTML;
  const stateScript = document.getElementById(STATE_SCRIPT_ID);
  if (stateScript) stateScript.textContent = newState.textContent;
  if (doc.title) document.title = doc.title;

  const historyHref = opts.historyHref ?? stripCacheBust(href);
  const dir = doc.documentElement.getAttribute("dir");
  if (dir) document.documentElement.setAttribute("dir", dir);
  else document.documentElement.removeAttribute("dir");

  if (opts.pushState) {
    history.pushState({ resumaNav: true }, "", historyHref);
  } else {
    history.replaceState({ resumaNav: true }, "", historyHref);
  }
  await remountPage();
  await activateClientComponents(root());
  if (opts.scroll) window.scrollTo(0, 0);
  focusMain();
  document.dispatchEvent(
    new CustomEvent("resuma:navigate", { detail: { href: historyHref } }),
  );
}

async function runNavigation(
  href: string,
  opts: Required<Pick<NavigateOptions, "pushState" | "scroll" | "usePrefetch">> &
    Pick<NavigateOptions, "historyHref">,
  gen: number,
  signal: AbortSignal,
): Promise<void> {
  let html: string | undefined = opts.usePrefetch ? prefetchCache.get(href) : undefined;
  if (!html) {
    // Also ignore prefetch for path-only keys when bust query differs.
    const res = await fetch(href, {
      headers: { Accept: "text/html" },
      credentials: "same-origin",
      cache: "no-store",
      signal,
    });
    if (gen !== navGen) return;
    if (!res.ok) {
      window.location.href = href;
      return;
    }
    html = await res.text();
  } else {
    prefetchCache.delete(href);
  }
  if (gen !== navGen) return;
  await swapToHtml(href, html, opts, gen);
}

export async function navigate(
  href: string,
  pushStateOrOpts: boolean | NavigateOptions = true,
): Promise<void> {
  // SPA navigation swaps fetched HTML into our origin — never do that for a
  // cross-origin target. Fall back to a full, browser-mediated navigation.
  if (!isSameOrigin(href)) {
    window.location.assign(href);
    return;
  }
  const opts = resolveNavigateOptions(pushStateOrOpts);
  const gen = ++navGen;
  navController?.abort();
  const controller = new AbortController();
  navController = controller;

  const run = async () => {
    try {
      await runNavigation(href, opts, gen, controller.signal);
    } catch (err) {
      if (err instanceof DOMException && err.name === "AbortError") return;
      console.error("[r] nav", err);
      window.location.href = href;
    }
  };

  const soft = !opts.pushState && !opts.scroll;
  const doc = document as DocWithVt;
  if (!soft && doc.startViewTransition && !hasTopLayer()) {
    try {
      await doc.startViewTransition(run).finished;
    } catch {
      await run();
    }
  } else {
    await run();
  }
}

function hasTopLayer(): boolean {
  try {
    return !!document.querySelector(":popover-open,dialog[open]");
  } catch {
    return false;
  }
}

/**
 * Follow redirect hints from submit/action JSON — uses SPA nav for same-origin paths.
 *
 * Always bypasses NavLink prefetch (mutations must see fresh loader data). Same-path
 * redirects soft-replace without scrolling; cross-path redirects push + scroll.
 */
export function followRedirect(path: string): void {
  if (path === REVALIDATE_SENTINEL) {
    void invalidate();
    return;
  }
  // Same-origin absolute paths use SPA nav. Reject protocol-relative (`//host`)
  // and non-http(s) schemes so a redirect hint can't bounce the user off-origin
  // or into a `javascript:`/`data:` URL.
  const go = (target: string) => {
    prefetchCache.clear();
    const current = normalizePathSearch(location.pathname + location.search);
    const next = normalizePathSearch(target);
    const same = current === next;
    void navigate(target, {
      pushState: !same,
      scroll: !same,
      usePrefetch: false,
      historyHref: next,
    });
  };

  if (path.startsWith("/") && !path.startsWith("//")) {
    go(path);
    return;
  }
  try {
    const url = new URL(path, location.origin);
    if (url.origin === location.origin && (url.protocol === "http:" || url.protocol === "https:")) {
      go(url.pathname + url.search + url.hash);
      return;
    }
  } catch {
    /* fall through to ignore */
  }
  // Anything else (cross-origin, protocol-relative, javascript:, data:) is
  // treated as untrusted: ignore rather than navigate.
  console.warn("[r] bad redirect", path);
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
  document.addEventListener(
    "mouseenter",
    (ev) => {
      const target = ev.target;
      if (!(target instanceof Element)) return;
      const a = target.closest("a[data-r-nav]") as HTMLAnchorElement | null;
      const href = a?.getAttribute("href");
      if (href) void prefetchRoute(href);
    },
    true,
  );

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
    void navigate(location.pathname + location.search, {
      pushState: false,
      scroll: false,
      usePrefetch: false,
    });
  });
}
