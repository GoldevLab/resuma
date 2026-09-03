/**
 * Lazy fallbacks for Popup / Modal / GestureView / VirtualFor.
 *
 * Happy path is native HTML from Rust (`popover`, `<dialog closedby>`, CSS
 * anchor). This chunk loads only when those markers exist.
 */

import { initNavModality, initPresence } from "./desktop.js";

type Cell = {
  value: unknown;
  subscribe: (fn: (v: unknown) => void) => () => void;
};

function onCleanup(fn: () => void): void {
  window.__resuma?.onCleanup?.(fn);
}

const hasPopover = () => typeof HTMLElement !== "undefined" && "popover" in HTMLElement.prototype;
const hasAnchor =
  typeof CSS !== "undefined" && typeof CSS.supports === "function" && CSS.supports("anchor-name: --x");
const hasClosedBy = () =>
  typeof HTMLDialogElement !== "undefined" && "closedBy" in HTMLDialogElement.prototype;
const hasInvoker =
  typeof HTMLButtonElement !== "undefined" && "command" in HTMLButtonElement.prototype;

function popupId(raw: string): string {
  return raw.startsWith("r-popup-") ? raw : `r-popup-${raw}`;
}
function modalId(raw: string): string {
  return raw.startsWith("r-modal-") ? raw : `r-modal-${raw}`;
}
function panelFor(id: string): HTMLElement | null {
  return document.getElementById(popupId(id)) ?? document.querySelector(`[data-r-popup="${CSS.escape(id)}"]`);
}
function triggerFor(id: string): HTMLElement | null {
  return (
    document.getElementById(`r-popup-anchor-${id}`) ??
    document.querySelector(`[data-r-popup-trigger="${CSS.escape(id)}"]`)
  );
}
function dialogFor(id: string): HTMLDialogElement | null {
  const el =
    document.getElementById(modalId(id)) ?? document.querySelector(`[data-r-modal="${CSS.escape(id)}"]`);
  return el instanceof HTMLDialogElement ? el : null;
}

function positionsOf(panel: HTMLElement): string[] {
  return (panel.getAttribute("data-r-popup-positions") ?? "bottom right top left")
    .split(/[\s,]+/)
    .filter(Boolean);
}

function placeFallback(panel: HTMLElement, anchor: HTMLElement): void {
  const ar = anchor.getBoundingClientRect();
  const pr = panel.getBoundingClientRect();
  const vw = innerWidth;
  const vh = innerHeight;
  const gap = 6;
  const order = positionsOf(panel);
  const tryPos = (pos: string) => {
    if (pos === "top") return { top: ar.top - pr.height - gap, left: ar.left };
    if (pos === "right") return { top: ar.top, left: ar.right + gap };
    if (pos === "left") return { top: ar.top, left: ar.left - pr.width - gap };
    return { top: ar.bottom + gap, left: ar.left };
  };
  let chosen = tryPos(order[0] ?? "bottom");
  for (const pos of order) {
    const c = tryPos(pos);
    if (c.top >= 0 && c.left >= 0 && c.top + pr.height <= vh && c.left + pr.width <= vw) {
      chosen = c;
      break;
    }
  }
  panel.style.position = "fixed";
  panel.style.top = `${Math.max(8, chosen.top)}px`;
  panel.style.left = `${Math.max(8, chosen.left)}px`;
  panel.setAttribute("data-r-popup-fallback", "");
}

function isPopoverOpen(panel: HTMLElement): boolean {
  try {
    if (panel.matches(":popover-open")) return true;
  } catch {
    /* selector unsupported */
  }
  return panel.classList.contains("r-popover-open");
}

export function showPopup(id: string): boolean {
  const panel = panelFor(id);
  const anchor = triggerFor(id);
  if (!panel || !anchor) return false;
  if (panel.hasAttribute("data-r-popup-dismiss-if-shown") && isPopoverOpen(panel)) {
    hidePopup(id);
    return false;
  }
  rememberTrigger(id, anchor);
  if (!hasAnchor) placeFallback(panel, anchor);
  if (hasPopover() && "showPopover" in panel) {
    try {
      (panel as HTMLElement & { showPopover: () => void }).showPopover();
    } catch {
      /* already open */
    }
  } else {
    panel.classList.add("r-popover-open");
    panel.style.display = "block";
  }
  return true;
}

