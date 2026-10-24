/**
 * Prefetch lazy handler chunks when resumable component boundaries enter the viewport.
 */

import { registerMountCleanup } from "./mount-cleanups.js";
import { prefetchHandlerChunk } from "./handler-loader.js";

const HANDLER_EVENTS = [
  "click", "input", "change", "submit", "focus", "blur", "keydown", "keyup", "keypress",
  "mousedown", "mouseup", "mousemove", "mouseenter", "mouseleave", "pointerdown",
  "pointerup", "pointermove", "touchstart", "touchend", "scroll", "wheel", "dragstart",
  "dragend", "drop", "load",
];
const HANDLER_SELECTOR = HANDLER_EVENTS.map((event) => `[data-r-on\\:${event}]`).join(", ");

export function prefetchLazyChunks(chunks: string[], root: HTMLElement): void {
  const requested = new Set(chunks);
  const chunksByElement = new Map<HTMLElement, Set<string>>();
  for (const el of root.querySelectorAll<HTMLElement>(HANDLER_SELECTOR)) {
    for (const attr of el.attributes) {
      if (!attr.name.startsWith("data-r-on:")) continue;
      const hash = attr.value.indexOf("#");
      if (hash < 0) continue;
      const chunk = attr.value.slice(0, hash);
      if (!requested.has(chunk)) continue;
      let elementChunks = chunksByElement.get(el);
      if (!elementChunks) chunksByElement.set(el, (elementChunks = new Set()));
      elementChunks.add(chunk);
    }
  }
  if (!chunksByElement.size) return;

  if (!("IntersectionObserver" in window)) {
    for (const elementChunks of chunksByElement.values()) {
      for (const chunk of elementChunks) prefetchHandlerChunk(chunk);
    }
    return;
  }

  const io = new IntersectionObserver(
    (entries, obs) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const el = entry.target as HTMLElement;
        for (const chunk of chunksByElement.get(el) ?? []) prefetchHandlerChunk(chunk);
        obs.unobserve(entry.target);
      }
    },
    { rootMargin: "120px" },
  );
  registerMountCleanup(() => io.disconnect());

  for (const el of chunksByElement.keys()) io.observe(el);
}
