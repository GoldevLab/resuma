/**
 * Lazy handler loader.
 *
 * Each handler reference looks like `<chunk>#<symbol>`. If the chunk is
 * `__page__` we resolve the symbol against the inline `handlers` map embedded
 * in the resumability payload. Otherwise we dynamically import
 * `/_resuma/handler/<chunk>.js` and grab the named export.
 */

export type Handler = (
  event: Event,
  state: Record<string, unknown>,
  resuma: {
    action(name: string, args: unknown[]): Promise<unknown>;
    signals: Map<string, unknown>;
  },
) => unknown | Promise<unknown>;

const inlineCache = new Map<string, Handler>();
const inFlight = new Map<string, Promise<Record<string, Function>>>();
/** Bumped on SPA navigation so stale in-flight imports cannot repopulate `loaded`. */
let loadGeneration = 0;

type ResumaHandlerState = {
  handlers: Record<string, Record<string, string>>;
  loaded: Map<string, Record<string, Function>>;
};

function resumaHandlers(): ResumaHandlerState {
  return (window as unknown as { __resuma: ResumaHandlerState }).__resuma;
}

function chunkUrl(chunk: string, bust: boolean): string {
  const base = `/_resuma/handler/${encodeURIComponent(chunk)}.js`;
  return bust ? `${base}?_=${Date.now()}` : base;
}

/** Drop in-memory and in-flight imports so merged server chunks can be refetched. */
export function invalidateHandlerChunks(
  chunks: Iterable<string>,
  loaded?: Map<string, Record<string, Function>>,
): void {
  loadGeneration++;
  const map = loaded ?? window.__resuma?.loaded;
  for (const chunk of chunks) {
    if (!chunk) continue;
    map?.delete(chunk);
    inFlight.delete(chunk);
    inFlight.delete(`${chunk}:bust`);
  }
}

async function loadChunkModule(chunk: string, bust = false): Promise<Record<string, Function>> {
  const r = resumaHandlers();
  const flightKey = bust ? `${chunk}:bust` : chunk;
  const generation = loadGeneration;

  if (!bust) {
    const cached = r.loaded.get(chunk);
    if (cached) return cached;
  }

  let pending = inFlight.get(flightKey);
  if (!pending) {
    pending = import(/* @vite-ignore */ chunkUrl(chunk, bust)) as Promise<
      Record<string, Function>
    >;
    inFlight.set(flightKey, pending);
    void pending.finally(() => inFlight.delete(flightKey));
  }
  const mod = await pending;
  if (generation !== loadGeneration) {
    return loadChunkModule(chunk, true);
  }
  if (!bust) {
    const cached = r.loaded.get(chunk);
    if (cached) return cached;
  }
  r.loaded.set(chunk, mod);
  return mod;
}

/** Prefetch a handler chunk when a boundary enters the viewport. */
export function prefetchHandlerChunk(chunk: string): void {
  const r = resumaHandlers();
  if (r.loaded.has(chunk)) return;
  void loadChunkModule(chunk).catch(() => {
    /* chunk may load on first interaction instead */
  });
}

/** Cache-bust handler chunks after SPA navigation (merged symbols on the server). */
export function warmHandlerChunks(chunks: Iterable<string>): void {
  for (const chunk of chunks) {
    if (!chunk) continue;
    void loadChunkModule(chunk, true).catch(() => {
      /* first click will retry */
    });
  }
}

export async function resolveHandler(ref: string, inline: string | null): Promise<Handler> {
  if (inline) {
    let h = inlineCache.get(inline);
    if (!h) {
      h = compileInline(inline);
      inlineCache.set(inline, h);
    }
    return h;
  }

  const hash = ref.indexOf("#");
  const chunk = hash === -1 ? ref : ref.slice(0, hash);
  const symbol = hash === -1 ? ref : ref.slice(hash + 1);
  const r = resumaHandlers();

  if (chunk === "__page__") {
    const src = r.handlers[chunk]?.[symbol];
    if (src) return compileInline(src);
  }

  let mod = await loadChunkModule(chunk);
  let fn = mod[symbol] as Handler | undefined;
  if (!fn) {
    // Server merges new symbols into existing chunk URLs across SSR requests;
    // bust the browser ES module cache once and retry.
    invalidateHandlerChunks([chunk]);
    mod = await loadChunkModule(chunk, true);
    fn = mod[symbol] as Handler | undefined;
  }
  if (!fn) throw new Error(`handler ${symbol} missing in ${chunk}`);
  return fn;
}

function compileInline(src: string): Handler {
  const trimmed = src.trim();
  const looksLikeFn =
    trimmed.startsWith("(") || trimmed.startsWith("function") || trimmed.startsWith("async");
  const body = looksLikeFn
    ? `return (${src});`
    : `return (async (event, state, __resuma) => { ${src} });`;

  const factory = new Function(body);
  const fn = factory();
  return fn as Handler;
}
