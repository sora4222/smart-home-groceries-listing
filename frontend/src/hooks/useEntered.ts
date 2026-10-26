import { useEffect, useState } from "react";

/**
 * False on the first paint, true from the next frame: switch classes on it
 * to play a CSS transition when something appears.
 */
export function useEntered(): boolean {
	const [entered, setEntered] = useState(false);
	useEffect(() => {
		const frame = requestAnimationFrame(() => setEntered(true));
		return () => cancelAnimationFrame(frame);
	}, []);
	return entered;
}
