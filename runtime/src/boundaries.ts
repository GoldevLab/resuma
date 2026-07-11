/**
 * Prefetch lazy handler chunks when resumable component boundaries enter the viewport.
 */

import { registerMountCleanup } from "./mount-cleanups.js";
import { prefetchHandlerChunk } from "./handler-loader.js";

const MARKER_ATTR = "data-r-lazy-chunk-marker";

export function prefetchLazyChunks(chunks: string[], root: HTMLElement): void {
  root.querySelectorAll(`[${MARKER_ATTR}]`).forEach((n) => n.remove());

  const unique = [...new Set(chunks.filter(Boolean))];
  if (!unique.length) return;

  if (!("IntersectionObserver" in window)) {
    for (const chunk of unique) prefetchHandlerChunk(chunk);
    return;
  }

  const io = new IntersectionObserver(
    (entries, obs) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const chunk = (entry.target as HTMLElement).dataset.rChunk;
        if (chunk) prefetchHandlerChunk(chunk);
        obs.unobserve(entry.target);
      }
    },
    { rootMargin: "120px" },
  );
  registerMountCleanup(() => io.disconnect());

  for (const el of root.querySelectorAll<HTMLElement>("resuma-boundary[data-r-chunk]")) {
    io.observe(el);
  }

  for (const chunk of unique) {
    const marker = document.createElement("resuma-boundary");
    marker.hidden = true;
    marker.dataset.rChunk = chunk;
    marker.setAttribute(MARKER_ATTR, "true");
    root.appendChild(marker);
    io.observe(marker);
  }
}
