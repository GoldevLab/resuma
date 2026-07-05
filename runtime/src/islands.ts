/**
 * Island bootstrap. Each `<resuma-island>` element on the page contains its
 * own SSR-rendered HTML plus props serialized as JSON. We dynamically import
 * its chunk and call `resume(props, signals, root)` if exported.
 */

import type { SignalCell } from "./signals.js";

const ISLAND_TAG = "resuma-island";

export function initIslands(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  const islands = root.querySelectorAll<HTMLElement>(ISLAND_TAG);
  islands.forEach((el) => {
    const chunk = el.getAttribute("data-r-chunk");
    if (!chunk) return;
    const load = el.getAttribute("data-r-load") ?? "eager";
    if (load === "visible" && "IntersectionObserver" in window) {
      const io = new IntersectionObserver(
        (entries, obs) => {
          for (const entry of entries) {
            if (!entry.isIntersecting) continue;
            obs.unobserve(entry.target);
            void mountIsland(el, chunk, signals);
          }
        },
        { rootMargin: "100px" },
      );
      io.observe(el);
      return;
    }
    void mountIsland(el, chunk, signals);
  });
}

async function mountIsland(
  el: HTMLElement,
  chunk: string,
  signals: Map<string, SignalCell<unknown>>,
): Promise<void> {
  const propsRaw = el.getAttribute("data-r-props") ?? "{}";
  let props: unknown = {};
  try {
    props = JSON.parse(propsRaw);
  } catch {
    /* keep default */
  }
  await hydrateIsland(chunk, props, el, signals);
}

async function hydrateIsland(
  chunk: string,
  props: unknown,
  el: HTMLElement,
  signals: Map<string, SignalCell<unknown>>,
): Promise<void> {
  if (el.dataset.rHydrated === "true") return;
  try {
    const mod: {
      resume?: (p: unknown, s: Map<string, SignalCell<unknown>>, root: HTMLElement) => void;
    } = await import(`/_resuma/island-chunk/${chunk}.js`);
    if (typeof mod.resume === "function") {
      mod.resume(props, signals, el);
      el.dataset.rHydrated = "true";
    }
  } catch (err) {
    console.debug("[resuma] island chunk unavailable, staying static", chunk, err);
  }
}

export { hydrateIsland as loadIslandChunk };
