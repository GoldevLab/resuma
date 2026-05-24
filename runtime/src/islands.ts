/**
 * Island bootstrap. Each `<resuma-island>` element on the page contains its
 * own SSR-rendered HTML plus props serialized as JSON. We dynamically import
 * its chunk and call `resume(props, signals, root)` if exported. This mirrors
 * the contract used by the macro layer.
 */

import type { SignalCell } from "./signals.js";

const ISLAND_TAG = "resuma-island";

export function initIslands(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  const islands = root.querySelectorAll<HTMLElement>(ISLAND_TAG);
  islands.forEach((el) => {
    const chunk = el.getAttribute("data-r-chunk");
    if (!chunk) return;
    const propsRaw = el.getAttribute("data-r-props") ?? "{}";
    let props: unknown = {};
    try { props = JSON.parse(propsRaw); } catch { /* keep default */ }
    void hydrateIsland(chunk, props, el, signals);
  });
}

async function hydrateIsland(
  chunk: string,
  props: unknown,
  el: HTMLElement,
  signals: Map<string, SignalCell<unknown>>,
): Promise<void> {
  try {
    const mod: { resume?: (p: unknown, s: Map<string, SignalCell<unknown>>, root: HTMLElement) => void } =
      await import(`/_resuma/island-chunk/${chunk}.js`);
    if (typeof mod.resume === "function") mod.resume(props, signals, el);
  } catch (err) {
    // Islands are optional — the static HTML still works without their JS.
    console.debug("[resuma] island chunk unavailable, staying static", chunk, err);
  }
}
