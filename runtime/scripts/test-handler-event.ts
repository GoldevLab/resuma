import assert from "node:assert/strict";
import { withHandlerCurrentTarget } from "../src/handler-event.ts";

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

console.log("handler-event: ok");
