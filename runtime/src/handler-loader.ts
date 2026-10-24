/**
 * Lazy handler / island chunk loader.
 *
 * Handler refs look like `<chunk>#<symbol>`. Island chunks export `resume`.
 * Chunk digests from the SSR payload drive deterministic cache-busting.
 */

export type Handler = (
  event: Event,
  state: Record<string, unknown>,
  resuma: {
    action(name: string, args: unknown[]): Promise<unknown>;
    signals: Map<string, unknown>;
  },
) => unknown | Promise<unknown>;

export type ChunkKind = "handler" | "island";

const inFlight = new Map<string, Promise<Record<string, Function>>>();
/** Bumped on SPA navigation so stale in-flight imports cannot repopulate caches. */
let loadGeneration = 0;

type ResumaChunkState = {
  loaded: Map<string, Record<string, Function>>;
  islandLoaded: Map<string, Record<string, Function>>;
  chunkDigests: Record<string, string>;
};

function resumaChunks(): ResumaChunkState {
  return (window as unknown as { __resuma: ResumaChunkState }).__resuma;
}

function moduleKey(kind: ChunkKind, chunk: string): string {
  return `${kind}:${chunk}`;
}

function chunkUrl(kind: ChunkKind, chunk: string, rev?: string): string {
  const base =
    kind === "handler"
      ? `/_resuma/handler/${encodeURIComponent(chunk)}.js`
      : `/_resuma/island-chunk/${encodeURIComponent(chunk)}.js`;
  if (rev) return `${base}?v=${encodeURIComponent(rev)}`;
  return base;
}

function chunkRevision(chunk: string, bust: boolean): string | undefined {
  if (bust) {
    const digest = resumaChunks().chunkDigests?.[chunk];
    return digest || String(Date.now());
  }
  return resumaChunks().chunkDigests?.[chunk];
}

/** Drop in-memory and in-flight imports so merged server chunks can be refetched. */
export function invalidateHandlerChunks(
  chunks: Iterable<string>,
  loaded?: Map<string, Record<string, Function>>,
): void {
  invalidateLazyChunks(
    [...chunks].map((chunk) => ({ kind: "handler" as const, chunk })),
    loaded,
  );
}

export function invalidateLazyChunks(
  entries: Iterable<{ kind: ChunkKind; chunk: string }>,
  handlerLoaded?: Map<string, Record<string, Function>>,
  islandLoaded?: Map<string, Record<string, Function>>,
): void {
  loadGeneration++;
  for (const { kind, chunk } of entries) {
    if (!chunk) continue;
    const map = kind === "handler" ? handlerLoaded : islandLoaded;
    map?.delete(chunk);
    const key = moduleKey(kind, chunk);
    inFlight.delete(key);
    inFlight.delete(`${key}:bust`);
  }
}

async function loadChunkModule(
  kind: ChunkKind,
  chunk: string,
  bust = false,
): Promise<Record<string, Function>> {
  const r = resumaChunks();
  const map = kind === "handler" ? r.loaded : r.islandLoaded;
  const flightKey = bust ? `${moduleKey(kind, chunk)}:bust` : moduleKey(kind, chunk);
  const generation = loadGeneration;
  const rev = chunkRevision(chunk, bust);

  if (!bust) {
    const cached = map.get(chunk);
    if (cached) return cached;
  }

  let pending = inFlight.get(flightKey);
  if (!pending) {
    pending = import(/* @vite-ignore */ chunkUrl(kind, chunk, rev)) as Promise<
      Record<string, Function>
    >;
    inFlight.set(flightKey, pending);
    const request = pending;
    const clearRequest = () => {
      if (inFlight.get(flightKey) === request) inFlight.delete(flightKey);
    };
    void request.then(clearRequest, clearRequest);
  }
  const mod = await pending;
  if (generation !== loadGeneration) {
    return loadChunkModule(kind, chunk, true);
  }
  if (!bust) {
    const cached = map.get(chunk);
    if (cached) return cached;
  }
  map.set(chunk, mod);
  return mod;
}

/** Prefetch a handler chunk when a boundary enters the viewport. */
export function prefetchHandlerChunk(chunk: string): void {
  const r = resumaChunks();
  if (r.loaded.has(chunk)) return;
  void loadChunkModule("handler", chunk).catch(() => {
    /* chunk may load on first interaction instead */
  });
}

export async function loadIslandModule(chunk: string): Promise<Record<string, Function>> {
  return loadChunkModule("island", chunk);
}

export async function loadHandlerChunk(chunk: string): Promise<Record<string, Function>> {
  return loadChunkModule("handler", chunk);
}

export async function resolveHandler(ref: string, _inline: string | null): Promise<Handler> {
  const hash = ref.indexOf("#");
  const chunk = hash === -1 ? ref : ref.slice(0, hash);
  const symbol = hash === -1 ? ref : ref.slice(hash + 1);

  let mod = await loadChunkModule("handler", chunk);
  let fn = mod[symbol] as Handler | undefined;
  if (!fn) {
    invalidateHandlerChunks([chunk]);
    mod = await loadChunkModule("handler", chunk, true);
    fn = mod[symbol] as Handler | undefined;
  }
  if (typeof fn !== "function") throw new Error(`handler ${symbol} missing in ${chunk}`);
  return fn;
}
