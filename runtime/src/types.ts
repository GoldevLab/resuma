/**
 * Shared client global types — kept in a leaf module to avoid circular imports.
 */

import type { SignalCell } from "./signals.js";

export interface ResumaGlobal {
  state: Record<string, SignalCell<unknown>>;
  signals: Map<string, SignalCell<unknown>>;
  handlers: Record<string, Record<string, string>>;
  contexts: Record<string, unknown>;
  action: (name: string, args: unknown[]) => Promise<unknown>;
  safeAction: (
    name: string,
    args: unknown[],
  ) => Promise<
    | { ok: true; value: unknown }
    | { ok: false; error: string; field_errors?: Record<string, string> }
  >;
  loaded: Map<string, Record<string, Function>>;
  islandLoaded: Map<string, Record<string, Function>>;
  chunkDigests: Record<string, string>;
  refreshIsland: (id: string) => Promise<void>;
  context: (key: string) => unknown;
  navigate: (href: string, pushState?: boolean) => Promise<void>;
  buildUrl: (path: string, query?: Record<string, string | null | undefined>) => string;
  invalidate: (path?: string, query?: Record<string, string | null | undefined>) => Promise<void>;
}

declare global {
  interface Window {
    __resuma?: ResumaGlobal;
    __resumaCoreReady?: Promise<void>;
  }
}

export {};
