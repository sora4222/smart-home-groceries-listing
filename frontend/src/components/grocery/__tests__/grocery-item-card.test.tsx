import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { GroceryItemCard } from "#/components/grocery/grocery-item-card";
import { itemFixture } from "#/components/grocery/__tests__/fixtures";

describe("GroceryItemCard", () => {
	it("shows the name and quantity", () => {
		render(<GroceryItemCard item={itemFixture()} />);

		expect(screen.getByText("milk")).toBeInTheDocument();
		expect(screen.getByLabelText("Quantity 2")).toHaveTextContent("×2");
	});

	it("says where the item came from", () => {
		render(<GroceryItemCard item={itemFixture({ source: "voice" })} />);

		expect(screen.getByText("Added via voice")).toBeInTheDocument();
	});
});
