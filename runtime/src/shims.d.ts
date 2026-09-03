declare module "/_resuma/flow.js" {
  export function initFlowWidgets(scope: ParentNode): void;
}

declare module "/_resuma/ui.js" {
  export function initUi(scope: ParentNode): void;
  export function showPopup(id: string): boolean;
  export function hidePopup(id: string): void;
  export function showModal(id: string): boolean;
  export function hideModal(id: string): void;
  export function dismissAll(): void;
  export function bindVirtualFor(
    el: HTMLElement,
    cell: { value: unknown; subscribe: (fn: (v: unknown) => void) => () => void },
    keyField: string | null,
    bindSubtree: (node: HTMLElement) => void,
  ): void;
}

declare module "/_resuma/handler/*.js" {
  const mod: Record<string, (...args: unknown[]) => unknown>;
  export default mod;
}

declare module "/_resuma/island-chunk/*.js" {
  export function resume(
    props: unknown,
    signals: Map<string, unknown>,
    root: HTMLElement,
  ): void;
}
