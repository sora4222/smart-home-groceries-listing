import { ImageOff } from "lucide-react";
import { useState } from "react";

import { cn } from "#/lib/utils";

const SIZES = {
	sm: "size-16",
	md: "size-20",
	lg: "size-24",
	// The store's own picture size, shrunk only to fit the row.
	full: "h-auto w-auto max-w-full",
} as const;

/** The placeholder needs a box even when the picture has its own size. */
const PLACEHOLDER = { ...SIZES, full: "size-24" } as const;

/**
 * A product's picture from the store's image server, at a fixed square size
 * so rows line up, or at the store's own size (`full`). When the store has no picture, or it fails to load, a
 * plain placeholder takes its place and says so to screen readers.
 */
export function ProductImage({
	src,
	name,
	size = "md",
}: {
	src: string | null | undefined;
	/** The product's name, used as the picture's text alternative. */
	name: string;
	size?: keyof typeof SIZES;
}) {
	// Remembering which link failed means a new link gets its own try.
	const [failedSrc, setFailedSrc] = useState<string | null>(null);
	if (!src || failedSrc === src) {
		return (
			<div
				role="img"
				aria-label={`No picture of ${name}`}
				className={cn(
					PLACEHOLDER[size],
					"shrink-0 rounded-md",
					"flex items-center justify-center bg-muted text-muted-foreground",
				)}
			>
				<ImageOff aria-hidden className="size-1/2" />
			</div>
		);
	}

	return (
		<img
			src={src}
			alt={name}
			loading="lazy"
			decoding="async"
			referrerPolicy="no-referrer"
			onError={() => setFailedSrc(src)}
			className={cn(
				SIZES[size],
				"shrink-0 rounded-md border border-border bg-background object-contain",
			)}
		/>
	);
}
