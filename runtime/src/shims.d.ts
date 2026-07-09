declare module "/_resuma/flow.js" {
  export function initFlowWidgets(scope: ParentNode): void;
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
