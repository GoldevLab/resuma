/**
 * Shared client global types — kept in a leaf module to avoid circular imports.
 */

import type { SignalCell } from "./signals.js";

export interface ResumaGlobal {
  state: Record<string, SignalCell<unknown>>;
  signals: Map<string, SignalCell<unknown>>;
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
  navigate: (
    href: string,
    pushStateOrOpts?:
      | boolean
      | {
          pushState?: boolean;
          scroll?: boolean;
          usePrefetch?: boolean;
          historyHref?: string;
        },
  ) => Promise<void>;
  buildUrl: (path: string, query?: Record<string, string | null | undefined>) => string;
  invalidate: (path?: string, query?: Record<string, string | null | undefined>) => Promise<void>;
  /** Resolve when ClientComponent `id` finished init (`bootClientComponent` / `notifyClientReady`). */
  waitClient: (id: string, timeoutMs?: number) => Promise<void>;
  onClientReady: (id: string, cb: () => void) => () => void;
  /** Polite live region (`#r-live`). */
  announce: (msg: string) => void;
  measure: (el: Element | string | null | undefined) => {
    x: number;
    y: number;
    width: number;
    height: number;
  } | null;
  storage: {
    get: (key: string) => string | null;
    set: (key: string, value: string) => void;
    remove: (key: string) => void;
  };
  online: () => boolean;
  presence: () => { idle: boolean; hidden: boolean };
  isNavigatingWithKeyboard: () => boolean;
  contentSizeMultiplier: () => number;
  /** Queued focus — last writer in the frame wins (SPA / modal / popup). */
  focus: (el: HTMLElement | (() => HTMLElement | null | undefined)) => void;
  /** Register a teardown hook flushed on the next `mountPage()`. */
  onCleanup: (fn: () => void) => void;
  showPopup: (id: string) => boolean | Promise<boolean>;
  hidePopup: (id: string) => void;
  showModal: (id: string) => boolean | Promise<boolean>;
  hideModal: (id: string) => void;
  dismissAll: () => void;
  runHandler?: (
    ref: string,
    inline: string | null,
    ev: Event,
    captures: string[],
    handlerEl?: EventTarget | null,
  ) => Promise<void>;
}

declare global {
  interface Window {
    __resuma?: ResumaGlobal;
    __resumaCoreReady?: Promise<void>;
    __resumaClientReady?: Record<string, unknown>;
  }
}

export {};
