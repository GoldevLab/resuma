/**
 * Client-side `use_visible_task` runner — deferred until viewport (IO) or eager
 * when IntersectionObserver is unavailable (headless tests, old browsers).
 */

import "./types.js";
import { signalId, type SignalCell, type RawSignalId } from "./signals.js";
import type { LoadHandlerChunk, RegisterMountCleanup } from "./lifecycle.js";
import type { ResumaGlobal } from "./types.js";

export interface VisibleTaskSpec {
  symbol: string;
  boundary_id?: number;
  captures?: Record<string, RawSignalId>;
}

export type VisibleTaskEntry = VisibleTaskSpec;

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
  global: ResumaGlobal,
  loadModule: LoadHandlerChunk,
  registerCleanup: RegisterMountCleanup,
): void {
  const specsById = new Map(Object.entries(tasks));
  if (specsById.size === 0) return;
  const taskRoot = rootEl();
  let active = true;
  let io: IntersectionObserver | undefined;
  const markers: HTMLElement[] = [];
  registerCleanup(() => {
    active = false;
    io?.disconnect();
    for (const marker of markers) marker.remove();
  });
  let lifecycleModule: Promise<Record<string, Function>> | undefined;
  const loadLifecycle = () => (lifecycleModule ??= loadModule("__visible_tasks__"));

  const run = async (id: string, spec: VisibleTaskSpec) => {
    try {
      const state = buildTaskState(spec.captures, signals, globalState);
      const module = await loadLifecycle();
      const fn = module[spec.symbol] as (
        state: Record<string, SignalCell<unknown>>,
        resuma: NonNullable<(typeof window)["__resuma"]>,
      ) => Promise<unknown> | unknown;
      if (typeof fn !== "function") throw new Error(`visible task ${id} missing from module`);
      if (!active) return;
      const result = await Promise.resolve(fn(state, global));
      if (typeof result === "function") {
        if (active) registerCleanup(result as () => void);
        else (result as () => void)();
      }
    } catch (err) {
      console.error("[r] task", id, err);
    }
  };

  const pending = new Set(specsById.keys());
  const runOnce = (id: string, spec: VisibleTaskSpec) => {
    if (!pending.has(id)) return;
    pending.delete(id);
    void run(id, spec);
  };

  if ("IntersectionObserver" in window) {
    const taskEntriesByTarget = new Map<Element, Array<[string, VisibleTaskSpec]>>();
    const anchorsByBoundary = new Map<number, HTMLElement>();
    for (const boundary of taskRoot.querySelectorAll<HTMLElement>("[data-r-boundary-id]")) {
      const boundaryId = Number(boundary.dataset.rBoundaryId);
      const anchor = boundary.querySelector<HTMLElement>(":scope > [data-r-boundary-anchor]");
      if (Number.isSafeInteger(boundaryId) && anchor) anchorsByBoundary.set(boundaryId, anchor);
    }

    for (const [id, spec] of specsById) {
      let target = spec.boundary_id == null ? undefined : anchorsByBoundary.get(spec.boundary_id);
      if (!target) {
        const marker = document.createElement("span");
        marker.dataset.rVisibleTask = id;
        marker.className = "r-visible-task-marker";
        marker.setAttribute("aria-hidden", "true");
        taskRoot.appendChild(marker);
        markers.push(marker);
        target = marker;
      }
      const entries = taskEntriesByTarget.get(target) ?? [];
      entries.push([id, spec]);
      taskEntriesByTarget.set(target, entries);
    }

    io = new IntersectionObserver(
      (ioEntries, obs) => {
        for (const entry of ioEntries) {
          if (!entry.isIntersecting) continue;
          for (const [id, spec] of taskEntriesByTarget.get(entry.target) ?? []) {
            runOnce(id, spec);
          }
          obs.unobserve(entry.target);
        }
      },
      { rootMargin: "50px" },
    );
    for (const target of taskEntriesByTarget.keys()) io.observe(target);
  } else {
    for (const [id, spec] of specsById) runOnce(id, spec);
  }
}