export function hidePopup(id: string): void {
  const panel = panelFor(id);
  if (!panel) return;
  if (hasPopover() && "hidePopover" in panel) {
    try {
      (panel as HTMLElement & { hidePopover: () => void }).hidePopover();
    } catch {
      /* already hidden */
    }
  }
  panel.classList.remove("r-popover-open");
  if (!hasPopover()) panel.style.display = "none";
}

export function hideAllPopups(): void {
  document.querySelectorAll<HTMLElement>("[data-r-popup]").forEach((p) => {
    const id = p.getAttribute("data-r-popup");
    if (id) hidePopup(id);
  });
}

const modalStack: string[] = [];
const triggerByOverlay = new Map<string, HTMLElement>();

function rememberTrigger(id: string, el: HTMLElement): void {
  triggerByOverlay.set(id, el);
}

function restoreTrigger(id: string): void {
  const el = triggerByOverlay.get(id);
  triggerByOverlay.delete(id);
  if (el?.isConnected) window.__resuma?.focus?.(el);
}

export function showModal(id: string): boolean {
  const dialog = dialogFor(id);
  if (!dialog) return false;
  if (dialog.open) return true;
  const active = document.activeElement;
  if (active instanceof HTMLElement) rememberTrigger(id, active);
  dialog.showModal();
  modalStack.push(id);
  return true;
}

export function hideModal(id: string): void {
  const dialog = dialogFor(id);
  if (dialog?.open) dialog.close();
}

export function dismissAll(): void {
  hideAllPopups();
  [...modalStack].reverse().forEach(hideModal);
}

function syncPopupExpanded(panel: HTMLElement): void {
  if (!panel.id) return;
  const open = isPopoverOpen(panel);
  document.querySelectorAll(`[popovertarget="${CSS.escape(panel.id)}"]`).forEach((btn) => {
    btn.setAttribute("aria-expanded", open ? "true" : "false");
  });
}

function bindPopups(scope: ParentNode): void {
  scope.querySelectorAll<HTMLElement>("[data-r-popup]").forEach((panel) => {
    if (panel.dataset.rUiBound) return;
    panel.dataset.rUiBound = "1";
    const id = panel.getAttribute("data-r-popup") ?? "";
    const anchor = triggerFor(id);
    const onToggle = () => {
      syncPopupExpanded(panel);
      if (!hasAnchor && anchor && isPopoverOpen(panel)) placeFallback(panel, anchor);
    };
    panel.addEventListener("toggle", onToggle);
    syncPopupExpanded(panel);
    if (!hasPopover()) {
      panel.style.display = "none";
      anchor?.addEventListener("click", (ev) => {
        ev.preventDefault();
        if (isPopoverOpen(panel) && panel.hasAttribute("data-r-popup-dismiss-if-shown")) hidePopup(id);
        else showPopup(id);
      });
      const onDocClick = (ev: MouseEvent) => {
        if (!isPopoverOpen(panel)) return;
        const t = ev.target;
        if (!(t instanceof Node)) return;
        if (panel.contains(t) || anchor?.contains(t)) return;
        hidePopup(id);
      };
      document.addEventListener("click", onDocClick);
      onCleanup(() => document.removeEventListener("click", onDocClick));
    } else if (!hasAnchor && anchor) {
      const follow = () => {
        if (isPopoverOpen(panel)) placeFallback(panel, anchor);
      };
      window.addEventListener("scroll", follow, { capture: true, passive: true });
      onCleanup(() => window.removeEventListener("scroll", follow, true));
    }
  });
}

function bindModals(scope: ParentNode): void {
  scope.querySelectorAll<HTMLDialogElement>("dialog[data-r-modal]").forEach((dialog) => {
    if (dialog.dataset.rUiBound) return;
    dialog.dataset.rUiBound = "1";
    const id = dialog.getAttribute("data-r-modal") ?? dialog.id.replace(/^r-modal-/, "");
    if (!hasClosedBy()) {
      dialog.addEventListener("click", (event) => {
        if (event.target !== dialog) return;
        const rect = dialog.getBoundingClientRect();
        if (
          event.clientX >= rect.left &&
          event.clientX <= rect.right &&
          event.clientY >= rect.top &&
          event.clientY <= rect.bottom
        ) {
          return;
        }
        dialog.close();
      });
    }
    dialog.addEventListener("close", () => {
      const idx = modalStack.lastIndexOf(id);
      if (idx >= 0) modalStack.splice(idx, 1);
      restoreTrigger(id);
      const openSig = dialog.getAttribute("data-r-open-signal");
      if (openSig) {
        const cell = window.__resuma?.signals.get(openSig);
        if (cell && cell.value) cell.set(false);
      }
    });
  });
  scope.querySelectorAll<HTMLElement>("[data-r-modal-open]").forEach((btn) => {
    if (btn.dataset.rUiBound || (hasInvoker && btn.hasAttribute("command"))) return;
    btn.dataset.rUiBound = "1";
    btn.addEventListener("click", () => {
      const id = btn.getAttribute("data-r-modal-open");
      if (id) {
        rememberTrigger(id, btn);
        showModal(id);
      }
    });
  });
}

