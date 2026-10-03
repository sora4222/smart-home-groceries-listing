import { render } from "@testing-library/react";
import type { ReactNode } from "react";
import { vi } from "vitest";

import {
	ProductDislikesProvider,
	type ProductDislikesValue,
} from "#/components/dislikes/product-dislikes";
import type { ProductDislike } from "#/lib/api";

/** Phu's dislike of the Coles milk the product fixture describes. */
export function dislikeFixture(
	overrides: Partial<ProductDislike> = {},
): ProductDislike {
	return {
		store: "coles",
		store_name: "Coles",
		product_id: "c-milk-3l",
		name: "Full Cream Milk",
		brand: "Coles",
		package_size: "3L",
		user_id: "phu-id",
		user_name: "Phu",
		mine: false,
		disliked_at: "2026-10-01T10:00:00Z",
		...overrides,
	};
}

/** A comparison's dislike context with every action mocked. */
export function dislikesValue(
	overrides: Partial<ProductDislikesValue> = {},
): ProductDislikesValue {
	return {
		itemId: "item-1",
		dislikes: [],
		overrides: [],
		dislike: vi.fn().mockResolvedValue(undefined),
		removeMine: vi.fn().mockResolvedValue(undefined),
		override: vi.fn().mockResolvedValue(undefined),
		clearOverride: vi.fn().mockResolvedValue(undefined),
		...overrides,
	};
}

/** Renders `ui` inside a comparison's dislike context. */
export function renderWithDislikes(ui: ReactNode, value: ProductDislikesValue) {
	return render(
		<ProductDislikesProvider value={value}>{ui}</ProductDislikesProvider>,
	);
}
