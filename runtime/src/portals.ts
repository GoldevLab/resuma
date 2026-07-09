/**
 * Portal projection — mount template content into remote targets without
 * clobbering other owners sharing the same target element.
 */

let ownerSeq = 0;
const ownerByElement = new WeakMap<HTMLElement, string>();
let currentMountOwners = new Set<string>();
let lastMountOwners = new Set<string>();

/** Remove portal slots registered by the previous page mount (SPA navigation). */
export function beginPortalMount(): void {
  for (const id of lastMountOwners) {
    document
      .querySelectorAll(`[data-r-portal-slot="${CSS.escape(id)}"]`)
      .forEach((n) => n.remove());
  }
  lastMountOwners = currentMountOwners;
  currentMountOwners = new Set();
}

export function findPortalTarget(targetId: string): HTMLElement | null {
  return (
    document.getElementById(targetId) ??
    document.querySelector<HTMLElement>(`[data-r-portal-target="${CSS.escape(targetId)}"]`)
  );
}

/** Stable owner id for a `<resuma-show>`, portal `<template>`, etc. */
export function portalOwnerId(ownerEl: HTMLElement): string {
  let id = ownerByElement.get(ownerEl);
  if (!id) {
    id = `p${++ownerSeq}`;
    ownerByElement.set(ownerEl, id);
  }
  currentMountOwners.add(id);
  return id;
}

export function clearPortalSlot(target: HTMLElement, ownerId: string): void {
  target
    .querySelectorAll(`[data-r-portal-slot="${CSS.escape(ownerId)}"]`)
    .forEach((n) => n.remove());
}

/** Replace this owner's slot inside `target` with cloned template content. */
export function mountPortalContent(
  tpl: HTMLTemplateElement,
  target: HTMLElement,
  ownerId: string,
): void {
  clearPortalSlot(target, ownerId);
  const slot = document.createElement("div");
  slot.dataset.rPortalSlot = ownerId;
  slot.appendChild(tpl.content.cloneNode(true));
  target.appendChild(slot);
}

/** Mount every portal template under `scope` not owned by reactive `<Show>`. */
export function mountStaticPortals(scope: HTMLElement): void {
  scope.querySelectorAll<HTMLTemplateElement>("template[data-r-portal]").forEach((tpl) => {
    const showBranch = tpl.closest<HTMLElement>("[data-r-show-if]");
    if (showBranch?.closest<HTMLElement>("resuma-show")?.dataset.rPortalTarget) return;
    if (showBranch?.hidden) return;
    const targetId = tpl.getAttribute("data-r-portal");
    if (!targetId) return;
    const target = findPortalTarget(targetId);
    if (!target) return;
    mountPortalContent(tpl, target, portalOwnerId(tpl));
  });
}

/** Mount portal templates inside a `<Show>` if-branch for one owner. */
export function mountShowPortals(ifBranch: HTMLElement, target: HTMLElement, ownerId: string): void {
  const tpl = ifBranch.querySelector<HTMLTemplateElement>("template[data-r-portal]");
  if (tpl) mountPortalContent(tpl, target, ownerId);
}
