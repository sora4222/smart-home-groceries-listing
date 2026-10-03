import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { AccessLogList } from "#/components/access-logs/access-log-list";
import { logEntry } from "./fixtures";

describe("AccessLogList", () => {
	it("says so when nothing is logged", () => {
		render(<AccessLogList entries={[]} />);

		expect(screen.getByText("No requests logged yet.")).toBeInTheDocument();
	});

	it("shows each request's method, path, status, user, source and time", () => {
		render(
			<AccessLogList
				entries={[
					logEntry(),
					logEntry({
						id: 6,
						method: "POST",
						path: "/api/voice-requests",
						status_code: 401,
						user_id: "unauthenticated",
						source_ip: "172.18.0.5",
						forwarded_for: "203.0.113.9",
					}),
				]}
			/>,
		);

		const rows = within(
			screen.getByRole("list", { name: "Access log" }),
		).getAllByRole("listitem");
		expect(rows).toHaveLength(2);
		expect(rows[0]).toHaveTextContent("GET");
		expect(rows[0]).toHaveTextContent("/api/grocery-items");
		expect(rows[0]).toHaveTextContent("user_abc");
		expect(rows[0]).toHaveTextContent("192.168.1.20");
		expect(rows[0]).toHaveTextContent("4 ms");
		expect(within(rows[0]).getByText("200")).toBeInTheDocument();
		expect(rows[0].querySelector("time")).toHaveAttribute(
			"datetime",
			"2026-10-02T04:40:16Z",
		);
		expect(rows[1]).toHaveTextContent("Not signed in");
		expect(rows[1]).toHaveTextContent("172.18.0.5 (says 203.0.113.9)");
		expect(within(rows[1]).getByText("401")).toHaveAttribute(
			"data-variant",
			"outline",
		);
	});
});