function bindOpenSignals(scope: ParentNode): void {
  const signals = window.__resuma?.signals;
  if (!signals) return;
  scope.querySelectorAll<HTMLElement>("[data-r-open-signal]").forEach((el) => {
    if (el.dataset.rOpenBound) return;
    el.dataset.rOpenBound = "1";
    const sid = el.getAttribute("data-r-open-signal");
    if (!sid) return;
    const cell = signals.get(sid);
    if (!cell) return;
    const id = el.getAttribute("data-r-popup") ?? el.getAttribute("data-r-modal") ?? "";
    const isModal = el.hasAttribute("data-r-modal");
    const apply = (v: unknown) => {
      const on = v === true || v === "true" || v === 1;
      if (isModal) {
        if (on) showModal(id);
        else hideModal(id);
      } else if (on) showPopup(id);
      else hidePopup(id);
    };
    apply(cell.value);
    onCleanup(cell.subscribe(apply));
  });
}

function dispatchGesture(el: HTMLElement, name: string, detail: unknown): void {
  const ev = new CustomEvent(name, { bubbles: true, cancelable: true, detail });
  const ref = el.getAttribute(`data-r-on:${name}`);
  const run = window.__resuma?.runHandler;
  if (ref && typeof run === "function") {
    const caps = (el.getAttribute(`data-r-cap:${name}`) ?? "")
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    void run(ref, el.getAttribute(`data-r-inline:${name}`), ev, caps, el);
    return;
  }
  el.dispatchEvent(ev);
}

function bindGestures(scope: ParentNode): void {
  scope.querySelectorAll<HTMLElement>("[data-r-gesture]").forEach((el) => {
    if (el.dataset.rUiBound) return;
    el.dataset.rUiBound = "1";
    const threshold = Number(el.getAttribute("data-r-pan-threshold") ?? "10") || 10;
    const preferred = el.getAttribute("data-r-preferred-pan") ?? "";
    let pointerId: number | null = null;
    let startX = 0;
    let startY = 0;
    let lastTap = 0;
    let longTimer = 0;
    let panning = false;
    const pointers = new Map<number, PointerEvent>();
    let pinchStart = 0;

    const clearLong = () => {
      if (longTimer) window.clearTimeout(longTimer);
      longTimer = 0;
    };

    el.addEventListener("pointerdown", (e) => {
      pointers.set(e.pointerId, e);
      if (pointers.size === 2) {
        const [a, b] = [...pointers.values()];
        pinchStart = Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
        clearLong();
        return;
      }
      pointerId = e.pointerId;
      startX = e.clientX;
      startY = e.clientY;
      panning = false;
      el.setPointerCapture?.(e.pointerId);
      clearLong();
      longTimer = window.setTimeout(() => dispatchGesture(el, "longpress", { x: startX, y: startY }), 500);
    });

    el.addEventListener("pointermove", (e) => {
      if (pointers.has(e.pointerId)) pointers.set(e.pointerId, e);
      if (pointers.size === 2 && pinchStart) {
        const [a, b] = [...pointers.values()];
        const dist = Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
        dispatchGesture(el, "pinch", { scale: dist / pinchStart });
        return;
      }
      if (pointerId !== e.pointerId) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      if (!panning && Math.hypot(dx, dy) < threshold) return;
      if (!panning) {
        if (preferred === "horizontal" && Math.abs(dx) < Math.abs(dy)) return;
        if (preferred === "vertical" && Math.abs(dy) < Math.abs(dx)) return;
        panning = true;
        clearLong();
      }
      const state = { dx, dy, startX, startY };
      dispatchGesture(el, "pan", state);
      dispatchGesture(el, Math.abs(dx) >= Math.abs(dy) ? "panhorizontal" : "panvertical", state);
    });

    const end = (e: PointerEvent) => {
      pointers.delete(e.pointerId);
      if (pointers.size < 2) pinchStart = 0;
      if (pointerId !== e.pointerId) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      clearLong();
      if (!panning && Math.hypot(dx, dy) < threshold) {
        const now = Date.now();
        if (now - lastTap < 300) dispatchGesture(el, "doubletap", { x: e.clientX, y: e.clientY });
        lastTap = now;
      }
      pointerId = null;
      panning = false;
    };
    el.addEventListener("pointerup", end);
    el.addEventListener("pointercancel", end);
    el.addEventListener(
      "wheel",
      (e) => dispatchGesture(el, "scrollwheel", { dx: e.deltaX, dy: e.deltaY }),
      { passive: true },
    );
  });
}

