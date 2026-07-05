/**
 * Client-side effect replay — computed, debounce, and registered side effects.
 */

import { signalId, type SignalCell, type RawSignalId } from "./signals.js";
import type { ResumaGlobal } from "./core.js";

export interface ClientEffectSpec {
  id: number;
  deps: RawSignalId[];
  captures?: Record<string, RawSignalId>;
  kind: string;
  body: string;
  target?: RawSignalId;
  debounce_ms?: number;
}

function buildEffectState(
  captures: Record<string, RawSignalId> | undefined,
  signals: Map<string, SignalCell<unknown>>,
  global: ResumaGlobal,
): Record<string, SignalCell<unknown>> {
  const local: Record<string, SignalCell<unknown>> = Object.create(global.state);
  if (!captures) return local;
  for (const [name, idRaw] of Object.entries(captures)) {
    const cell = signals.get(signalId(idRaw));
    if (cell) local[name] = cell;
  }
  return local;
}

// Per-mount cleanups (signal unsubscribes + pending debounce timers). SPA
// navigation re-runs initEffects against fresh signal cells, so the previous
// mount's subscriptions and timers must be torn down or they leak and fire
// against orphaned cells (mirrors the `flowCleanups` pattern in flow.ts).
const effectCleanups: Array<() => void> = [];

export function flushEffectCleanups(): void {
  while (effectCleanups.length) {
    const fn = effectCleanups.pop();
    try {
      fn?.();
    } catch {
      /* ignore cleanup errors */
    }
  }
}

export function initEffects(
  effects: ClientEffectSpec[],
  signals: Map<string, SignalCell<unknown>>,
  global: ResumaGlobal,
): void {
  for (const eff of effects) {
    try {
      const state = buildEffectState(eff.captures, signals, global);
      // rs2js emits the body as an arrow expression `(state, __resuma) => { … }`.
      // Build it once, then invoke on each dependency change.
      const run = new Function(`return (${eff.body});`)() as (
        state: Record<string, SignalCell<unknown>>,
        resuma: ResumaGlobal,
      ) => unknown;

      const targetCell =
        eff.target != null ? signals.get(signalId(eff.target)) ?? null : null;

      const execute = () => {
        try {
          const result = run(state, global);
          // `computed!` returns a derived value bound to a target signal;
          // `effect!` mutates signals itself and returns undefined.
          if (targetCell && result !== undefined) targetCell.set(result);
        } catch (err) {
          console.error("[resuma] effect", eff.id, err);
        }
      };

      let debounceTimer: ReturnType<typeof setTimeout> | undefined;
      const schedule = () => {
        const ms = eff.debounce_ms;
        if (ms != null && ms > 0) {
          if (debounceTimer !== undefined) clearTimeout(debounceTimer);
          debounceTimer = setTimeout(execute, ms);
          return;
        }
        execute();
      };

      schedule();

      effectCleanups.push(() => {
        if (debounceTimer !== undefined) clearTimeout(debounceTimer);
      });

      const depKeys = new Set<string>();
      for (const dep of eff.deps) {
        depKeys.add(signalId(dep));
      }
      if (eff.captures) {
        for (const idRaw of Object.values(eff.captures)) {
          depKeys.add(signalId(idRaw));
        }
      }

      for (const depKey of depKeys) {
        const cell = signals.get(depKey);
        const unsubscribe = cell?.subscribe(() => schedule());
        if (unsubscribe) effectCleanups.push(unsubscribe);
      }
    } catch (err) {
      console.error("[resuma] effect init", eff.id, err);
    }
  }
}
