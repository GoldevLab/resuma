/**
 * Resuma client component SDK — mount contract for TypeScript widgets.
 *
 * Copy this file into your app's `client/` directory (see `site-docs` for a full example).
 */

export type ClientCleanup = () => void;

export type ClientInit = (
  root: HTMLElement,
  props: Record<string, unknown>,
) => ClientCleanup | void | Promise<ClientCleanup | void>;

const registry = new Map<string, ClientInit>();

export function defineClientComponent(id: string, init: ClientInit): void {
  registry.set(id, init);
}

export function prefersReducedMotion(): boolean {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

function readProps(root: HTMLElement): Record<string, unknown> {
  const raw = root.dataset.rClientProps;
  if (!raw) return {};
  try {
    const parsed = JSON.parse(raw);
    return typeof parsed === 'object' && parsed !== null
      ? (parsed as Record<string, unknown>)
      : {};
  } catch {
    return {};
  }
}

export async function mountClientRoots(id: string): Promise<ClientCleanup[]> {
  const init = registry.get(id);
  if (!init) return [];

  const cleanups: ClientCleanup[] = [];
  const selector = `[data-r-client="${id}"]:not([data-r-client-mounted])`;

  for (const root of document.querySelectorAll<HTMLElement>(selector)) {
    root.dataset.rClientMounted = 'true';
    const cleanup = await init(root, readProps(root));
    if (typeof cleanup === 'function') cleanups.push(cleanup);
  }

  return cleanups;
}

/** Register + mount a single client component entry (typical esbuild bundle footer). */
export function bootClientComponent(id: string, init: ClientInit): void {
  defineClientComponent(id, init);

  const run = () => {
    void mountClientRoots(id).then(() => notifyClientReady(id));
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', run, { once: true });
  } else {
    run();
  }

  document.addEventListener('resuma:navigate', run);
}

const readyIds = new Set<string>();
const waiters = new Map<string, Set<() => void>>();
let listening = false;

function markReady(id: string): void {
  if (!id) return;
  readyIds.add(id);
  const set = waiters.get(id);
  if (!set) return;
  waiters.delete(id);
  for (const cb of set) cb();
}

function ensureListener(): void {
  if (listening || typeof document === 'undefined') return;
  listening = true;
  document.addEventListener('resuma:client-ready', (ev) => {
    const id = (ev as CustomEvent<{ id?: string }>).detail?.id;
    if (typeof id === 'string') markReady(id);
  });
  const bag = (globalThis as { __resumaClientReady?: Record<string, unknown> })
    .__resumaClientReady;
  if (bag && typeof bag === 'object') {
    for (const key of Object.keys(bag)) markReady(key);
  }
}

export function onClientReady(id: string, cb: () => void): () => void {
  ensureListener();
  const bag = (globalThis as { __resumaClientReady?: Record<string, unknown> })
    .__resumaClientReady;
  if (readyIds.has(id) || bag?.[id]) {
    cb();
    return () => {};
  }
  let set = waiters.get(id);
  if (!set) {
    set = new Set();
    waiters.set(id, set);
  }
  set.add(cb);
  return () => {
    set!.delete(cb);
  };
}

export function waitClient(id: string, timeoutMs = 15000): Promise<void> {
  ensureListener();
  const bag = (globalThis as { __resumaClientReady?: Record<string, unknown> })
    .__resumaClientReady;
  if (readyIds.has(id) || bag?.[id]) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => {
      unsub();
      reject(new Error(`waitClient(${id}) timed out after ${timeoutMs}ms`));
    }, timeoutMs);
    const unsub = onClientReady(id, () => {
      clearTimeout(t);
      resolve();
    });
  });
}

export function notifyClientReady(id: string): void {
  ensureListener();
  const g = globalThis as { __resumaClientReady?: Record<string, number> };
  g.__resumaClientReady = g.__resumaClientReady || Object.create(null);
  g.__resumaClientReady![id] = 1;
  markReady(id);
  document.dispatchEvent(
    new CustomEvent('resuma:client-ready', { detail: { id } }),
  );
}
