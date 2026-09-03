/** ClientComponent ready — `resuma:client-ready` + `__resumaClientReady[id]`. */

type Cb = () => void;
const ready = new Set<string>();
const waiters = new Map<string, Set<Cb>>();
let on = false;

function bag(): Record<string, unknown> | undefined {
  return (globalThis as { __resumaClientReady?: Record<string, unknown> }).__resumaClientReady;
}

function mark(id: string): void {
  if (!id) return;
  ready.add(id);
  const s = waiters.get(id);
  if (!s) return;
  waiters.delete(id);
  for (const cb of s) cb();
}

function listen(): void {
  if (on || typeof document === "undefined") return;
  on = true;
  document.addEventListener("resuma:client-ready", (e) => {
    const id = (e as CustomEvent<{ id?: string }>).detail?.id;
    if (typeof id === "string") mark(id);
  });
  const b = bag();
  if (b) for (const id of Object.keys(b)) mark(id);
}

export function isClientReady(id: string): boolean {
  return ready.has(id) || !!bag()?.[id];
}

export function onClientReady(id: string, cb: Cb): () => void {
  listen();
  if (isClientReady(id)) {
    cb();
    return () => {};
  }
  let s = waiters.get(id);
  if (!s) waiters.set(id, (s = new Set()));
  s.add(cb);
  return () => {
    s!.delete(cb);
  };
}

export function waitClient(id: string, ms = 15000): Promise<void> {
  listen();
  if (isClientReady(id)) return Promise.resolve();
  return new Promise((ok, fail) => {
    const t = setTimeout(() => {
      off();
      fail(new Error(`waitClient(${id}) timed out after ${ms}ms`));
    }, ms);
    const off = onClientReady(id, () => {
      clearTimeout(t);
      ok();
    });
  });
}

export function notifyClientReady(id: string): void {
  listen();
  const g = globalThis as { __resumaClientReady?: Record<string, number> };
  (g.__resumaClientReady ??= Object.create(null))[id] = 1;
  mark(id);
  document.dispatchEvent(new CustomEvent("resuma:client-ready", { detail: { id } }));
}

/** Drop ready flags before an SPA swap so `waitClient` waits for the remount. */
export function resetClientReady(): void {
  ready.clear();
  const g = globalThis as { __resumaClientReady?: Record<string, unknown> };
  if (g.__resumaClientReady) {
    for (const key of Object.keys(g.__resumaClientReady)) {
      delete g.__resumaClientReady[key];
    }
  }
}

/**
 * After SPA `innerHTML` swap, module scripts do not run. Re-import each
 * ClientComponent bundle so first-visit-via-nav boots; already-evaluated
 * modules remount via `resuma:navigate` + `bootClientComponent`.
 */
export async function activateClientComponents(scope: ParentNode = document): Promise<void> {
  const tasks: Promise<void>[] = [];
  for (const el of scope.querySelectorAll('script[type="module"]')) {
    const text = el.textContent || "";
    const url = /import\s+"([^"]+)"/.exec(text)?.[1];
    if (!url || !url.includes("/static/client/")) continue;
    el.remove();
    tasks.push(
      import(/* webpackIgnore: true */ url).then(
        () => undefined,
        (err) => {
          console.warn("[resuma] ClientComponent import failed", url, err);
        },
      ),
    );
  }
  if (tasks.length) await Promise.all(tasks);
}
