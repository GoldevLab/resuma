/**
 * Mirror of `resuma-core::Signal` on the client. A SignalCell is the smallest
 * possible reactive cell: a value plus an array of subscribers. When `.set()`
 * is called, every subscriber is invoked.
 */

import {
  clearPortalSlot,
  findPortalTarget,
  mountShowPortals,
  portalOwnerId,
} from "./portals.js";
import { registerMountCleanup } from "./mount-cleanups.js";

export interface SignalCell<T> {
  readonly id: string;
  value: T;
  set(v: T): void;
  update(fn: (v: T) => T | void): void;
  subscribe(fn: (v: T) => void): () => void;
}

export type RawSignalId = string | number | { 0: number };

interface RawSignal { id: RawSignalId; value: unknown; }

export function initSignals(raws: RawSignal[]): Map<string, SignalCell<unknown>> {
  const map = new Map<string, SignalCell<unknown>>();
  for (const r of raws) {
    const id = signalId(r.id);
    map.set(id, makeCell(id, r.value));
  }
  return map;
}

export function signalId(raw: RawSignalId): string {
  if (typeof raw === "string") return raw;
  if (typeof raw === "number") return `s${raw}`;
  return `s${raw[0]}`;
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
    },
    subscribe(fn) { subs.add(fn); return () => subs.delete(fn); },
  };
  return cell;
}

const TEXT_TAG = "RESUMA-DYN";

// Track already-bound nodes so repeated `applyDom()` passes (island refresh /
// HMR) never subscribe the same element twice — a duplicate subscription leaks
// closures and double-writes on every signal update. Nodes removed from the DOM
// are garbage-collected out of these sets automatically.
const boundTextNodes = new WeakSet<Element>();
const boundAttrEls = new WeakSet<Element>();
const boundShowEls = new WeakSet<Element>();
const boundForEls = new WeakSet<Element>();
const boundMatchEls = new WeakSet<Element>();

