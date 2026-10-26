import { useEffect, useState } from "react";

/**
 * The rendered width of an element, in CSS pixels, kept up to date as it
 * resizes. Pass the returned callback as the element's `ref`. Starts at
 * `fallback`, which is also what server rendering uses.
 */
export function useElementWidth<T extends Element>(
	fallback: number,
): [(element: T | null) => void, number] {
	const [element, setElement] = useState<T | null>(null);
	const [width, setWidth] = useState(fallback);

	useEffect(() => {
		if (!element || typeof ResizeObserver === "undefined") return;
		const observer = new ResizeObserver(([entry]) => {
			const next = Math.round(entry.contentRect.width);
			if (next > 0) setWidth(next);
		});
		observer.observe(element);
		return () => observer.disconnect();
	}, [element]);

	return [setElement, width];
}