function itemLabel(item: unknown): string {
  if (item && typeof item === "object") {
    for (const key of ["title", "name", "label", "text"]) {
      const v = (item as Record<string, unknown>)[key];
      if (typeof v === "string") return v;
    }
  }
  return item == null ? "" : String(item);
}

function listKey(item: unknown, keyField: string | null, index: number): string {
  if (keyField && item && typeof item === "object") {
    const v = (item as Record<string, unknown>)[keyField];
    if (v !== undefined && v !== null) return String(v);
  }
  return String(index);
}

export function bindVirtualFor(
  el: HTMLElement,
  cell: Cell,
  keyField: string | null,
  bindSubtree: (node: HTMLElement) => void,
): void {
  const listEl = el.querySelector<HTMLElement>("[data-r-for-list]");
  if (!listEl) return;
  const itemHeight = Math.max(1, Number(el.getAttribute("data-r-item-height") ?? "48") || 48);
  const overscan = Math.max(0, Number(el.getAttribute("data-r-overscan") ?? "6") || 6);
  const scroller =
    (el.closest("[data-r-virtual-scroller]") as HTMLElement | null) ??
    (listEl.parentElement as HTMLElement | null) ??
    el;

  const ensureSpacer = (attr: string, atStart: boolean): HTMLElement => {
    let node = listEl.querySelector<HTMLElement>(`[${attr}]`);
    if (!node) {
      node = document.createElement("div");
      node.className = "r-virtual-spacer";
      node.setAttribute(attr, "");
      if (atStart) listEl.insertBefore(node, listEl.firstChild);
      else listEl.appendChild(node);
    }
    return node;
  };
  const topSpacer = ensureSpacer("data-r-virtual-top", true);
  const botSpacer = ensureSpacer("data-r-virtual-bot", false);
  const sample = listEl.querySelector<HTMLElement>("[data-r-for-item]") ?? undefined;
  const pool = new Map<string, HTMLElement>();

  const renderWindow = () => {
    const list = Array.isArray(cell.value) ? (cell.value as unknown[]) : [];
    const total = list.length;
    const viewH = scroller.clientHeight || window.innerHeight;
    const start = Math.max(0, Math.floor(scroller.scrollTop / itemHeight) - overscan);
    const visible = Math.ceil(viewH / itemHeight) + overscan * 2;
    const end = Math.min(total, start + visible);
    topSpacer.style.height = `${start * itemHeight}px`;
    botSpacer.style.height = `${Math.max(0, total - end) * itemHeight}px`;
    const keep = new Set<string>();
    const frag = document.createDocumentFragment();
    frag.appendChild(topSpacer);
    for (let i = start; i < end; i++) {
      const item = list[i];
      const key = listKey(item, keyField, i);
      keep.add(key);
      let node = pool.get(key);
      if (!node) {
        node = sample ? (sample.cloneNode(true) as HTMLElement) : document.createElement("div");
        node.setAttribute("data-r-for-item", "");
        node.setAttribute("data-r-for-new", "true");
        pool.set(key, node);
      }
      node.setAttribute("data-r-for-key", key);
      node.style.height = `${itemHeight}px`;
      if (!node.querySelector("[data-r-signal]") && node.childElementCount === 0) {
        node.textContent = itemLabel(item);
      }
      frag.appendChild(node);
    }
    frag.appendChild(botSpacer);
    listEl.replaceChildren(frag);
    pool.forEach((_, key) => {
      if (!keep.has(key)) pool.delete(key);
    });
    bindSubtree(listEl);
  };

  renderWindow();
  const onScroll = () => renderWindow();
  scroller.addEventListener("scroll", onScroll, { passive: true });
  onCleanup(() => scroller.removeEventListener("scroll", onScroll));
  onCleanup(cell.subscribe(renderWindow));
}

export function initUi(scope: ParentNode = document): void {
  initNavModality();
  initPresence();
  bindPopups(scope);
  bindModals(scope);
  bindOpenSignals(scope);
  bindGestures(scope);
}
