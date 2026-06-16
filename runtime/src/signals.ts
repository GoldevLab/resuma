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

/** Toggle `<Show>` branches bound to bool signals. */
export function bindShows(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  root.querySelectorAll<HTMLElement>("resuma-show").forEach((el) => {
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
        const target =
          document.getElementById(portalTargetId) ??
          document.querySelector<HTMLElement>(`[data-r-portal-target="${portalTargetId}"]`);
        if (target) {
          if (!on) {
            target.replaceChildren();
          } else if (portalTpl) {
            mountPortals(ifBranch);
          }
        }
      }
    };
    apply(cell.value);
    cell.subscribe(apply);
  });
}

function mountPortals(scope: HTMLElement): void {
  scope.querySelectorAll("template[data-r-portal]").forEach((tpl) => {
    const showBranch = tpl.closest<HTMLElement>("[data-r-show-if]");
    if (showBranch?.hidden) return;
    const targetId = tpl.getAttribute("data-r-portal");
    if (!targetId) return;
    const target =
      document.getElementById(targetId) ??
      document.querySelector(`[data-r-portal-target="${targetId}"]`);
    if (!target) return;
    target.replaceChildren(tpl.content.cloneNode(true));
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

function createForItemNode(item: unknown, key: string, sample: HTMLElement | undefined): HTMLElement {
  if (sample) {
    const node = sample.cloneNode(true) as HTMLElement;
    node.setAttribute("data-r-for-key", key);
    node.removeAttribute("data-r-for-new");
    const titleEl = node.querySelector(".todo-title");
    if (titleEl) {
      titleEl.textContent = itemLabel(item);
    }
    return node;
  }
  const wrap = document.createElement("div");
  wrap.setAttribute("data-r-for-item", "");
  wrap.setAttribute("data-r-for-key", key);
  const li = document.createElement("li");
  li.className = "todo-item";
  const span = document.createElement("span");
  span.className = "todo-title";
  span.textContent = itemLabel(item);
  li.appendChild(span);
  wrap.appendChild(li);
  return wrap;
}

function listKey(item: unknown, keyField: string | null, index: number): string {
  if (keyField && item && typeof item === "object") {
    const v = (item as Record<string, unknown>)[keyField];
    if (v !== undefined && v !== null) return String(v);
  }
  return String(index);
}

/** Keyed list reconciliation for `<For each={signal}>`. */
export function bindFor(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  root.querySelectorAll<HTMLElement>("resuma-for").forEach((el) => {
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

      const nextKeys: string[] = [];
      const frag = document.createDocumentFragment();

      const sample = listEl.querySelector<HTMLElement>("[data-r-for-item]:not([data-r-for-new])") ?? undefined;

      list.forEach((item, index) => {
        const key = listKey(item, keyField, index);
        nextKeys.push(key);
        let node = existing.get(key);
        if (node) {
          existing.delete(key);
        } else {
          node = createForItemNode(item, key, sample);
          node.setAttribute("data-r-for-new", "true");
        }
        frag.appendChild(node);
      });

      listEl.replaceChildren(frag);

      existing.forEach((orphan) => orphan.remove());
    };

    apply(cell.value);
    cell.subscribe(apply);
  });
}

/** Multi-branch match for `<Match value={signal}>`. */
export function bindMatch(root: HTMLElement, signals: Map<string, SignalCell<unknown>>): void {
  root.querySelectorAll<HTMLElement>("resuma-match").forEach((el) => {
    const sigId = el.getAttribute("data-r-match");
    if (!sigId) return;
    const cell = signals.get(sigId);
    if (!cell) return;
    const cases = el.querySelectorAll<HTMLElement>("[data-r-match-case]");
    const defaultBranch = el.querySelector<HTMLElement>("[data-r-match-default]");

    const apply = (v: unknown) => {
      const current =
        typeof v === "string" ? v : v === null || v === undefined ? "" : formatValue(v);
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
    cell.subscribe(apply);
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
