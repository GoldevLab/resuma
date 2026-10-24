/**
 * Client-side effect replay — computed, debounce, and registered side effects.
 */

import { signalId, type SignalCell, type RawSignalId } from "./signals.js";
import type { LoadHandlerChunk, RegisterMountCleanup } from "./lifecycle.js";
import type { ResumaGlobal } from "./types.js";

export interface ClientEffectSpec {
  id: number;
  deps: RawSignalId[];
  captures?: Record<string, RawSignalId>;
  kind: string;
  symbol: string;
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

export async function initEffects(
  effects: ClientEffectSpec[],
  signals: Map<string, SignalCell<unknown>>,
  global: ResumaGlobal,
  loadModule: LoadHandlerChunk,
  registerCleanup: RegisterMountCleanup,
): Promise<void> {
  if (!effects.length) return;
  let active = true;
  registerCleanup(() => { active = false; });
  let module: Record<string, Function>;
  try {
    module = await loadModule("__effects__");
  } catch (err) {
    if (active) console.error("[r] effect module", err);
    return;
  }
  if (!active) return;

  for (const eff of effects) {
    try {
      const state = buildEffectState(eff.captures, signals, global);
      const run = module[eff.symbol] as (
        state: Record<string, SignalCell<unknown>>,
        resuma: ResumaGlobal,
      ) => unknown;
      if (typeof run !== "function") throw new Error(`effect ${eff.id} missing from module`);

      const isDebounce = eff.kind === "debounce";

      if (isDebounce) {
        const teardown = run(state, global);
        if (typeof teardown === "function") {
          registerCleanup(teardown as () => void);
        }
      } else {
        const targetCell =
          eff.target != null ? signals.get(signalId(eff.target)) : undefined;

        const execute = () => {
          try {
            const result = run(state, global);
            if (targetCell && result !== undefined) targetCell.set(result);
          } catch (err) {
            console.error("[r] effect", eff.id, err);
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

        registerCleanup(() => {
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
          if (unsubscribe) registerCleanup(unsubscribe);
        }
      }
    } catch (err) {
      console.error("[r] effect init", eff.id, err);
    }
  }
}
