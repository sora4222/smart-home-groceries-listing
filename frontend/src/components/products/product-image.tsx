import { ImageOff } from "lucide-react";
import { useState } from "react";

import { cn } from "#/lib/utils";

const SIZES = { sm: "size-10", md: "size-16", lg: "size-24" } as const;

/**
 * A product's picture from the store's image server, at a fixed square size
 * so rows line up. When the store has no picture, or it fails to load, a
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
	const box = cn(SIZES[size], "shrink-0 rounded-md");

	if (!src || failedSrc === src) {
		return (
			<div
				role="img"
				aria-label={`No picture of ${name}`}
				className={cn(
					box,
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
			className={cn(box, "border border-border bg-background object-contain")}
		/>
	);
}
