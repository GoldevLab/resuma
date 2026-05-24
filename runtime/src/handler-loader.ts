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

export async function resolveHandler(ref: string, inline: string | null): Promise<Handler> {
  if (inline) {
    let h = inlineCache.get(inline);
    if (!h) {
      h = compileInline(inline);
      inlineCache.set(inline, h);
    }
    return h;
  }

  const [chunk, symbol] = ref.split("#");
  const r = (window as unknown as {
    __resuma: {
      handlers: Record<string, Record<string, string>>;
      loaded: Map<string, Record<string, Function>>;
    };
  }).__resuma;

  if (chunk === "__page__") {
    const src = r.handlers[chunk]?.[symbol];
    if (src) return compileInline(src);
  }

  let mod = r.loaded.get(chunk);
  if (!mod) {
    mod = (await import(`/_resuma/handler/${chunk}.js`)) as Record<string, Function>;
    r.loaded.set(chunk, mod);
  }
  const fn = mod[symbol] as Handler | undefined;
  if (!fn) throw new Error(`[resuma] handler ${symbol} not found in chunk ${chunk}`);
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
