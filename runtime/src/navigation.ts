/**
 * Client-side navigation for `<NavLink data-r-nav>` — fetches SSR HTML and
 * swaps `#resuma-root` + `#resuma-state` without a full document reload.
 */

import "./types.js";

const ROOT_ID = "resuma-root";
const STATE_SCRIPT_ID = "resuma-state";

/** Prefetched SSR HTML keyed by same-origin href (NavLink hover). */
const prefetchCache = new Map<string, string>();
const prefetchInFlight = new Set<string>();

async function prefetchRoute(href: string): Promise<void> {
  if (!href || prefetchInFlight.has(href)) return;
  if (href.startsWith("http") && !href.startsWith(location.origin)) return;
  prefetchInFlight.add(href);
  try {
    const res = await fetch(href, {
      headers: { Accept: "text/html" },
      credentials: "same-origin",
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
    const exact = a.hasAttribute("data-r-nav-exact");
    a.className = pathsMatch(href, path, exact) ? `${base} ${activeClass}`.trim() : base;
  });
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

/** Re-run server loaders for a path via SPA navigation (cache-bust query). */
export async function invalidate(
  path?: string,
  query?: Record<string, string | null | undefined>,
): Promise<void> {
  prefetchCache.clear();
  const targetPath = path?.split("?")[0] ?? location.pathname;
  const bust = String(Date.now());
  await navigate(buildUrl(targetPath, { ...query, _r: bust }));
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
  pushState: boolean,
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

  root().innerHTML = newRoot.innerHTML;
  const stateScript = document.getElementById(STATE_SCRIPT_ID);
  if (stateScript) stateScript.textContent = newState.textContent;
  if (doc.title) document.title = doc.title;

  if (pushState) history.pushState({ resumaNav: true }, "", href);
  await remountPage();
  if (pushState) window.scrollTo(0, 0);
  focusMain();
  document.dispatchEvent(new CustomEvent("resuma:navigate", { detail: { href } }));
}

async function runNavigation(href: string, pushState: boolean, gen: number, signal: AbortSignal): Promise<void> {
  let html: string | undefined = prefetchCache.get(href);
  if (!html) {
    const res = await fetch(href, {
      headers: { Accept: "text/html" },
      credentials: "same-origin",
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
  await swapToHtml(href, html, pushState, gen);
}

export async function navigate(href: string, pushState = true): Promise<void> {
  // SPA navigation swaps fetched HTML into our origin — never do that for a
  // cross-origin target. Fall back to a full, browser-mediated navigation.
  if (!isSameOrigin(href)) {
    window.location.assign(href);
    return;
  }
  const gen = ++navGen;
  navController?.abort();
  const controller = new AbortController();
  navController = controller;

  const run = async () => {
    try {
      await runNavigation(href, pushState, gen, controller.signal);
    } catch (err) {
      if (err instanceof DOMException && err.name === "AbortError") return;
      console.error("[r] nav", err);
      window.location.href = href;
    }
  };

  const doc = document as DocWithVt;
  if (doc.startViewTransition) {
    try {
      await doc.startViewTransition(run).finished;
    } catch {
      await run();
    }
  } else {
    await run();
  }
}

/** Follow redirect hints from submit/action JSON — uses SPA nav for same-origin paths. */
export function followRedirect(path: string): void {
  // Same-origin absolute paths use SPA nav. Reject protocol-relative (`//host`)
  // and non-http(s) schemes so a redirect hint can't bounce the user off-origin
  // or into a `javascript:`/`data:` URL.
  if (path.startsWith("/") && !path.startsWith("//")) {
    void navigate(path);
    return;
  }
  try {
    const url = new URL(path, location.origin);
    if (url.origin === location.origin && (url.protocol === "http:" || url.protocol === "https:")) {
      // Same-origin absolute URLs take the same SPA path as root-relative ones.
      void navigate(url.pathname + url.search + url.hash);
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
    void navigate(location.pathname + location.search, false);
  });
}