export function bindReactiveText(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  const nodes = root.querySelectorAll<HTMLElement>(TEXT_TAG.toLowerCase());
  nodes.forEach((node) => {
    if (boundTextNodes.has(node)) return;
    const sigId = node.getAttribute("data-r-signal");
    if (!sigId) return;
    const cell = signals.get(sigId);
    if (!cell) return;
    node.textContent = formatValue(cell.value);
    registerMountCleanup(cell.subscribe((v) => { node.textContent = formatValue(v); }));
    boundTextNodes.add(node);
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

// Attributes that must never receive a reactive (potentially user-derived)
// value: inline event handlers execute JS, and `style` allows CSS injection.
function isUnsafeBindTarget(target: string): boolean {
  const lower = target.toLowerCase();
  return lower.startsWith("on") || lower === "style";
}

// Attributes interpreted as URLs — sanitize dangerous schemes so a bound value
// like `javascript:...` cannot execute when the attribute is later activated.
const URL_BIND_ATTRS = new Set([
  "href",
  "src",
  "action",
  "formaction",
  "xlink:href",
  "poster",
  "background",
  "ping",
  "data",
]);

function sanitizeUrlValue(value: string): string {
  // Strip control chars/whitespace that browsers ignore when parsing schemes
  // (e.g. `java\tscript:`), then reject dangerous URL schemes.
  const collapsed = value.replace(/[\u0000-\u0020]+/g, "");
  if (/^(?:javascript|vbscript|data):/i.test(collapsed)) return "";
  return value;
}

function bindElementAttrs(el: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  if (boundAttrEls.has(el)) return;
  let boundAny = false;
  for (const attr of Array.from(el.attributes)) {
    const name = attr.name;
    if (!name.startsWith("data-r-bind:")) continue;
    const target = name.slice("data-r-bind:".length);
    if (isUnsafeBindTarget(target)) {
      // Refuse to reflect reactive values into event-handler / style attributes.
      continue;
    }
    const [sigId, fmt = "{}"] = attr.value.split("|");
    const cell = signals.get(sigId);
    if (!cell) continue;
    const isUrl = URL_BIND_ATTRS.has(target.toLowerCase());
    const apply = (v: unknown) => {
      let formatted = fmt.replace("{}", formatValue(v));
      if (isUrl) formatted = sanitizeUrlValue(formatted);
      el.setAttribute(target, formatted);
    };
    apply(cell.value);
    registerMountCleanup(cell.subscribe(apply));
    boundAny = true;
  }
  if (boundAny) boundAttrEls.add(el);
}

function formatValue(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (typeof v === "string") return v;
  if (typeof v === "number" || typeof v === "boolean") return String(v);
  try { return JSON.stringify(v); } catch { return String(v); }
}

/**
 * Mirror Rust `match_value_string` — keep SSR and client branch keys aligned.
 * Rust uses `serde_json::Value::to_string()` (JSON text for non-string scalars).
 */
export function matchValueString(v: unknown): string {
  if (v === undefined) return "";
  if (typeof v === "string") return v;
  try {
    const json = JSON.stringify(v);
    if (json === undefined) return "";
    const parsed: unknown = JSON.parse(json);
    if (typeof parsed === "string") return parsed;
    return json;
  } catch {
    return formatValue(v);
  }
}

/** Toggle `<Show>` branches bound to bool signals. */
export function bindShows(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  root.querySelectorAll<HTMLElement>("resuma-show").forEach((el) => {
    if (boundShowEls.has(el)) return;
    const sigId = el.getAttribute("data-r-show");
    if (!sigId) return;
    const inverted = el.getAttribute("data-r-inverted") === "true";
    const ifBranch = el.querySelector<HTMLElement>("[data-r-show-if]");
    const elseBranch = el.querySelector<HTMLElement>("[data-r-show-else]");
    const cell = signals.get(sigId);
    if (!cell || !ifBranch) return;
    const apply = (v: unknown) => {
      const on = inverted ? !Boolean(v) : Boolean(v);
      ifBranch.hidden = !on;
      if (elseBranch) elseBranch.hidden = on;
      let portalTargetId = el.dataset.rPortalTarget;
      const portalTpl = ifBranch.querySelector<HTMLTemplateElement>("template[data-r-portal]");
      if (!portalTargetId && portalTpl) {
        portalTargetId = portalTpl.getAttribute("data-r-portal") ?? undefined;
        if (portalTargetId) el.dataset.rPortalTarget = portalTargetId;
      }
      if (portalTargetId) {
        const target = findPortalTarget(portalTargetId);
        if (target) {
          const ownerId = portalOwnerId(el);
          if (!on) {
            clearPortalSlot(target, ownerId);
          } else if (portalTpl) {
            mountShowPortals(ifBranch, target, ownerId);
          }
        }
      }
    };
    apply(cell.value);
    // Register the unsubscribe so SPA remount (new signal map) does not leave
    // stale closures pinned to the previous cell forever.
    registerMountCleanup(cell.subscribe(apply));
    boundShowEls.add(el);
  });
}

function itemLabel(item: unknown): string {
  if (item && typeof item === "object") {
    for (const key of ["title", "name", "label", "text"]) {
      const v = (item as Record<string, unknown>)[key];
      if (typeof v === "string") return v;
    }
  }
  return formatValue(item);
}

function isSignalBoundDyn(el: HTMLElement): boolean {
  return el.hasAttribute("data-r-signal");
}

/** Bind reactive text/attrs inside a subtree (e.g. after `<For>` list reconciliation). */
export function bindReactiveSubtree(
  root: HTMLElement,
  signals: Map<string, SignalCell<unknown>>,
): void {
  bindReactiveText(root, signals);
  bindReactiveAttrs(root, signals);
}

function createForItemNode(item: unknown, key: string, sample: HTMLElement | undefined): HTMLElement {
  if (sample) {
    const node = sample.cloneNode(true) as HTMLElement;
    node.setAttribute("data-r-for-key", key);
    node.removeAttribute("data-r-for-new");
    // Refresh static placeholders; leave signal-bound `resuma-dyn` for bindReactiveSubtree.
    node.querySelectorAll<HTMLElement>("resuma-dyn").forEach((el) => {
      if (isSignalBoundDyn(el)) return;
      el.textContent = itemLabel(item);
    });
    // Legacy samples without resuma-dyn (e.g. todo demo) keep the old hook.
    if (!node.querySelector("resuma-dyn")) {
      const titleEl = node.querySelector(".todo-title");
      if (titleEl) {
        titleEl.textContent = itemLabel(item);
      }
    }
    return node;
  }
  const wrap = document.createElement("div");
  wrap.setAttribute("data-r-for-item", "");
  wrap.setAttribute("data-r-for-key", key);
  const label = document.createElement("span");
  label.textContent = itemLabel(item);
  wrap.appendChild(label);
  return wrap;
}

function updateForItemContent(node: HTMLElement, item: unknown): void {
  node.querySelectorAll<HTMLElement>("resuma-dyn").forEach((el) => {
    if (isSignalBoundDyn(el)) return;
    el.textContent = itemLabel(item);
  });
  if (!node.querySelector("resuma-dyn")) {
    const titleEl = node.querySelector(".todo-title");
    if (titleEl) titleEl.textContent = itemLabel(item);
  }
}

function listKey(item: unknown, keyField: string | null, index: number): string {
  if (keyField && item && typeof item === "object") {
    const v = (item as Record<string, unknown>)[keyField];
    if (v !== undefined && v !== null) return String(v);
  }
  return String(index);
}

/** Match SSR `for_list` duplicate-key disambiguation (`{key}:{index}` suffix). */
function listKeys(list: unknown[], keyField: string | null): string[] {
  const seen = new Set<string>();
  return list.map((item, index) => {
    let key = listKey(item, keyField, index);
    if (seen.has(key)) key = `${key}:${index}`;
    seen.add(key);
    return key;
  });
}

/** Keyed list reconciliation for `<For each={signal}>`. */
export function bindFor(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  root.querySelectorAll<HTMLElement>("resuma-for").forEach((el) => {
    if (boundForEls.has(el)) return;
    const sigId = el.getAttribute("data-r-for");
    if (!sigId) return;
    const keyField = el.getAttribute("data-r-key");
    const listEl = el.querySelector<HTMLElement>("[data-r-for-list]");
    const cell = signals.get(sigId);
    if (!cell || !listEl) return;

    const apply = (v: unknown) => {
      const list = Array.isArray(v) ? v : [];
      const existing = new Map<string, HTMLElement>();
      listEl.querySelectorAll<HTMLElement>("[data-r-for-item]").forEach((node) => {
        const key = node.getAttribute("data-r-for-key");
        if (key) existing.set(key, node);
      });

      const frag = document.createDocumentFragment();

      const sample = listEl.querySelector<HTMLElement>("[data-r-for-item]:not([data-r-for-new])") ?? undefined;

      const keys = listKeys(list, keyField);
      list.forEach((item, index) => {
        const key = keys[index]!;
        let node = existing.get(key);
        if (node) {
          existing.delete(key);
          updateForItemContent(node, item);
        } else {
          node = createForItemNode(item, key, sample);
          node.setAttribute("data-r-for-new", "true");
        }
        frag.appendChild(node);
      });

      listEl.replaceChildren(frag);

      existing.forEach((orphan) => orphan.remove());

      // New/cloned rows may contain `resuma-dyn` markers that were not bound on first pass.
      bindReactiveSubtree(listEl, signals);
    };

    apply(cell.value);
    registerMountCleanup(cell.subscribe(apply));
    boundForEls.add(el);
  });
}

/** Multi-branch match for `<Match value={signal}>`. */
export function bindMatch(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  root.querySelectorAll<HTMLElement>("resuma-match").forEach((el) => {
    if (boundMatchEls.has(el)) return;
    const sigId = el.getAttribute("data-r-match");
    if (!sigId) return;
    const cell = signals.get(sigId);
    if (!cell) return;
    const cases = el.querySelectorAll<HTMLElement>("[data-r-match-case]");
    const defaultBranch = el.querySelector<HTMLElement>("[data-r-match-default]");

    const apply = (v: unknown) => {
      const current = matchValueString(v);
      let matched = false;
      cases.forEach((branch) => {
        const when = branch.getAttribute("data-r-match-when") ?? "";
        const on = when === current;
        branch.hidden = !on;
        if (on) matched = true;
      });
      if (defaultBranch) {
        defaultBranch.hidden = matched;
      }
    };

    apply(cell.value);
    registerMountCleanup(cell.subscribe(apply));
    boundMatchEls.add(el);
  });
}

/** Re-run all bindings after a partial DOM swap (HMR / island refresh). */
export function applyDom(): void {
  const r = (window as unknown as { __resuma?: { signals: Map<string, SignalCell<unknown>> } }).__resuma;
  if (!r) return;
  const root = document.getElementById("resuma-root") ?? document.body;
  bindReactiveText(root, r.signals);
  bindReactiveAttrs(root, r.signals);
  bindShows(root, r.signals);
  bindFor(root, r.signals);
  bindMatch(root, r.signals);
}
