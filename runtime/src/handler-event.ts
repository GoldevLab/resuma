/**
 * Resuma delegates events from `document` (capture). The native Event's
 * `currentTarget` is therefore `document`, which breaks the DOM contract
 * handlers expect (`event.currentTarget` = the node with `data-r-on:*`).
 *
 * Wrap so `js!` / lazy chunks see the handler element as `currentTarget`.
 * Methods (`preventDefault`, `stopPropagation`, …) stay bound to the
 * original event.
 */
export function withHandlerCurrentTarget(ev: Event, handlerEl: EventTarget): Event {
  if (ev.currentTarget === handlerEl) return ev;
  return new Proxy(ev, {
    get(target, prop) {
      if (prop === "currentTarget") return handlerEl;
      const value = Reflect.get(target, prop, target);
      return typeof value === "function" ? value.bind(target) : value;
    },
  });
}

/** Walk from `event.target` to the node that declared this event's handler. */
export function inferHandlerElement(ev: Event): EventTarget | null {
  let target: Element | null =
    ev.target instanceof Element
      ? ev.target
      : ev.target instanceof Text
        ? ev.target.parentElement
        : null;
  const attr = `data-r-on:${ev.type}`;
  while (target && target !== document.body) {
    if (target.hasAttribute(attr)) return target;
    target = target.parentElement;
  }
  return null;
}

/** Prefer the node found by delegation; fall back to walking `event.target`. */
export function retargetHandlerEvent(ev: Event, handlerEl?: EventTarget | null): Event {
  const el = handlerEl ?? inferHandlerElement(ev);
  return el ? withHandlerCurrentTarget(ev, el) : ev;
}
