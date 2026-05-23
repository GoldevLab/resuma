/* Resuma client runtime — fallback bundle.
 *
 * This file is replaced by the optimized bundle produced from
 * `runtime/src/runtime.ts` via `pnpm --filter @resuma/runtime build`.
 * The implementation below mirrors the TypeScript source so apps work
 * out of the box even before the build pipeline has been run.
 */
(() => {
  const STATE_SCRIPT_ID = "resuma-state";
  const ROOT_ID = "resuma-root";
  const HANDLER_PREFIX = "data-r-on:";
  const CAPTURES_PREFIX = "data-r-cap:";
  const INLINE_PREFIX = "data-r-inline:";
  const TEXT_TAG = "resuma-dyn";
  const ISLAND_TAG = "resuma-island";

  const KNOWN_EVENTS = [
    "click", "input", "change", "submit", "focus", "blur", "keydown",
    "keyup", "keypress", "mousedown", "mouseup", "mousemove", "mouseenter",
    "mouseleave", "pointerdown", "pointerup", "pointermove", "touchstart",
    "touchend", "scroll", "wheel", "dragstart", "dragend", "drop",
  ];

  const root = () => document.getElementById(ROOT_ID) || document.body;

  function readPayload() {
    const node = document.getElementById(STATE_SCRIPT_ID);
    if (!node || !node.textContent) return { signals: [], handlers: {}, islands: [], actions: [] };
    try { return JSON.parse(node.textContent); }
    catch (e) { console.error("[resuma] bad payload", e); return { signals: [], handlers: {}, islands: [], actions: [] }; }
  }

  function makeCell(id, initial) {
    let value = initial;
    const subs = new Set();
    const cell = {
      id,
      get value() { return value; },
      set value(v) { cell.set(v); },
      set(v) { if (Object.is(v, value)) return; value = v; subs.forEach((s) => s(value)); },
      update(fn) { const next = fn(value); if (next !== undefined) cell.set(next); else subs.forEach((s) => s(value)); },
      subscribe(fn) { subs.add(fn); return () => subs.delete(fn); },
    };
    return cell;
  }

  function initSignals(raws) {
    const map = new Map();
    for (const r of raws) {
      const id = typeof r.id === "string" ? r.id : (r.id && r.id[0] !== undefined ? `s${r.id[0]}` : `s${r.id}`);
      map.set(id, makeCell(id, r.value));
    }
    return map;
  }

  function formatValue(v) {
    if (v == null) return "";
    if (typeof v === "string") return v;
    if (typeof v === "number" || typeof v === "boolean") return String(v);
    try { return JSON.stringify(v); } catch { return String(v); }
  }

  function bindReactiveText(rootEl, signals) {
    rootEl.querySelectorAll(TEXT_TAG).forEach((node) => {
      const sigId = node.getAttribute("data-r-signal");
      const cell = sigId && signals.get(sigId);
      if (!cell) return;
      cell.subscribe((v) => { node.textContent = formatValue(v); });
    });
  }

  function bindReactiveAttrs(rootEl, signals) {
    const walker = document.createTreeWalker(rootEl, NodeFilter.SHOW_ELEMENT);
    let node = walker.currentNode;
    while (node) {
      if (node instanceof HTMLElement) {
        for (const attr of Array.from(node.attributes)) {
          if (!attr.name.startsWith("data-r-bind:")) continue;
          const target = attr.name.slice("data-r-bind:".length);
          const [sigId, fmt = "{}"] = attr.value.split("|");
          const cell = signals.get(sigId);
          if (!cell) continue;
          const apply = (v) => node.setAttribute(target, fmt.replace("{}", formatValue(v)));
          apply(cell.value);
          cell.subscribe(apply);
        }
      }
      node = walker.nextNode();
    }
  }

  const inlineCache = new Map();
  function compileInline(src) {
    const trimmed = src.trim();
    const looksLikeFn = trimmed.startsWith("(") || trimmed.startsWith("function") || trimmed.startsWith("async");
    const body = looksLikeFn ? `return (${src});` : `return (async (event, state, __resuma) => { ${src} });`;
    return new Function(body)();
  }

  async function resolveHandler(ref, inline) {
    if (inline) {
      let h = inlineCache.get(inline);
      if (!h) { h = compileInline(inline); inlineCache.set(inline, h); }
      return h;
    }
    const [chunk, symbol] = ref.split("#");
    const r = window.__resuma;
    if (chunk === "__page__") {
      const src = r.handlers[chunk] && r.handlers[chunk][symbol];
      if (!src) throw new Error("[resuma] missing inline handler " + ref);
      return compileInline(src);
    }
    let mod = r.loaded.get(chunk);
    if (!mod) { mod = await import("/_resuma/handler/" + chunk + ".js"); r.loaded.set(chunk, mod); }
    const fn = mod[symbol];
    if (!fn) throw new Error("[resuma] missing handler " + symbol);
    return fn;
  }

  async function callServerAction(name, args) {
    const res = await fetch("/_resuma/action/" + encodeURIComponent(name), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ args }),
    });
    if (!res.ok) throw new Error("[resuma] action " + name + " failed: " + res.status);
    const data = await res.json();
    if (data.ok === false) throw new Error(data.error || "action failed");
    return data.value;
  }

  function buildLocalState(capturesAttr) {
    // capturesAttr is comma-separated "name:id" pairs.
    const r = window.__resuma;
    if (!capturesAttr.length) return r.state;
    const local = Object.create(r.state);
    for (const pair of capturesAttr) {
      const [name, id] = pair.split(":");
      const cell = id ? r.signals.get(id) : r.signals.get(name);
      if (cell) local[name] = cell;
    }
    return local;
  }

  async function dispatchEvent(ev) {
    let target = ev.target;
    if (!(target instanceof Element)) return;
    const attr = HANDLER_PREFIX + ev.type;
    const capAttr = CAPTURES_PREFIX + ev.type;
    const inlineAttr = INLINE_PREFIX + ev.type;

    while (target && target !== document.body) {
      const ref = target.getAttribute && target.getAttribute(attr);
      if (ref) {
        const captures = (target.getAttribute(capAttr) || "")
          .split(",").map((s) => s.trim()).filter(Boolean);
        const inline = target.getAttribute(inlineAttr);
        try {
          const fn = await resolveHandler(ref, inline);
          const state = buildLocalState(captures);
          const r = window.__resuma;
          await fn(ev, state, r);
        } catch (err) { console.error("[resuma] handler error", err); }
        return;
      }
      target = target.parentElement;
    }
  }

  function attachEventDelegation() {
    for (const ev of KNOWN_EVENTS) document.addEventListener(ev, dispatchEvent, true);
  }

  async function hydrateIsland(chunk, props, el, signals) {
    try {
      const mod = await import("/_resuma/island/" + chunk + ".js");
      if (typeof mod.resume === "function") mod.resume(props, signals, el);
    } catch (err) { console.debug("[resuma] island static-only", chunk); }
  }

  function initIslands(rootEl, signals) {
    rootEl.querySelectorAll(ISLAND_TAG).forEach((el) => {
      const chunk = el.getAttribute("data-r-chunk");
      if (!chunk) return;
      let props = {};
      try { props = JSON.parse(el.getAttribute("data-r-props") || "{}"); } catch (_) {}
      hydrateIsland(chunk, props, el, signals);
    });
  }

  function bootstrap() {
    const payload = readPayload();
    const signals = initSignals(payload.signals);
    const state = {};
    for (const [k, cell] of signals) state[k] = cell;
    window.__resuma = {
      state, signals, handlers: payload.handlers, loaded: new Map(),
      action: callServerAction,
      refreshIsland: async (instance) => {
        const res = await fetch("/_resuma/island/" + encodeURIComponent(instance));
        if (!res.ok) return;
        const html = await res.text();
        const t = document.querySelector('resuma-island[data-r-instance="' + instance + '"]');
        if (t) t.outerHTML = html;
      },
    };
    bindReactiveText(root(), signals);
    bindReactiveAttrs(root(), signals);
    initIslands(root(), signals);
    attachEventDelegation();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootstrap, { once: true });
  } else {
    bootstrap();
  }
})();
