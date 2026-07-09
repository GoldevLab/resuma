/**
 * Teardown hooks for per-page mount resources (IntersectionObservers, global
 * listeners registered by visible tasks, etc.). Flushed at the start of each
 * `mountPage()` before bindings re-initialize.
 */

const cleanups: Array<() => void> = [];

export function registerMountCleanup(fn: () => void): void {
  cleanups.push(fn);
}

export function flushMountCleanups(): void {
  while (cleanups.length) {
    const fn = cleanups.pop();
    try {
      fn?.();
    } catch {
      /* ignore cleanup errors */
    }
  }
}
