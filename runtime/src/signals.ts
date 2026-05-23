/**
 * Mirror of `resuma-core::Signal` on the client. A SignalCell is the smallest
 * possible reactive cell: a value plus an array of subscribers. When `.set()`
 * is called, every subscriber is invoked.
 */

export interface SignalCell<T> {
  readonly id: string;
  value: T;
  set(v: T): void;
  update(fn: (v: T) => T | void): void;
  subscribe(fn: (v: T) => void): () => void;
}

interface RawSignal { id: string; value: unknown; }

export function initSignals(raws: RawSignal[]): Map<string, SignalCell<unknown>> {
  const map = new Map<string, SignalCell<unknown>>();
  for (const r of raws) map.set(r.id, makeCell(r.id, r.value));
  return map;
}

function makeCell<T>(id: string, initial: T): SignalCell<T> {
  let value = initial;
  const subs = new Set<(v: T) => void>();
  const cell: SignalCell<T> = {
    id,
    get value() { return value; },
    set value(v: T) { cell.set(v); },
    set(v: T) {
      if (Object.is(v, value)) return;
      value = v;
      subs.forEach((s) => s(value));
    },
    update(fn) {
      const next = fn(value);
      if (next !== undefined) cell.set(next as T);
      else subs.forEach((s) => s(value));
    },
    subscribe(fn) { subs.add(fn); return () => subs.delete(fn); },
  };
  return cell;
}

const TEXT_TAG = "RESUMA-DYN";

export function bindReactiveText(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  const nodes = root.querySelectorAll<HTMLElement>(TEXT_TAG.toLowerCase());
  nodes.forEach((node) => {
    const sigId = node.getAttribute("data-r-signal");
    if (!sigId) return;
    const cell = signals.get(sigId);
    if (!cell) return;
    cell.subscribe((v) => { node.textContent = formatValue(v); });
  });
}

export function bindReactiveAttrs(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  const els = root.querySelectorAll<HTMLElement>("[data-r-bind]");
  // We support generic data-r-bind:<attr> attributes. Walk all attrs once.
  els.forEach((el) => bindElementAttrs(el, signals));
  // The previous selector won't catch attributes whose names contain colons
  // — fall back to attribute scan over all elements once.
  scanAndBindAttrs(root, signals);
}

function scanAndBindAttrs(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);
  let node: Node | null = walker.currentNode;
  while (node) {
    if (node instanceof HTMLElement) bindElementAttrs(node, signals);
    node = walker.nextNode();
  }
}

function bindElementAttrs(el: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  for (const attr of Array.from(el.attributes)) {
    const name = attr.name;
    if (!name.startsWith("data-r-bind:")) continue;
    const target = name.slice("data-r-bind:".length);
    const [sigId, fmt = "{}"] = attr.value.split("|");
    const cell = signals.get(sigId);
    if (!cell) continue;
    const apply = (v: unknown) => {
      const formatted = fmt.replace("{}", formatValue(v));
      el.setAttribute(target, formatted);
    };
    apply(cell.value);
    cell.subscribe(apply);
  }
}

function formatValue(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  if (typeof v === "number" || typeof v === "boolean") return String(v);
  try { return JSON.stringify(v); } catch { return String(v); }
}

/** Re-run all bindings after a partial DOM swap (HMR / island refresh). */
export function applyDom(): void {
  const r = (window as unknown as { __resuma?: { signals: Map<string, SignalCell<unknown>> } }).__resuma;
  if (!r) return;
  const root = document.getElementById("resuma-root") ?? document.body;
  bindReactiveText(root, r.signals);
  bindReactiveAttrs(root, r.signals);
}
