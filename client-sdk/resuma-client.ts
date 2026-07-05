/**
 * Resuma client component SDK — mount contract for TypeScript widgets.
 *
 * Copy this file into your app's `client/` directory (see `site-docs` for a full example).
 */

export type ClientCleanup = () => void;

export type ClientInit = (
  root: HTMLElement,
  props: Record<string, unknown>,
) => ClientCleanup | void;

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

export function mountClientRoots(id: string): ClientCleanup[] {
  const init = registry.get(id);
  if (!init) return [];

  const cleanups: ClientCleanup[] = [];
  const selector = `[data-r-client="${id}"]:not([data-r-client-mounted])`;

  for (const root of document.querySelectorAll<HTMLElement>(selector)) {
    root.dataset.rClientMounted = 'true';
    const cleanup = init(root, readProps(root));
    if (typeof cleanup === 'function') cleanups.push(cleanup);
  }

  return cleanups;
}

/** Register + mount a single client component entry (typical esbuild bundle footer). */
export function bootClientComponent(id: string, init: ClientInit): void {
  defineClientComponent(id, init);

  const run = () => {
    mountClientRoots(id);
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', run, { once: true });
  } else {
    run();
  }

  document.addEventListener('resuma:navigate', () => {
    mountClientRoots(id);
  });
}
