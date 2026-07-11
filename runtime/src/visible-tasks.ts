/**
 * Client-side `use_visible_task` runner — deferred until viewport (IO) or eager
 * when IntersectionObserver is unavailable (headless tests, old browsers).
 */

import "./types.js";
import { signalId, type SignalCell, type RawSignalId } from "./signals.js";
import { registerMountCleanup } from "./mount-cleanups.js";

export interface VisibleTaskSpec {
  body: string;
  captures?: Record<string, RawSignalId>;
}

/** Legacy payload entries were plain JS source strings. */
export type VisibleTaskEntry = string | VisibleTaskSpec;

function normalizeVisibleTask(entry: VisibleTaskEntry): VisibleTaskSpec {
  if (typeof entry === "string") return { body: entry };
  return entry;
}

function buildTaskState(
  captures: Record<string, RawSignalId> | undefined,
  signals: Map<string, SignalCell<unknown>>,
  globalState: Record<string, SignalCell<unknown>>,
): Record<string, SignalCell<unknown>> {
  const local: Record<string, SignalCell<unknown>> = Object.create(globalState);
  if (!captures) return local;
  for (const [name, idRaw] of Object.entries(captures)) {
    const cell = signals.get(signalId(idRaw));
    if (cell) local[name] = cell;
  }
  return local;
}

export function runVisibleTasks(
  tasks: Record<string, VisibleTaskEntry>,
  signals: Map<string, SignalCell<unknown>>,
  globalState: Record<string, SignalCell<unknown>>,
  rootEl: () => HTMLElement,
): void {
  const entries = Object.entries(tasks).map(
    ([id, raw]) => [id, normalizeVisibleTask(raw)] as const,
  );
  if (!entries.length) return;

  const run = async (id: string, spec: VisibleTaskSpec) => {
    try {
      let trimmed = spec.body.trim();
      if (trimmed.endsWith(")()")) trimmed = trimmed.slice(0, -2);
      const state = buildTaskState(spec.captures, signals, globalState);
      const fn = new Function(
        "state",
        "__resuma",
        `return (${trimmed})(state, __resuma);`,
      ) as (
        state: Record<string, SignalCell<unknown>>,
        resuma: NonNullable<(typeof window)["__resuma"]>,
      ) => Promise<unknown> | unknown;
      const result = await Promise.resolve(fn(state, window.__resuma!));
      if (typeof result === "function") {
        registerMountCleanup(result as () => void);
      }
    } catch (err) {
      console.error("[r] task", id, err);
    }
  };

  const pending = new Set(entries.map(([id]) => id));
  const runOnce = (id: string, spec: VisibleTaskSpec) => {
    if (!pending.has(id)) return;
    pending.delete(id);
    void run(id, spec);
  };

  if ("IntersectionObserver" in window) {
    const io = new IntersectionObserver(
      (ioEntries, obs) => {
        for (const entry of ioEntries) {
          if (!entry.isIntersecting) continue;
          const id = (entry.target as HTMLElement).dataset.rVisibleTask;
          const spec = id
            ? entries.find(([taskId]) => taskId === id)?.[1]
            : undefined;
          if (id && spec) runOnce(id, spec);
          obs.unobserve(entry.target);
        }
      },
      { rootMargin: "50px" },
    );
    registerMountCleanup(() => io.disconnect());

    rootEl().querySelectorAll("[data-r-visible-task]").forEach((n) => n.remove());
    for (const [id] of entries) {
      const marker = document.createElement("span");
      marker.dataset.rVisibleTask = id;
      marker.className = "r-visible-task-marker";
      marker.setAttribute("aria-hidden", "true");
      rootEl().appendChild(marker);
      io.observe(marker);
    }
  } else {
    for (const [id, spec] of entries) runOnce(id, spec);
  }
}
