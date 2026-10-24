/** Optional client-side effects and visible-task runners. */

export type LoadHandlerChunk = (chunk: string) => Promise<Record<string, Function>>;
export type RegisterMountCleanup = (cleanup: () => void) => void;

export { initEffects, type ClientEffectSpec } from "./effects.js";
export { runVisibleTasks, type VisibleTaskEntry } from "./visible-tasks.js";