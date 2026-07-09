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

type ResumaHandlerState = {
  handlers: Record<string, Record<string, string>>;
  loaded: Map<string, Record<string, Function>>;
};

function resumaHandlers(): ResumaHandlerState {
  return (window as unknown as { __resuma: ResumaHandlerState }).__resuma;
}

async function loadChunkModule(chunk: string): Promise<Record<string, Function>> {
  const r = resumaHandlers();
  let mod = r.loaded.get(chunk);
  if (mod) return mod;

  let pending = inFlight.get(chunk);
  if (!pending) {
    pending = import(`/_resuma/handler/${chunk}.js`) as Promise<
      Record<string, Function>
    >;
    inFlight.set(chunk, pending);
    void pending.finally(() => inFlight.delete(chunk));
  }
  mod = await pending;
  r.loaded.set(chunk, mod);
  return mod;
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

  const mod = await loadChunkModule(chunk);
  const fn = mod[symbol] as Handler | undefined;
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
