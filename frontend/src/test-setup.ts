import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";

afterEach(() => {
	cleanup();
});

// Radix Select captures the pointer and scrolls the chosen option into view;
// jsdom implements neither, so give it harmless stand-ins.
if (!Element.prototype.hasPointerCapture) {
	Element.prototype.hasPointerCapture = () => false;
	Element.prototype.releasePointerCapture = () => {};
}
if (!Element.prototype.scrollIntoView) {
	Element.prototype.scrollIntoView = () => {};
}
// Radix measures a switch inside a <form> and a radio group; jsdom has no
// ResizeObserver.
if (!("ResizeObserver" in globalThis)) {
	globalThis.ResizeObserver = class {
		observe() {}
		unobserve() {}
		disconnect() {}
	};
}
