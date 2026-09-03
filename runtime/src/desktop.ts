/** Keyboard modality, idle presence, measure, storage, live-region announce. */

const IDLE_MS = 60_000;
let idle = false;
let idleTimer = 0;
let presenceReady = false;

function setIdle(next: boolean): void {
  if (idle === next) return;
  idle = next;
  (window as unknown as { __rIdle?: boolean }).__rIdle = idle;
  document.dispatchEvent(
    new CustomEvent("resuma:presence", { detail: { idle, hidden: document.hidden } }),
  );
}

function bumpIdle(): void {
  setIdle(false);
  window.clearTimeout(idleTimer);
  idleTimer = window.setTimeout(() => setIdle(true), IDLE_MS);
}

export function initPresence(): void {
  if (presenceReady) return;
  presenceReady = true;
  ["pointerdown", "keydown", "scroll", "touchstart"].forEach((ev) => {
    document.addEventListener(ev, bumpIdle, { passive: true });
  });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) setIdle(true);
    else bumpIdle();
  });
  bumpIdle();
}

export function presence(): { idle: boolean; hidden: boolean } {
  return { idle: idle || document.hidden, hidden: document.hidden };
}

let navReady = false;

export function initNavModality(): void {
  if (navReady) return;
  navReady = true;
  document.addEventListener(
    "keydown",
    (e) => {
      if (e.key === "Tab") document.documentElement.dataset.rNav = "keyboard";
    },
    true,
  );
  document.addEventListener(
    "pointerdown",
    () => {
      delete document.documentElement.dataset.rNav;
    },
    true,
  );
}

export function isNavigatingWithKeyboard(): boolean {
  return document.documentElement.getAttribute("data-r-nav") === "keyboard";
}

export function contentSizeMultiplier(): number {
  const px = parseFloat(getComputedStyle(document.documentElement).fontSize);
  return Number.isFinite(px) ? px / 16 : 1;
}

export type LayoutInfo = { x: number; y: number; width: number; height: number };

export function measure(el: Element | string | null | undefined): LayoutInfo | null {
  const node = typeof el === "string" ? document.querySelector(el) : el;
  if (!(node instanceof Element)) return null;
  const r = node.getBoundingClientRect();
  return { x: r.x, y: r.y, width: r.width, height: r.height };
}

export const storage = {
  get(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  set(key: string, value: string): void {
    try {
      localStorage.setItem(key, value);
    } catch {
      /* private mode / SSR */
    }
  },
  remove(key: string): void {
    try {
      localStorage.removeItem(key);
    } catch {
      /* private mode / SSR */
    }
  },
};

export function isOnline(): boolean {
  return typeof navigator === "undefined" ? true : navigator.onLine;
}

export function announce(msg: string): void {
  let node = document.getElementById("r-live");
  if (!node) {
    node = document.createElement("div");
    node.id = "r-live";
    node.setAttribute("role", "status");
    node.setAttribute("aria-live", "polite");
    node.setAttribute("aria-atomic", "true");
    node.style.cssText =
      "position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0";
    document.body.appendChild(node);
  }
  node.textContent = "";
  requestAnimationFrame(() => {
    node!.textContent = msg;
  });
}
