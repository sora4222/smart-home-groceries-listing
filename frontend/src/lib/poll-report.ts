import type { PollReport } from "#/lib/api";

function plural(count: number, one: string, many: string): string {
	return `${count} ${count === 1 ? one : many}`;
}

/** One short sentence saying what a Google Tasks check found. */
export function pollReportMessage(report: PollReport): string {
	const parts: string[] = [];
	if (report.recorded > 0) {
		parts.push(`${plural(report.recorded, "new item", "new items")} found.`);
	}
	if (report.blank > 0) {
		parts.push(
			`${plural(report.blank, "empty task", "empty tasks")} left alone.`,
		);
	}
	if (report.not_deleted > 0) {
		parts.push(
			`${plural(report.not_deleted, "task", "tasks")} could not be removed from Google — will try again.`,
		);
	}
	return parts.length > 0 ? parts.join(" ") : "Nothing new.";
}
