import assert from "node:assert/strict";
import { isHandlerMountCurrent, withHandlerCurrentTarget } from "../src/handler-event.ts";
import { initEffects } from "../src/effects.ts";
import { runVisibleTasks } from "../src/visible-tasks.ts";
import { flushMountCleanups, registerMountCleanup } from "../src/mount-cleanups.ts";

const ev = new Event("click", { bubbles: true, cancelable: true });
const handlerEl = { id: "btn" } as unknown as EventTarget;
const wrapped = withHandlerCurrentTarget(ev, handlerEl);

assert.equal(wrapped.currentTarget, handlerEl);
assert.equal(wrapped.type, "click");
assert.equal(wrapped.target, null);

wrapped.preventDefault();
assert.equal(ev.defaultPrevented, true);
assert.equal(wrapped.defaultPrevented, true);

const same = withHandlerCurrentTarget(wrapped, handlerEl);
assert.equal(same.currentTarget, handlerEl);
const firstMount = {};
assert.equal(isHandlerMountCurrent(firstMount, firstMount, handlerEl), true);
assert.equal(isHandlerMountCurrent(firstMount, {}, handlerEl), false);
assert.equal(
	isHandlerMountCurrent(firstMount, firstMount, { isConnected: false } as unknown as EventTarget),
	false,
);

const previousWindow = Object.getOwnPropertyDescriptor(globalThis, "window");
const previousIntersectionObserver = Object.getOwnPropertyDescriptor(globalThis, "IntersectionObserver");
Object.defineProperty(globalThis, "window", { configurable: true, value: {} });

try {
	let releaseEffects: (module: Record<string, Function>) => void = () => {};
	const effectsModule = new Promise<Record<string, Function>>((resolve) => {
		releaseEffects = resolve;
	});
	let effectRuns = 0;
	const global = { state: {}, signals: new Map() } as never;
	const mountingEffects = initEffects(
		[{ id: 1, deps: [], captures: {}, kind: "effect", symbol: "run" }],
		new Map(),
		global,
		() => effectsModule,
		registerMountCleanup,
	);
	flushMountCleanups();
	releaseEffects({ run: () => { effectRuns++; } });
	await mountingEffects;
	assert.equal(effectRuns, 0, "late effects must not run after an SPA unmount");

	let releaseModule: (module: Record<string, Function>) => void = () => {};
	const taskModule = new Promise<Record<string, Function>>((resolve) => {
		releaseModule = resolve;
	});
	let taskRuns = 0;
	runVisibleTasks(
		{ 1: { symbol: "run" } },
		new Map(),
		{},
		() => ({} as HTMLElement),
		global,
		() => taskModule,
		registerMountCleanup,
	);
	flushMountCleanups();
	releaseModule({ run: () => { taskRuns++; } });
	await new Promise<void>((resolve) => setImmediate(resolve));
	assert.equal(taskRuns, 0, "late visible tasks must not run after an SPA unmount");

	const observed: Element[] = [];
	const observers: Array<{ enter(target: Element): void }> = [];
	class FakeIntersectionObserver {
		constructor(private readonly callback: IntersectionObserverCallback) {
			observers.push({
				enter: (target) => callback([{ target, isIntersecting: true } as IntersectionObserverEntry], this as unknown as IntersectionObserver),
			});
		}
		observe(target: Element): void { observed.push(target); }
		unobserve(): void {}
		disconnect(): void {}
	}
	Object.defineProperty(globalThis, "IntersectionObserver", {
		configurable: true,
		value: FakeIntersectionObserver,
	});
	Object.defineProperty(globalThis.window, "IntersectionObserver", {
		configurable: true,
		value: FakeIntersectionObserver,
	});
	const anchors = [
		{ id: "owner-1" } as unknown as HTMLElement,
		{ id: "owner-2" } as unknown as HTMLElement,
	];
	const boundaries = anchors.map((anchor, index) => ({
		dataset: { rBoundaryId: String(index + 1) },
		querySelector: () => anchor,
	})) as unknown as HTMLElement[];
	let appendedMarkers = 0;
	const taskRoot = {
		querySelectorAll: (selector: string) => selector === "[data-r-boundary-id]" ? boundaries : [],
		appendChild: () => { appendedMarkers++; },
	} as unknown as HTMLElement;
	const ownedTasks: string[] = [];
	runVisibleTasks(
		{
			first: { symbol: "first", boundary_id: 1 },
			second: { symbol: "second", boundary_id: 2 },
		},
		new Map(),
		{},
		() => taskRoot,
		global,
		async () => ({
			first: () => { ownedTasks.push("first"); },
			second: () => { ownedTasks.push("second"); },
		}),
		registerMountCleanup,
	);
	assert.deepEqual(observed, anchors);
	assert.equal(appendedMarkers, 0, "owned tasks must not fall back to page-end markers");
	observers[0]?.enter(anchors[0]!);
	await new Promise<void>((resolve) => setImmediate(resolve));
	assert.deepEqual(ownedTasks, ["first"]);
	observers[0]?.enter(anchors[1]!);
	await new Promise<void>((resolve) => setImmediate(resolve));
	assert.deepEqual(ownedTasks, ["first", "second"]);
} finally {
	flushMountCleanups();
	if (previousWindow) Object.defineProperty(globalThis, "window", previousWindow);
	else delete (globalThis as typeof globalThis & { window?: unknown }).window;
	if (previousIntersectionObserver) Object.defineProperty(globalThis, "IntersectionObserver", previousIntersectionObserver);
	else delete (globalThis as typeof globalThis & { IntersectionObserver?: unknown }).IntersectionObserver;
}

console.log("runtime lifecycle: ok");
